"""
Quantum Climate Orchestrator V1 — Sistema de predicción y manejo de climas extremos
===================================================================================
Integra: predicción meteorológica · evaluación de riesgo · optimización energética
Autor: GioCorpus — QuantumEnergyOS, Mexicali, B.C.
"""

from __future__ import annotations

import json
import logging
import math
import os
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Any

import requests

try:
    from lru import LRU
except ImportError:
    LRU = dict  # Fallback simple

# ── Configuración de logging ────────────────────────────────────────────────────
log = logging.getLogger("qeos.climate")

# ── Enums ────────────────────────────────────────────────────────────────────────
class RiskLevel(str, Enum):
    NORMAL = "NORMAL"
    WARNING = "WARNING"
    CRITICAL = "CRITICAL"

class EventType(str, Enum):
    HEATWAVE = "heatwave"
    GRID_OVERLOAD = "grid_overload"
    BLACKOUT_RISK = "blackout_risk"
    CPU_OVERHEAT = "cpu_overheat"
    ENERGY_SHORTAGE = "energy_shortage"

# ── Dataclasses ──────────────────────────────────────────────────────────────────
@dataclass
class ClimateData:
    location: str
    temperature_c: float
    humidity: int
    power_grid_load: float  # 0.0–1.0
    cpu_load: float         # 0.0–1.0
    energy_reserve: int     # percentage
    time_of_day: str        # "HH:MM"
    forecast_next_6h_temp: list[float] = field(default_factory=list)
    wind_kph: float = 0.0

@dataclass
class Prediction:
    event_type: EventType
    confidence: float       # 0.0–1.0
    time_to_event_min: int
    description: str

@dataclass
class Action:
    id: str
    name: str
    command: str
    description: str
    impact_kw: float = 0.0

@dataclass
class OrchestratorResult:
    risk_level: RiskLevel
    predictions: list[Prediction]
    recommended_actions: list[Action]
    explanation: str
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

# ── Climate Orchestrator Core ────────────────────────────────────────────────────
class ClimateOrchestrator:
    """
    Orquestador inteligente de respuesta energética ante climas extremos.
    Combina: datos meteorológicos · estado del sistema · reglas de decisión
    """

    def __init__(
        self,
        openweather_api_key: str | None = None,
        location: str = "Mexicali,Baja California,MX",
        cache_ttl: int = 300,  # 5 minutos
    ):
        self.openweather_api_key = openweather_api_key or os.environ.get("OPENWEATHER_API_KEY")
        self.location = location
        self.cache_ttl = cache_ttl
        # Cache: usar LRU si disponible, sino dict simple
        if LRU is dict:
            self._cache = {}
        else:
            self._cache = LRU(256)

        # Umbrales de riesgo (ajustables)
        self.TEMP_CRITICAL_C = 45.0
        self.TEMP_WARNING_C = 40.0
        self.GRID_LOAD_CRITICAL = 0.90
        self.GRID_LOAD_WARNING = 0.80
        self.CPU_LOAD_CRITICAL = 0.85
        self.CPU_LOAD_WARNING = 0.70
        self.ENERGY_RESERVE_CRITICAL = 20
        self.ENERGY_RESERVE_WARNING = 40

        # Acciones predefinidas
        self._actions_registry = self._init_actions()

    def _init_actions(self) -> dict[str, Action]:
        """Registro de acciones ejecutables en el sistema."""
        return {
            "limit_cpu_frequency": Action(
                id="limit_cpu_frequency",
                name="Limitar frecuencia CPU",
                command="cpupower frequency-set -g powersave || true",
                description="Reduce CPU a modo ahorro (soporta Intel/AMD)",
                impact_kw=15.0,
            ),
            "shutdown_non_essential_services": Action(
                id="shutdown_non_essential_services",
                name="Apagar servicios no críticos",
                command="systemctl stop docker selenium chromium-browser || true",
                description="Apaga contenedores y navegadores",
                impact_kw=25.0,
            ),
            "activate_energy_saving_mode": Action(
                id="activate_energy_saving_mode",
                name="Activar modo ahorro energético",
                command="echo 'performance' > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true",
                description="Configura CPUs en modo ahorro",
                impact_kw=10.0,
            ),
            "enable_night_mode_early": Action(
                id="enable_night_mode_early",
                name="Activar modo nocturno anticipado",
                command="systemctl start night-mode-daemon || true",
                description="Inicia modo nocturno (iluminación, climatización)",
                impact_kw=8.0,
            ),
            "balance_grid_load": Action(
                id="balance_grid_load",
                name="Balancear carga eléctrica",
                command="/opt/qeos/bin/grid-balancer --auto || true",
                description="Redistribye carga entre nodos de la red",
                impact_kw=0.0,  # optimización, no reduce directamente
            ),
            "trigger_battery_backup": Action(
                id="trigger_battery_backup",
                name="Activar respaldo de baterías",
                command="systemctl start ups-mode || true",
                description="Conecta respaldo UPS/baterías",
                impact_kw=0.0,
            ),
            "reduce_solar_inverters": Action(
                id="reduce_solar_inverters",
                name="Reducir inversores solares",
                command="ssh solar-control 'inverter-set -m 50%%' || true",
                description="Reduce producción solar para evitar sobrecorte",
                impact_kw=0.0,
            ),
            "scale_down_compute_nodes": Action(
                id="scale_down_compute_nodes",
                name="Escalar nodos de cómputo",
                command="/opt/qeos/bin/k8s-scale-down --critical || true",
                description="Apaga nodos Kubernetes no esenciales",
                impact_kw=50.0,
            ),
        }

    # ── Predicción y evaluación ────────────────────────────────────────────────────
    def evaluate(self, data: ClimateData) -> OrchestratorResult:
        """
        Evalúa condiciones y determina riesgo + acciones.

        Args:
            data: Datos actuales del sistema + clima

        Returns:
            OrchestratorResult con riesgo, predicciones y acciones recomendadas
        """
        predictions: list[Prediction] = []
        risk_factors: dict[str, float] = {}

        # 1. Evaluar temperatura
        if data.temperature_c >= self.TEMP_CRITICAL_C:
            risk_factors["extreme_heat"] = 0.9
            predictions.append(Prediction(
                event_type=EventType.HEATWAVE,
                confidence=0.95,
                time_to_event_min=60,
                description=f"Ola de calor extremo ({data.temperature_c}°C) — pico en 1–2h",
            ))
        elif data.temperature_c >= self.TEMP_WARNING_C:
            risk_factors["extreme_heat"] = 0.5

        # 2. Evaluar carga de red
        if data.power_grid_load >= self.GRID_LOAD_CRITICAL:
            risk_factors["grid_overload"] = 0.9
            predictions.append(Prediction(
                event_type=EventType.GRID_OVERLOAD,
                confidence=0.90,
                time_to_event_min=30,
                description=f"Saturación eléctrica ({data.power_grid_load*100:.0f}%) — riesgo de apagón en 30 min",
            ))
        elif data.power_grid_load >= self.GRID_LOAD_WARNING:
            risk_factors["grid_overload"] = 0.4

        # 3. Evaluar CPU
        if data.cpu_load >= self.CPU_LOAD_CRITICAL:
            risk_factors["cpu_overheat"] = 0.8
            predictions.append(Prediction(
                event_type=EventType.CPU_OVERHEAT,
                confidence=0.85,
                time_to_event_min=15,
                description=f"Carga CPU crítica ({data.cpu_load*100:.0f}%) — throttling inminente",
            ))
        elif data.cpu_load >= self.CPU_LOAD_WARNING:
            risk_factors["cpu_overheat"] = 0.3

        # 4. Evaluar reserva energética
        if data.energy_reserve <= self.ENERGY_RESERVE_CRITICAL:
            risk_factors["energy_shortage"] = 0.9
            predictions.append(Prediction(
                event_type=EventType.ENERGY_SHORTAGE,
                confidence=0.88,
                time_to_event_min=45,
                description=f"Reserva energética crítica ({data.energy_reserve}%) — modo emergencia",
            ))
        elif data.energy_reserve <= self.ENERGY_RESERVE_WARNING:
            risk_factors["energy_shortage"] = 0.4

        # 5. Predicción por pronóstico (si hay datos)
        if data.forecast_next_6h_temp:
            next_max = max(data.forecast_next_6h_temp[:3])  # próximas 3h
            if next_max > data.temperature_c + 3:
                risk_factors["rising_heat"] = 0.6
                predictions.append(Prediction(
                    event_type=EventType.HEATWAVE,
                    confidence=0.70,
                    time_to_event_min=180,
                    description=f"Temperatura subirá a {next_max}°C en ~3h",
                ))

        # 6. Multi-riesgo: Combinaciones peligrosas
        if risk_factors.get("extreme_heat") and risk_factors.get("grid_overload"):
            risk_factors["compound_risk"] = 0.95
            predictions.append(Prediction(
                event_type=EventType.BLACKOUT_RISK,
                confidence=0.93,
                time_to_event_min=20,
                description="🔥 Ola de calor + saturación eléctrica — APAGÓN INMINENTE",
            ))

        # Determinar nivel de riesgo
        max_risk = max(risk_factors.values()) if risk_factors else 0.1
        if max_risk >= 0.85:
            risk_level = RiskLevel.CRITICAL
        elif max_risk >= 0.5:
            risk_level = RiskLevel.WARNING
        else:
            risk_level = RiskLevel.NORMAL

        # Seleccionar acciones
        actions = self._select_actions(risk_level, risk_factors, data)

        # Generar explicación
        explanation = self._generate_explanation(risk_level, predictions, actions, data)

        return OrchestratorResult(
            risk_level=risk_level,
            predictions=predictions,
            recommended_actions=actions,
            explanation=explanation,
        )

    def _select_actions(
        self,
        risk_level: RiskLevel,
        risk_factors: dict[str, float],
        data: ClimateData,
    ) -> list[Action]:
        """Selecciona acciones basadas en riesgo y factores."""
        selected: list[Action] = []

        if risk_level == RiskLevel.NORMAL:
            return []

        # CRITICAL: acción inmediata
        if risk_level == RiskLevel.CRITICAL:
            # Siempre limitar CPU en crítico
            selected.append(self._actions_registry["limit_cpu_frequency"])
            selected.append(self._actions_registry["shutdown_non_essential_services"])

            if data.energy_reserve <= 15:
                selected.append(self._actions_registry["trigger_battery_backup"])
                selected.append(self._actions_registry["scale_down_compute_nodes"])

            if "grid_overload" in risk_factors:
                selected.append(self._actions_registry["balance_grid_load"])

        # WARNING: acciones preventivas
        elif risk_level == RiskLevel.WARNING:
            selected.append(self._actions_registry["activate_energy_saving_mode"])
            if data.energy_reserve <= 30:
                selected.append(self._actions_registry["enable_night_mode_early"])

        # Deduplicar
        seen = set()
        unique_actions = []
        for a in selected:
            if a.id not in seen:
                seen.add(a.id)
                unique_actions.append(a)

        return unique_actions

    def _generate_explanation(
        self,
        risk_level: RiskLevel,
        predictions: list[Prediction],
        actions: list[Action],
        data: ClimateData,
    ) -> str:
        """Genera narrativa de la decisión."""
        if risk_level == RiskLevel.NORMAL:
            return f"Sistema estable — temperatura {data.temperature_c}°C, carga {data.power_grid_load*100:.0f}%.无明显异常."

        lines = [f"Nivel de riesgo: {risk_level}"]

        if predictions:
            lines.append("\n🔮 Predicciones:")
            for p in predictions:
                lines.append(f"  • {p.description}")

        if actions:
            lines.append("\n⚡ Acciones recomendadas:")
            for a in actions:
                lines.append(f"  • {a.name}: {a.command}")

        return "\n".join(lines)

    # ── Integración con APIs meteorológicas ─────────────────────────────────────
    def fetch_weather_openweather(self, lat: float, lon: float) -> dict[str, Any] | None:
        """
        Obtiene datos meteorológicos de OpenWeatherMap.
        cache_key = f"owm:{lat:.3f}:{lon:.3f}"
        cached = self._cache.get(cache_key)
        if cached:
            return cached

        if not self.openweather_api_key:
            log.warning("OPENWEATHER_API_KEY no configurada")
            return None

        try:
            url = "https://api.openweathermap.org/data/2.5/weather"
            params = {
                "lat": lat,
                "lon": lon,
                "appid": self.openweather_api_key,
                "units": "metric",
            }
            resp = requests.get(url, params=params, timeout=10)
            resp.raise_for_status()
            data = resp.json()

            # Extraer datos relevantes
            result = {
                "temp": data["main"]["temp"],
                "humidity": data["main"]["humidity"],
                "wind": data["wind"]["speed"],  # m/s → mph luego
                "pressure": data["main"]["pressure"],
                "description": data["weather"][0]["description"],
                "timestamp": datetime.now(timezone.utc).isoformat(),
            }

            self._cache[cache_key] = result
            return result

        except Exception as e:
            log.error(f"Error OpenWeatherMap: {e}")
            return None

    def fetch_forecast_openweather(self, lat: float, lon: float) -> list[float] | None:
        """Pronóstico de temperatura próximas 6h."""
        cache_key = f"owm_forecast:{lat:.3f}:{lon:.3f}"
        cached = self._cache.get(cache_key)
        if cached:
            return cached

        if not self.openweather_api_key:
            return None

        try:
            url = "https://api.openweathermap.org/data/2.5/forecast"
            params = {
                "lat": lat,
                "lon": lon,
                "appid": self.openweather_api_key,
                "units": "metric",
            }
            resp = requests.get(url, params=params, timeout=10)
            resp.raise_for_status()
            data = resp.json()

            # Tomar temperaturas de próximos 18 puntos (6h, cada 3h)
            temps = []
            for item in data["list"][:6]:
                temps.append(round(item["main"]["temp"], 1))

            self._cache[cache_key] = temps
            return temps

        except Exception as e:
            log.error(f"Error forecast OWM: {e}")
            return None

    # ── Integración con datos históricos locales ────────────────────────────────
    def load_historical_patterns(self, location: str) -> dict[str, Any]:
        """Carga patrones históricos de consumo/clima desde JSON local."""
        # Buscar archivo de datos
        base_dir = os.path.dirname(__file__)
        data_file = os.path.join(base_dir, "data", f"{location.lower().replace(' ', '-')}.json")

        if os.path.exists(data_file):
            with open(data_file, "r", encoding="utf-8") as f:
                return json.load(f)
        return {}

    # ── Ejecución de acciones ─────────────────────────────────────────────────────
    def execute_action(self, action: Action, dry_run: bool = True) -> dict[str, Any]:
        """
        Ejecuta una acción en el sistema.
        dry_run=True: solo simula (log), no ejecuta comando real.
        """
        log.info(f"[ACTION] {action.name}: {action.command}")

        if dry_run:
            return {
                "action": action.id,
                "status": "simulated",
                "command": action.command,
                "dry_run": True,
            }

        try:
            import subprocess
            result = subprocess.run(
                action.command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            return {
                "action": action.id,
                "status": "success" if result.returncode == 0 else "failed",
                "stdout": result.stdout[:200],
                "stderr": result.stderr[:200],
                "returncode": result.returncode,
            }
        except Exception as e:
            return {"action": action.id, "status": "error", "error": str(e)}

    # ── Ciclo automático ──────────────────────────────────────────────────────────
    def run_autonomous_cycle(
        self,
        weather_lat: float = 32.6245,
        weather_lon: float = -115.4523,
        execute_actions: bool = False,
        dry_run: bool = True,
    ) -> OrchestratorResult:
        """
        Ciclo completo: obtener datos → evaluar → decidir → ejecutar (opcional).
        """
        # 1. Obtener datos meteorológicos
        weather = self.fetch_weather_openweather(weather_lat, weather_lon)

        # 2. Obtener forecast
        forecast = self.fetch_forecast_openweather(weather_lat, weather_lon)

        # 3. Construir ClimateData (combinar con métricas del sistema)
        system_metrics = self._get_system_metrics()

        data = ClimateData(
            location=self.location,
            temperature_c=weather["temp"] if weather else system_metrics.get("cpu_temp_c", 35.0),
            humidity=weather["humidity"] if weather else 50,
            power_grid_load=system_metrics.get("grid_load", 0.70),
            cpu_load=system_metrics.get("cpu_load", 0.50),
            energy_reserve=system_metrics.get("energy_reserve_pct", 80),
            time_of_day=datetime.now().strftime("%H:%M"),
            forecast_next_6h_temp=forecast or [],
            wind_kph=weather["wind"] * 3.6 if weather else 0,
        )

        # 4. Evaluar
        result = self.evaluate(data)

        # 5. Ejecutar acciones si se solicita
        if execute_actions and result.risk_level != RiskLevel.NORMAL:
            execution_results = []
            for action in result.recommended_actions:
                exec_result = self.execute_action(action, dry_run=dry_run)
                execution_results.append(exec_result)
            # Podría añadirse al resultado

        return result

    def _get_system_metrics(self) -> dict[str, Any]:
        """Recolecta métricas del sistema local (CPU, energía, etc.)."""
        metrics: dict[str, Any] = {}

        try:
            import psutil

            # CPU
            metrics["cpu_load"] = psutil.cpu_percent(interval=1) / 100.0

            # Temperatura (si está disponible)
            temps = psutil.sensors_temperatures()
            if temps:
                for name, entries in temps.items():
                    if entries:
                        metrics[f"cpu_temp_c"] = entries[0].current
                        break

            # Memoria
            mem = psutil.virtual_memory()
            metrics["memory_used_pct"] = mem.percent / 100.0

        except ImportError:
            log.warning("psutil no disponible — usando valores por defecto")

        # Valores por defecto si no hay psutil
        metrics.setdefault("cpu_load", 0.5)
        metrics.setdefault("cpu_temp_c", 45.0)

        return metrics


# ── Función de conveniencia ──────────────────────────────────────────────────────
def create_orchestrator() -> ClimateOrchestrator:
    """Factory que crea el orquestador desde variables de entorno."""
    api_key = os.environ.get("OPENWEATHER_API_KEY")
    location = os.environ.get("QEOS_LOCATION", "Mexicali,Baja California,MX")
    return ClimateOrchestrator(openweather_api_key=api_key, location=location)
