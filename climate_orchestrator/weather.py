"""
Integración con APIs meteorológicas externas
"""

import logging
from typing import Any

import requests

log = logging.getLogger("qeos.climate_api")

class WeatherAPIError(Exception):
    pass

class OpenWeatherMapClient:
    """Cliente para OpenWeatherMap API."""

    BASE_URL = "https://api.openweathermap.org/data/2.5"

    def __init__(self, api_key: str):
        self.api_key = api_key
        self.session = requests.Session()
        self.session.timeout = 10

    def get_current(self, lat: float, lon: float) -> dict[str, Any]:
        """Obtiene clima actual."""
        params = {
            "lat": lat,
            "lon": lon,
            "appid": self.api_key,
            "units": "metric",
        }
        resp = self.session.get(f"{self.BASE_URL}/weather", params=params)
        resp.raise_for_status()
        data = resp.json()

        return {
            "temp_c": data["main"]["temp"],
            "humidity": data["main"]["humidity"],
            "wind_mps": data["wind"].get("speed", 0),
            "pressure_hpa": data["main"]["pressure"],
            "description": data["weather"][0]["description"],
            "clouds_pct": data["clouds"]["all"] if "clouds" in data else 0,
            "timestamp": data["dt"],
        }

    def get_forecast(self, lat: float, lon: float, hours: int = 6) -> list[dict[str, Any]]:
        """Pronóstico por horas."""
        params = {
            "lat": lat,
            "lon": lon,
            "appid": self.api_key,
            "units": "metric",
        }
        resp = self.session.get(f"{self.BASE_URL}/forecast", params=params)
        resp.raise_for_status()
        data = resp.json()

        # OpenWeatherMap devuelve puntos cada 3h
        points_needed = max(1, hours // 3)
        forecast = []
        for item in data["list"][:points_needed]:
            forecast.append({
                "temp_c": item["main"]["temp"],
                "humidity": item["main"]["humidity"],
                "wind_mps": item["wind"].get("speed", 0),
                "clouds_pct": item["clouds"]["all"] if "clouds" in item else 0,
                "pop": item.get("pop", 0),  # prob. precipitación
                "timestamp": item["dt"],
            })

        return forecast

class NOAAClient:
    """Cliente para NOAA API (requiere token)."""

    BASE_URL = "https://api.weather.gov"

    def __init__(self, user_agent: str = "QuantumEnergyOS/1.0"):
        self.user_agent = user_agent
        self.session = requests.Session()
        self.session.headers.update({"User-Agent": user_agent})
        self.session.timeout = 10

    def get_alerts(self, lat: float, lon: float) -> list[dict[str, Any]]:
        """Obtiene alertas meteorológicas activas (EE.UU.)."""
        # NOAA solo cubre EE.UU., pero útil para Tijuana/San Diego
        try:
            # Obtener grid point
            url = f"{self.BASE_URL}/points/{lat},{lon}"
            resp = self.session.get(url)
            resp.raise_for_status()
            grid = resp.json()

            alerts_url = grid.get("properties", {}).get("alerts")
            if not alerts_url:
                return []

            resp = self.session.get(alerts_url)
            resp.raise_for_status()
            alerts_data = resp.json()

            return alerts_data.get("features", [])

        except Exception as e:
            log.error(f"NOAA get_alerts error: {e}")
            return []

def get_weather_client(api_key: str | None = None) -> OpenWeatherMapClient | None:
    """Factory: crea cliente meteorológico."""
    key = api_key or __import__("os").environ.get("OPENWEATHER_API_KEY")
    if not key:
        log.warning("OPENWEATHER_API_KEY no configurada — APIs meteorológicas deshabilitadas")
        return None
    return OpenWeatherMapClient(api_key=key)
