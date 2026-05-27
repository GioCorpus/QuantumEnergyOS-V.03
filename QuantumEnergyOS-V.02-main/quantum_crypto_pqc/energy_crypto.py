"""
QuantumEnergyOS V.02 - Energy Infrastructure PQC Integration
═══════════════════════════════════════════════════════════════

Integración específica de criptografía PQC con la red eléctrica.
Protección de comandos de balanceo, alertas de apagón y optimizaciones.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
"""

from __future__ import annotations

import hashlib
import logging
import secrets
import time
from typing import Any, Optional

from .models import (
    SecurityLevel,
    PQCSession,
    BlackoutAlert,
    EnergyCommand,
    PQCSignature,
)
from .hybrid import HybridChannelManager, HybridMode
from .utils import secure_encrypt, secure_decrypt, secure_hash, derive_key_hkdf, RateLimiter

log = logging.getLogger("qeos.pqc.energy")


class EnergyInfrastructurePQC:
    """
    Integración de PQC con infraestructura eléctrica.
    
    Protege:
    - Comandos de balanceo de carga entre subestaciones
    - Alertas de apagón firmadas
    - Comunicaciones SCADA-like entre nodos
    - Optimizaciones del Climate Orchestrator
    """
    
    def __init__(self, security_level: SecurityLevel = SecurityLevel.STANDARD):
        self.security_level = security_level
        self.hybrid_manager = HybridChannelManager(self._get_hybrid_mode(security_level))
        self.session_manager: Optional[Any] = None
        self.rate_limiter = RateLimiter(max_requests=200, window_seconds=60)
        self._node_keys: dict[str, tuple[bytes, bytes]] = {}
        self.active_sessions: dict[str, PQCSession] = {}
    
    def _get_hybrid_mode(self, level: SecurityLevel) -> HybridMode:
        """Mapea SecurityLevel a HybridMode."""
        mapping = {
            SecurityLevel.STANDARD: HybridMode.PQC_ONLY,
            SecurityLevel.HIGH: HybridMode.PQC_ECDH,
            SecurityLevel.CRITICAL: HybridMode.TRIPLE_HYBRID,
        }
        return mapping.get(level, HybridMode.PQC_ECDH)
    
    def establish_secure_channel(
        self,
        node_a: str,
        node_b: str,
        key_rotation_hours: Optional[int] = None
    ) -> PQCSession:
        """
        Establece canal seguro entre dos nodos eléctricos.
        
        Args:
            node_a: ID del primer nodo (ej: SUBESTACION_MEXICALI)
            node_b: ID del segundo nodo (ej: CENTRO_CONTROL)
            key_rotation_hours: Horas hasta rotación de claves
            
        Returns:
            PQCSession con el canal seguro establecido
        """
        key_rotation_hours = key_rotation_hours or {
            SecurityLevel.STANDARD: 24,
            SecurityLevel.HIGH: 6,
            SecurityLevel.CRITICAL: 1,
        }.get(self.security_level, 6)
        
        result = self.hybrid_manager.establish_channel()
        
        session = PQCSession(
            session_id=result.channel_id,
            node_a=node_a,
            node_b=node_b,
            master_key=result.master_key,
            security_level=self.security_level,
            pqc_data={
                "mode": result.mode.name,
                "algorithm": "Kyber+Dilithium",
            },
            classical_shared=result.ecdh_shared,
            qkd_key=result.qkd_shared,
            expires_at=time.time() + (key_rotation_hours * 3600),
        )
        
        self.active_sessions[session.session_id] = session
        
        log.info(
            f"Canal seguro establecido: {node_a} <-> {node_b} "
            f"| Seguridad: {self.security_level.name} | ID: {session.session_id[:8]}..."
        )
        
        return session
    
    def sign_grid_command(self, command: EnergyCommand, private_key: bytes) -> EnergyCommand:
        """
        Firma un comando de infraestructura eléctrica.
        
        Args:
            command: Comando a firmar
            private_key: Clave privada de firma
            
        Returns:
            Comando con firma adjunta
        """
        from . import DilithiumSignature
        
        message_bytes = command.to_bytes()
        signature = self.hybrid_manager.sign_message(
            message_bytes,
            private_key,
            algorithm="dilithium"
        )
        command.signature = signature
        
        log.info(
            f"Comando firmado: {command.command_type} | "
            f"Nodo: {command.target_node} | Firma: {len(signature)} bytes"
        )
        
        return command
    
    def verify_grid_command(
        self,
        command: EnergyCommand,
        public_key: bytes
    ) -> bool:
        """
        Verifica la firma de un comando de infraestructura.
        
        Args:
            command: Comando con firma
            public_key: Clave pública para verificación
            
        Returns:
            True si la firma es válida
        """
        if not command.signature:
            return False
        
        message_bytes = command.to_bytes()
        valid = self.hybrid_manager.verify_message(
            message_bytes,
            command.signature,
            public_key,
            algorithm="dilithium"
        )
        
        if valid:
            log.info(f"Comando verificado: {command.command_type} válido")
        else:
            log.warning(f"Comando rechazado: {command.command_type} - firma inválida")
        
        return valid
    
    def sign_blackout_alert(self, alert: BlackoutAlert, private_key: bytes) -> BlackoutAlert:
        """
        Firma una alerta de apagón.
        
        Args:
            alert: Alerta a firmar
            private_key: Clave privada de firma
            
        Returns:
            Alerta con firma adjunta
        """
        from . import DilithiumSignature
        
        message_bytes = alert.to_bytes()
        signature = self.hybrid_manager.sign_message(
            message_bytes,
            private_key,
            algorithm="dilithium"
        )
        alert.signature = signature
        
        log.warning(
            f"ALERTA DE APAGÓN FIRMADA | {alert.location} | "
            f"Riesgo: {alert.risk_level} | Firma: {len(signature)} bytes"
        )
        
        return alert
    
    def verify_blackout_alert(
        self,
        alert: BlackoutAlert,
        public_key: bytes
    ) -> bool:
        """
        Verifica la firma de una alerta de apagón.
        
        Args:
            alert: Alerta con firma
            public_key: Clave pública para verificación
            
        Returns:
            True si la alerta es auténtica
        """
        if not alert.signature:
            return False
        
        message_bytes = alert.to_bytes()
        valid = self.hybrid_manager.verify_message(
            message_bytes,
            alert.signature,
            public_key,
            algorithm="dilithium"
        )
        
        if valid:
            log.info(f"Alerta verificada: {alert.location} - autenticada")
        else:
            log.error(f"ALERTA RECHAZADA: {alert.location} - firma inválida, posible ataque")
        
        return valid
    
    def secure_transmit(
        self,
        session_id: str,
        message: bytes
    ) -> tuple[bytes, bytes]:
        """
        Transmite mensaje de forma segura usando una sesión.
        
        Args:
            session_id: ID de sesión activa
            message: Mensaje a transmitir
            
        Returns:
            Tuple de (nonce, ciphertext)
        """
        session = self.active_sessions.get(session_id)
        if not session or not session.is_valid():
            raise ValueError(f"Sesión inválida o expirada: {session_id}")
        
        nonce, ciphertext = secure_encrypt(message, session.master_key)
        
        log.debug(f"Mensaje encriptado en sesión {session_id[:8]}...")
        return nonce, ciphertext
    
    def secure_receive(
        self,
        session_id: str,
        ciphertext: bytes,
        nonce: bytes
    ) -> bytes:
        """
        Recibe mensaje de forma segura usando una sesión.
        
        Args:
            session_id: ID de sesión activa
            ciphertext: Mensaje encriptado
            nonce: Nonce usado en encriptación
            
        Returns:
            Mensaje desencriptado
        """
        session = self.active_sessions.get(session_id)
        if not session or not session.is_valid():
            raise ValueError(f"Sesión inválida o expirada: {session_id}")
        
        plaintext = secure_decrypt(ciphertext, session.master_key, nonce)
        
        log.debug(f"Mensaje desencriptado en sesión {session_id[:8]}...")
        return plaintext
    
    def rotate_keys(self, session_id: str) -> Optional[PQCSession]:
        """
        Rota las claves de una sesión para forward secrecy.
        
        Args:
            session_id: ID de sesión a rotar
            
        Returns:
            Nueva sesión con claves rotadas
        """
        old_session = self.active_sessions.get(session_id)
        if not old_session:
            return None
        
        new_session = self.establish_secure_channel(
            old_session.node_a,
            old_session.node_b
        )
        
        log.info(f"Claves rotadas para sesión {session_id[:8]}...")
        return new_session
    
    def get_security_status(self) -> dict[str, Any]:
        """Obtiene estado del sistema de seguridad PQC."""
        return {
            "security_level": self.security_level.name,
            "active_sessions": len(self.active_sessions),
            "registered_nodes": len(self._node_keys),
            "rate_limit_remaining": self.rate_limiter.get_remaining("system"),
            "algorithms": {
                "kem": "Kyber",
                "signature": "Dilithium",
                "hybrid_mode": self.hybrid_manager.mode.name,
            },
        }