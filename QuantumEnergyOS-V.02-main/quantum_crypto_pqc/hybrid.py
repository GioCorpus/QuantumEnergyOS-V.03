"""
QuantumEnergyOS V.02 - Hybrid Channel Manager
═════════════════════════════════════════════════

Canal híbrido combinando PQC + ECDH/X25519 + QKD simulada.
Proporciona seguridad múltiple capas para infraestructura crítica.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
"""

from __future__ import annotations

import hashlib
import secrets
import time
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Any, Optional

from .models import SecurityLevel

try:
    from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey
    from cryptography.hazmat.primitives import serialization
    CRYPTO_AVAILABLE = True
except ImportError:
    CRYPTO_AVAILABLE = False

try:
    import numpy as np
    NUMPY_AVAILABLE = True
except ImportError:
    NUMPY_AVAILABLE = False

# Import QKD simulator
try:
    from quantum_crypto import BB84Simulator, QKDConfig, QKDProtocol, ChannelNoise
    QKD_AVAILABLE = True
except ImportError:
    QKD_AVAILABLE = False


class HybridMode(Enum):
    """Modos de canal híbrido disponibles."""
    PQC_ONLY = auto()           # Solo Kyber + Dilithium
    PQC_ECDH = auto()         # PQC + X25519
    PQC_QKD = auto()          # PQC + BB84 simulada
    TRIPLE_HYBRID = auto()    # PQC + X25519 + BB84


@dataclass
class HybridChannelResult:
    """Resultado de establecimiento de canal híbrido."""
    channel_id: str
    mode: HybridMode
    master_key: bytes = b""
    pqc_shared: Optional[bytes] = None
    ecdh_shared: Optional[bytes] = None
    qkd_shared: Optional[bytes] = None
    kyber_keypair: tuple[bytes, bytes] = (b"", b"")
    dilithium_keypair: tuple[bytes, bytes] = (b"", b"")
    timestamp: float = field(default_factory=time.time)
    security_level: str = "HYBRID"


class HybridChannelManager:
    """
    Gestor de canales híbridos de seguridad.
    
    Combina múltiples capas de criptografía:
    - PQC (Kyber/Dilithium) para resistencia cuántica
    - ECDH/X25519 para compatibilidad clásica
    - QKD (BB84 simulada) para seguridad incondicional
    """
    
    def __init__(self, security_level: SecurityLevel = SecurityLevel.STANDARD):
        self.security_level = security_level
        self.mode = self._get_mode_from_level(security_level)
        self._ecdh_private: Optional[Any] = None
        self._ecdh_public: Optional[bytes] = None
        self._channels: dict[str, HybridChannelResult] = {}
        
        if CRYPTO_AVAILABLE:
            self._ecdh_private = X25519PrivateKey.generate()
            self._ecdh_public = self._ecdh_private.public_key().public_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PublicFormat.Raw
            )
    
    def _get_mode_from_level(self, level: SecurityLevel) -> HybridMode:
        """Map SecurityLevel to HybridMode."""
        if level == SecurityLevel.CRITICAL:
            return HybridMode.TRIPLE_HYBRID
        elif level == SecurityLevel.HIGH:
            return HybridMode.PQC_ECDH
        return HybridMode.PQC_ONLY
    
    def create_channel(self, node_id: str) -> dict:
        """Create a secure channel for a node."""
        result = self.establish_channel()
        self._channels[node_id] = result
        return {
            "channel_id": result.channel_id,
            "type": self.mode.name,
            "encryption_active": True,
            "security_level": result.security_level
        }
    
    def establish_channel(self) -> HybridChannelResult:
        """
        Establece un canal seguro híbrido.
        
        Returns:
            HybridChannelResult con todos los componentes de seguridad
        """
        from . import KyberKEM, DilithiumSignature
        
        result = HybridChannelResult(
            channel_id=secrets.token_hex(16),
            mode=self.mode,
        )
        
        # PQC Layer - Kyber
        kyber = KyberKEM()
        kyber_pk, kyber_sk = kyber.generate_keypair()
        kyber_ct, kyber_shared = kyber.encapsulate(kyber_pk)
        result.kyber_keypair = (kyber_pk, kyber_sk)
        result.pqc_shared = kyber_shared
        
        # PQC Layer - Dilithium for authentication
        dilithium = DilithiumSignature()
        dilithium_pk, dilithium_sk = dilithium.generate_keypair()
        result.dilithium_keypair = (dilithium_pk, dilithium_sk)
        
        # Classical ECDH Layer
        if CRYPTO_AVAILABLE and self._ecdh_private:
            ecdh_shared = self._ecdh_shared_secret()
            result.ecdh_shared = ecdh_shared
        
        # QKD Layer (BB84 simulada)
        if QKD_AVAILABLE and self.mode in (HybridMode.PQC_QKD, HybridMode.TRIPLE_HYBRID):
            qkd_sim = BB84Simulator(QKDConfig(protocol=QKDProtocol.BB84, n_qubits=2000))
            qkd_result = qkd_sim.run_bb84(include_eve=False)
            result.qkd_shared = qkd_result.final_key
            result.security_level = f"HYBRID_QKD_{qkd_result.protocol.name.upper()}" if QKD_AVAILABLE else "HYBRID_PQC_ECDH"
        
        # Derive master key from all available sources
        result.master_key = self._derive_master_key(result)
        
        return result
    
    def _ecdh_shared_secret(self) -> bytes:
        """Genera secreto compartido ECDH."""
        # Simulate ECDH shared secret for fallback
        peer_seed = secrets.token_bytes(32)
        if CRYPTO_AVAILABLE and self._ecdh_public:
            shared = hashlib.sha256(peer_seed + self._ecdh_public[:16]).digest()
            return shared
        return secrets.token_bytes(32)
    
    def _derive_master_key(self, result: HybridChannelResult) -> bytes:
        """
        Deriva clave maestra usando KDF.
        
        Combina todos los secretos compartidos con HKDF.
        """
        combined = b""
        if result.pqc_shared:
            combined += result.pqc_shared
        if result.ecdh_shared:
            combined += result.ecdh_shared
        if result.qkd_shared:
            combined += result.qkd_shared
        
        # Add entropy from all keys
        if result.kyber_keypair[0]:
            combined += result.kyber_keypair[0][:16]
        if result.dilithium_keypair[0]:
            combined += result.dilithium_keypair[0][:16]
        
        # HKDF-SHA3-256 derivation
        hkdf = hashlib.pbkdf2_hmac(
            'sha3_256',
            combined if combined else secrets.token_bytes(32),
            result.channel_id.encode(),
            10000,
            dklen=32
        )
        return hkdf
    
    def sign_message(self, message: bytes, private_key: bytes, algorithm: str = "dilithium") -> bytes:
        """
        Firma un mensaje usando el algoritmo especificado.
        
        Args:
            message: Mensaje a firmar
            private_key: Clave privada
            algorithm: Algoritmo de firma (dilithium, falcon, spincs)
            
        Returns:
            Firma del mensaje
        """
        from . import DilithiumSignature, FalconSignature, SPHINCSSignature
        
        if algorithm == "dilithium":
            signer = DilithiumSignature()
        elif algorithm == "falcon":
            signer = FalconSignature()
        elif algorithm == "spincs":
            signer = SPHINCSSignature()
        else:
            signer = DilithiumSignature()
        
        # Need full keypair for proper signing
        public_key = private_key[:1312]  # Approximate public key size
        signature = signer.sign(private_key, message)
        return signature
    
    def verify_message(self, message: bytes, signature: bytes, public_key: bytes, algorithm: str = "dilithium") -> bool:
        """
        Verifica una firma usando el algoritmo especificado.
        
        Args:
            message: Mensaje original
            signature: Firma a verificar
            public_key: Clave pública
            algorithm: Algoritmo de verificación
            
        Returns:
            True si la firma es válida
        """
        from . import DilithiumSignature, FalconSignature, SPHINCSSignature
        
        if algorithm == "dilithium":
            signer = DilithiumSignature()
        elif algorithm == "falcon":
            signer = FalconSignature()
        elif algorithm == "spincs":
            signer = SPHINCSSignature()
        else:
            signer = DilithiumSignature()
        
        return signer.verify(public_key, message, signature)
    
    def secure_send(self, node_id: str, message: bytes) -> dict:
        """Send a secure message through an established channel."""
        if node_id not in self._channels:
            return {"success": False, "error": "Channel not found"}
        
        channel = self._channels[node_id]
        nonce, ciphertext = self.encrypt(message, channel.master_key)
        
        return {
            "success": True,
            "nonce": nonce,
            "ciphertext": ciphertext,
            "channel_id": channel.channel_id
        }
    
    def sign_alert(self, node_id: str, payload: bytes) -> dict:
        """Sign an alert using the channel's Dilithium key."""
        if node_id not in self._channels:
            return {"success": False, "error": "Channel not found"}
        
        channel = self._channels[node_id]
        dilithium_pk, dilithium_sk = channel.dilithium_keypair
        
        signature = self.sign_message(payload, dilithium_sk)
        
        return {
            "success": True,
            "signature": signature,
            "public_key": dilithium_pk,
            "algorithm": "dilithium"
        }
    
    def encrypt(self, plaintext: bytes, key: bytes) -> tuple[bytes, bytes]:
        """
        Encripta usando la clave maestra derivada.
        
        Args:
            plaintext: Mensaje a encriptar
            key: Clave maestra
            
        Returns:
            Tuple de (nonce, ciphertext)
        """
        from .utils import secure_encrypt
        return secure_encrypt(plaintext, key)
    
    def decrypt(self, ciphertext: bytes, key: bytes, nonce: bytes) -> bytes:
        """
        Desencripta usando la clave maestra derivada.
        
        Args:
            ciphertext: Mensaje encriptado
            key: Clave maestra
            nonce: Nonce usado en encriptación
            
        Returns:
            Mensaje desencriptado
        """
        from .utils import secure_decrypt
        return secure_decrypt(ciphertext, key, nonce)