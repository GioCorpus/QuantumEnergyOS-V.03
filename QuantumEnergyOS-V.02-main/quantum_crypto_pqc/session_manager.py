"""
QuantumEnergyOS V.02 - Session Manager
═════════════════════════════════════════

Gestión de sesiones seguras con forward secrecy para nodos de infraestructura.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
"""

from __future__ import annotations

import secrets
import time
from dataclasses import dataclass, field
from typing import Any, Optional

from .models import SecurityLevel, PQCSession
from .hybrid import HybridChannelManager, HybridMode


class SessionManager:
    """
    Gestor de sesiones seguras para la red eléctrica.
    
    Funcionalidades:
    - Rotación automática de claves con forward secrecy
    - Gestión de sesiones entre nodos (subestaciones, centro de control)
    - Cleanup de sesiones expiradas
    - Auditoría de conexiones
    """
    
    DEFAULT_EXPIRY = {
        SecurityLevel.STANDARD: 24 * 3600,      # 24 horas
        SecurityLevel.HIGH: 6 * 3600,           # 6 horas
        SecurityLevel.CRITICAL: 3600,            # 1 hora
    }
    
    def __init__(self, security_level: SecurityLevel = SecurityLevel.STANDARD):
        self.security_level = security_level
        self.hybrid_manager = HybridChannelManager(self._get_hybrid_mode())
        self.sessions: dict[str, PQCSession] = {}
        self._cleanup_interval = 300  # 5 minutos
        self._last_cleanup = time.time()
    
    def _get_hybrid_mode(self) -> HybridMode:
        """Mapea SecurityLevel a HybridMode."""
        mapping = {
            SecurityLevel.STANDARD: HybridMode.PQC_ONLY,
            SecurityLevel.HIGH: HybridMode.PQC_ECDH,
            SecurityLevel.CRITICAL: HybridMode.TRIPLE_HYBRID,
        }
        return mapping.get(self.security_level, HybridMode.PQC_ECDH)
    
    def create_session(
        self,
        node_a: str,
        node_b: str,
        custom_expiry: Optional[int] = None
    ) -> PQCSession:
        """
        Crea una nueva sesión segura entre dos nodos.
        
        Args:
            node_a: ID del primer nodo
            node_b: ID del segundo nodo
            custom_expiry: Tiempo de expiración en segundos (opcional)
            
        Returns:
            PQCSession creada
        """
        self._cleanup_expired()
        
        result = self.hybrid_manager.establish_channel()
        expiry = custom_expiry or self.DEFAULT_EXPIRY.get(self.security_level, 3600)
        
        session = PQCSession(
            session_id=result.channel_id,
            node_a=node_a,
            node_b=node_b,
            master_key=result.master_key,
            security_level=self.security_level,
            pqc_data={
                "mode": result.mode.name,
                "kyber_pk": result.kyber_keypair[0].hex() if result.kyber_keypair else None,
            },
            classical_shared=result.ecdh_shared,
            qkd_key=result.qkd_shared,
            expires_at=time.time() + expiry,
        )
        
        self.sessions[session.session_id] = session
        
        return session
    
    def get_session(self, session_id: str) -> Optional[PQCSession]:
        """
        Obtiene una sesión por ID.
        
        Args:
            session_id: ID de sesión
            
        Returns:
            PQCSession si existe y está activa, None en caso contrario
        """
        self._cleanup_expired()
        
        session = self.sessions.get(session_id)
        if session and session.is_valid():
            return session
        
        return None
    
    def validate_session(self, session_id: str) -> bool:
        """
        Valida una sesión (existe y no expiró).
        
        Args:
            session_id: ID de sesión a validar
            
        Returns:
            True si la sesión es válida
        """
        session = self.get_session(session_id)
        return session is not None
    
    def close_session(self, session_id: str) -> bool:
        """
        Cierra una sesión activa.
        
        Args:
            session_id: ID de sesión a cerrar
            
        Returns:
            True si la sesión fue cerrada
        """
        session = self.sessions.get(session_id)
        if session:
            session.active = False
            del self.sessions[session_id]
            return True
        return False
    
    def _cleanup_expired(self):
        """Remueve sesiones expiradas."""
        now = time.time()
        if now - self._last_cleanup < self._cleanup_interval:
            return
        
        expired = [
            sid for sid, session in self.sessions.items()
            if session.is_expired() or not session.active
        ]
        
        for sid in expired:
            del self.sessions[sid]
        
        self._last_cleanup = now
    
    def get_active_sessions(self) -> list[dict[str, Any]]:
        """Obtiene lista de sesiones activas (metadata solo)."""
        self._cleanup_expired()
        
        return [
            {
                "session_id": s.session_id,
                "nodes": (s.node_a, s.node_b),
                "security_level": s.security_level.name,
                "created_at": s.created_at,
                "expires_at": s.expires_at,
                "age_seconds": int(time.time() - s.created_at),
            }
            for s in self.sessions.values()
            if s.active
        ]
    
    def session_exists(self, session_id: str) -> bool:
        """Verifica si una sesión existe (sin validar expiración)."""
        return session_id in self.sessions
    
    def rotate_session(self, session_id: str) -> Optional[PQCSession]:
        """
        Rota una sesión existente (forward secrecy).
        
        Args:
            session_id: ID de sesión a rotar
            
        Returns:
            Nueva sesión con claves rotadas
        """
        old_session = self.sessions.get(session_id)
        if not old_session:
            return None
        
        new_session = self.create_session(
            old_session.node_a,
            old_session.node_b
        )
        
        # Mantener la antigua como backup por un tiempo corto
        old_session.expires_at = time.time() + 60  # 1 minuto de gracia
        
        return new_session
    
    def get_stats(self) -> dict[str, Any]:
        """Obtiene estadísticas del gestor de sesiones."""
        self._cleanup_expired()
        
        return {
            "total_sessions": len(self.sessions),
            "active_sessions": len([s for s in self.sessions.values() if s.active]),
            "security_level": self.security_level.name,
            "hybrid_mode": self.hybrid_manager.mode.name,
        }