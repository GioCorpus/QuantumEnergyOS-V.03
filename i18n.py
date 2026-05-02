"""
i18n.py — Internacionalización para QuantumEnergyOS API
Soporte multi-idioma para respuestas del servidor Flask.
"""

from flask import request, jsonify
from typing import Dict, Any

# Mensajes traducidos para el backend
MESSAGES: Dict[str, Dict[str, Dict[str, str]]] = {
    "en": {
        "errors": {
            "invalid_request": "Invalid request parameters",
            "server_error": "Internal server error",
            "not_found": "Resource not found",
            "unauthorized": "Unauthorized access",
            "forbidden": "Access forbidden",
        },
        "success": {
            "grid_balanced": "Grid balanced successfully",
            "prediction_complete": "Quantum prediction completed",
            "operation_success": "Operation completed successfully",
        },
        "health": {
            "healthy": "System healthy",
            "degraded": "System operating in degraded mode",
            "unhealthy": "System requires attention",
        },
        "solar": {
            "low_risk": "Low solar activity",
            "medium_risk": "Moderate solar activity",
            "high_risk": "High solar activity - grid at risk",
            "extreme_risk": "Extreme solar storm detected",
        },
        "climate": {
            "normal": "Climate conditions normal",
            "warning": "Weather alert - monitoring conditions",
            "critical": "Critical climate conditions detected",
        }
    },
    "es": {
        "errors": {
            "invalid_request": "Parámetros de solicitud inválidos",
            "server_error": "Error interno del servidor",
            "not_found": "Recurso no encontrado",
            "unauthorized": "Acceso no autorizado",
            "forbidden": "Acceso prohibido",
        },
        "success": {
            "grid_balanced": "Red balanceada exitosamente",
            "prediction_complete": "Predicción cuántica completada",
            "operation_success": "Operación completada exitosamente",
        },
        "health": {
            "healthy": "Sistema saludable",
            "degraded": "Sistema operando en modo degradado",
            "unhealthy": "El sistema requiere atención",
        },
        "solar": {
            "low_risk": "Baja actividad solar",
            "medium_risk": "Actividad solar moderada",
            "high_risk": "Alta actividad solar - red en riesgo",
            "extreme_risk": "Tormenta solar extrema detectada",
        },
        "climate": {
            "normal": "Condiciones climáticas normales",
            "warning": "Alerta meteorológica - monitoreando condiciones",
            "critical": "Condiciones climáticas críticas detectadas",
        }
    }
}

def get_locale() -> str:
    """Detecta el idioma preferido del cliente."""
    # Primero revisar Accept-Language header
    lang = request.accept_languages.best_match(['en', 'es', 'es-MX'])
    
    # Si hay cookie de preferencia, usarla
    if 'qeos_language' in request.cookies:
        saved_lang = request.cookies.get('qeos_language')
        if saved_lang in MESSAGES:
            return saved_lang
    
    return lang if lang else 'en'

def translate(key: str, locale: str = None, **kwargs) -> str:
    """Traduce una clave al idioma especificado."""
    if locale is None:
        locale = get_locale()
    
    # Asegurar que el locale existe
    if locale not in MESSAGES:
        locale = 'en'
    
    # Navegar por el árbol de claves (ej: "errors.invalid_request")
    keys = key.split('.')
    msg = MESSAGES[locale]
    
    for k in keys:
        if isinstance(msg, dict) and k in msg:
            msg = msg[k]
        else:
            return key
    
    # Reemplazar placeholders
    if kwargs and isinstance(msg, str):
        for k, v in kwargs.items():
            msg = msg.replace(f'{{{k}}}', str(v))
    
    return msg

def localized_response(data: Dict[str, Any], locale: str = None) -> Dict[str, Any]:
    """Crea una respuesta JSON con mensajes localizados."""
    return {
        **data,
        "locale": locale or get_locale()
    }