"""
QuantumEnergyOS V.02 - PQC Models
═══════════════════════════════════

Modelos de datos para criptografía post-cuántica.
"""

from __future__ import annotations

import json
import secrets
import time
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Any, Optional


class SecurityLevel(Enum):
    """Niveles de seguridad configurables para la red eléctrica."""
    STANDARD = auto()    # Kyber + Dilithium, rotación cada 24h
    HIGH = auto()        # Kyber + Dilithium, rotación cada 6h, ECDH adicional
    CRITICAL = auto()    # Híbrido: PQC + X25519 + QKD, rotación cada hora


class PQCSignature:
    """Firma PQC con metadata de auditoría."""
    algorithm: str
    signature: bytes
    public_key: bytes
    message: bytes
    timestamp: float
    node_id: Optional[str] = None

    def __init__(
        self,
        algorithm: str,
        signature: bytes,
        public_key: bytes,
        message: bytes,
        node_id: Optional[str] = None
    ):
        self.algorithm = algorithm
        self.signature = signature
        self.public_key = public_key
        self.message = message
        self.node_id = node_id
        self.timestamp = time.time()


@dataclass
class PQCKeyPair:
    """Par de claves PQC con metadata."""
    public_key: bytes
    private_key: bytes
    algorithm: str
    created_at: float = field(default_factory=time.time)
    expires_at: Optional[float] = None
    key_id: str = field(default_factory=lambda: secrets.token_hex(8))
    
    def is_expired(self) -> bool:
        if self.expires_at is None:
            return False
        return time.time() > self.expires_at


@dataclass
class PQCSession:
    """Sesión segura entre nodos de la red eléctrica."""
    session_id: str
    node_a: str
    node_b: str
    master_key: bytes
    created_at: float = field(default_factory=time.time)
    expires_at: Optional[float] = None
    security_level: SecurityLevel = SecurityLevel.STANDARD
    pqc_data: dict[str, Any] = field(default_factory=dict)
    classical_shared: Optional[bytes] = None
    qkd_key: Optional[bytes] = None
    active: bool = True

    def is_valid(self) -> bool:
        return self.active and not self.is_expired()

    def is_expired(self) -> bool:
        if self.expires_at is None:
            return False
        return time.time() > self.expires_at


@dataclass
class BlackoutAlert:
    """Alerta de apagón firmada criptográficamente."""
    alert_id: str
    location: str
    risk_level: str
    predicted_time: Optional[float]
    grid_load_percent: float
    temperature: float
    signature: Optional[bytes] = None
    timestamp: float = field(default_factory=time.time)
    node_origin: Optional[str] = None

    def to_bytes(self) -> bytes:
        """Serializa la alerta para firmar/verificar."""
        data = f"{self.alert_id}|{self.location}|{self.risk_level}|{self.grid_load_percent}|{self.temperature}"
        return data.encode()


@dataclass
class EnergyCommand:
    """Comando de control de infraestructura eléctrica firmado."""
    command_id: str
    command_type: str  # BALANCE_LOAD, SHUTDOWN_SUBESTATION, RESTART_GRID, etc.
    target_node: str
    parameters: dict[str, Any]
    signature: Optional[bytes] = None
    timestamp: float = field(default_factory=time.time)
    expires_at: Optional[float] = None

    def to_bytes(self) -> bytes:
        """Serializa el comando para firmar/verificar."""
        payload = {
            "command_id": self.command_id,
            "command_type": self.command_type,
            "target_node": self.target_node,
            "parameters": self.parameters,
        }
        return json.dumps(payload, sort_keys=True).encode()