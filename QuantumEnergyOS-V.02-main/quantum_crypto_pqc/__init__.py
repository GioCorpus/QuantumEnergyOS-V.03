"""
QuantumEnergyOS V.02 - Quantum Crypto PQC Module
══════════════════════════════════════════════

Módulo de Criptografía Post-Cuántica (PQC) para protección de infraestructura crítica.
Implementa algoritmos NIST PQC: Kyber, Dilithium, Falcon, SPHINCS+.
Soporte híbrido: PQC + ECDH/X25519 + QKD simulada.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
Misión: Nunca más apagones en Mexicali — protegidos por criptografía post-cuántica.
"""

from __future__ import annotations

from .models import (
    SecurityLevel,
    PQCSignature,
    PQCKeyPair,
    PQCSession,
    BlackoutAlert,
    EnergyCommand,
)
from .kyber import KyberKEM
from .dilithium import DilithiumSignature
from .falcon import FalconSignature
from .spincs import SPHINCSSignature
from .hybrid import HybridChannelManager
from .energy_crypto import EnergyInfrastructurePQC
from .session_manager import SessionManager

__version__ = "2.0.0"
__all__ = [
    "SecurityLevel",
    "PQCSignature",
    "PQCKeyPair",
    "PQCSession",
    "BlackoutAlert",
    "EnergyCommand",
    "KyberKEM",
    "DilithiumSignature",
    "FalconSignature",
    "SPHINCSSignature",
    "HybridChannelManager",
    "EnergyInfrastructurePQC",
    "SessionManager",
]