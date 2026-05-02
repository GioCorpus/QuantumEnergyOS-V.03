"""
Bridge Rust — Climate Actions (QuantumEnergyOS)
================================================
Puente entre Python (climate_orchestrator) y acciones de bajo nivel en Rust.
Permite ejecutar acciones energéticas de forma segura y eficiente.
"""

import asyncio
import json
import logging
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

log = logging.getLogger("qeos.climate_bridge")

# ── Configuración ────────────────────────────────────────────────────────────────
BRIDGE_PATH = Path(__file__).parent.parent / "photonic-bridge" / "target" / "release"
if sys.platform == "win32":
    BRIDGE_EXE = BRIDGE_PATH / "photonic-bridge.exe"
else:
    BRIDGE_EXE = BRIDGE_PATH / "photonic-bridge"

# Fallback a comandos nativos si el bridge no está disponible
NATIVE_COMMANDS = {
    "limit_cpu_frequency": "cpupower frequency-set -g powersave || true",
    "shutdown_non_essential_services": "systemctl stop docker selenium chromium-browser || true",
    "activate_energy_saving_mode": "echo 'powersave' > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true",
    "enable_night_mode_early": "systemctl start night-mode-daemon || true",
    "balance_grid_load": "/opt/qeos/bin/grid-balancer --auto || true",
    "trigger_battery_backup": "systemctl start ups-mode || true",
    "scale_down_compute_nodes": "/opt/qeos/bin/k8s-scale-down --critical || true",
}


class ClimateBridge:
    """
    Puente para ejecutar acciones climáticas/energéticas.
    Usa el binario Rust si está disponible, fallback a subprocess.
    """

    def __init__(self, use_rust_bridge: bool = True):
        self.use_rust_bridge = use_rust_bridge and BRIDGE_EXE.exists()
        log.info(f"ClimateBridge — use_rust: {self.use_rust_bridge}")

    async def execute(self, action_id: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        """
        Ejecuta una acción de forma asincrónica.
        """
        if self.use_rust_bridge:
            return await self._execute_rust(action_id, params or {})
        else:
            return await self._execute_native(action_id)

    async def _execute_rust(self, action_id: str, params: dict[str, Any]) -> dict[str, Any]:
        """Ejecuta vía bridge Rust (IPCJSON)."""
        try:
            proc = await asyncio.create_subprocess_exec(
                str(BRIDGE_EXE),
                "climate-action",
                action_id,
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )

            payload = json.dumps(params).encode("utf-8")
            stdout, stderr = await proc.communicate(input=payload, timeout=15)

            if proc.returncode != 0:
                return {
                    "action": action_id,
                    "status": "failed",
                    "error": stderr.decode().strip(),
                    "backend": "rust-failed",
                }

            result = json.loads(stdout.decode("utf-8"))
            result["backend"] = "rust"
            return result

        except Exception as e:
            log.error(f"Rust bridge error: {e}")
            return {
                "action": action_id,
                "status": "error",
                "error": str(e),
                "fallback": "native",
            }

    async def _execute_native(self, action_id: str) -> dict[str, Any]:
        """Ejecuta usando subprocess nativo de Python."""
        cmd = NATIVE_COMMANDS.get(action_id)
        if not cmd:
            return {
                "action": action_id,
                "status": "unknown",
                "error": f"Acción no reconocida: {action_id}",
            }

        try:
            proc = await asyncio.create_subprocess_shell(
                cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await proc.communicate(timeout=30)

            return {
                "action": action_id,
                "status": "success" if proc.returncode == 0 else "failed",
                "stdout": stdout.decode()[:200],
                "stderr": stderr.decode()[:200],
                "returncode": proc.returncode,
                "backend": "native",
            }

        except Exception as e:
            return {"action": action_id, "status": "error", "error": str(e)}

    # ── Método síncrono para compatibilidad ───────────────────────────────────────
    def execute_sync(self, action_id: str, dry_run: bool = True) -> dict[str, Any]:
        """Ejecución síncrona (para llamadas desde Flask sin asyncio)."""
        if dry_run:
            return {
                "action": action_id,
                "status": "simulated",
                "command": NATIVE_COMMANDS.get(action_id, "unknown"),
                "dry_run": True,
            }

        try:
            cmd = NATIVE_COMMANDS.get(action_id)
            if not cmd:
                return {"action": action_id, "status": "unknown", "error": "Unknown action"}

            result = subprocess.run(
                cmd,
                shell=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            return {
                "action": action_id,
                "status": "success" if result.returncode == 0 else "failed",
                "stdout": result.stdout[:200],
                "stderr": result.stderr[:200],
                "returncode": result.returncode,
            }
        except Exception as e:
            return {"action": action_id, "status": "error", "error": str(e)}


# Singleton global
_bridge: ClimateBridge | None = None

def get_bridge() -> ClimateBridge:
    global _bridge
    if _bridge is None:
        _bridge = ClimateBridge(use_rust_bridge=os.environ.get("USE_RUST_BRIDGE", "1") == "1")
    return _bridge
