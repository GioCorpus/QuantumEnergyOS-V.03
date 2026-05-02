"""
Actions — Definición y registry de acciones ejecutables
"""

from dataclasses import dataclass
from typing import Callable

@dataclass
class Action:
    id: str
    name: str
    command: str
    description: str
    impact_kw: float = 0.0
    callback: Callable | None = None

class ActionRegistry:
    """Registry de acciones disponibles en el sistema."""

    def __init__(self):
        self._actions: dict[str, Action] = {}

    def register(self, action: Action) -> None:
        self._actions[action.id] = action

    def get(self, action_id: str) -> Action | None:
        return self._actions.get(action_id)

    def all(self) -> dict[str, Action]:
        return self._actions.copy()

    def execute(self, action_id: str, dry_run: bool = True) -> dict:
        """Ejecuta acción (síncrona, para Flask)."""
        action = self.get(action_id)
        if not action:
            return {"error": f"Unknown action: {action_id}"}

        if dry_run:
            return {
                "action": action_id,
                "status": "simulated",
                "command": action.command,
            }

        import subprocess
        try:
            result = subprocess.run(
                action.command,
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
            }
        except Exception as e:
            return {"action": action_id, "status": "error", "error": str(e)}

# Registry global
registry = ActionRegistry()

# Registrar acciones por defecto
registry.register(Action(
    id="limit_cpu_frequency",
    name="Limitar frecuencia CPU",
    command="cpupower frequency-set -g powersave || true",
    description="Reducir CPU a modo ahorro",
    impact_kw=15.0,
))

registry.register(Action(
    id="shutdown_non_essential_services",
    name="Apagar servicios no críticos",
    command="systemctl stop docker selenium chromium-browser || true",
    description="Apaga contenedores y navegadores",
    impact_kw=25.0,
))

registry.register(Action(
    id="activate_energy_saving_mode",
    name="Activar modo ahorro energético",
    command="echo 'powersave' > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true",
    description="Configura CPUs en modo ahorro",
    impact_kw=10.0,
))

registry.register(Action(
    id="enable_night_mode_early",
    name="Activar modo nocturno anticipado",
    command="systemctl start night-mode-daemon || true",
    description="Inicia modo nocturno (iluminación, climatización)",
    impact_kw=8.0,
))

registry.register(Action(
    id="balance_grid_load",
    name="Balancear carga eléctrica",
    command="/opt/qeos/bin/grid-balancer --auto || true",
    description="Redistribye carga entre nodos de la red",
    impact_kw=0.0,
))

registry.register(Action(
    id="trigger_battery_backup",
    name="Activar respaldo de baterías",
    command="systemctl start ups-mode || true",
    description="Conecta respaldo UPS/baterías",
    impact_kw=0.0,
))

registry.register(Action(
    id="scale_down_compute_nodes",
    name="Escalar nodos de cómputo",
    command="/opt/qeos/bin/k8s-scale-down --critical || true",
    description="Apaga nodos Kubernetes no esenciales",
    impact_kw=50.0,
))
