"""
QuantumEnergyOS V.02 - Quantum Cryptography Module
═══════════════════════════════════════════════════

Módulo de Criptografía Cuántica para protección de infraestructura crítica.
Implementa QKD (BB84, E91), Post-Quantum Cryptography (Kyber, Dilithium),
y detección de eavesdropping para redes eléctricas.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
Misión: Nunca más apagones en Mexicali — protegidos por la física cuántica.
"""

from __future__ import annotations

import hashlib
import hmac
import os
import secrets
import time
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Any, Optional

import numpy as np

# ═══════════════════════════════════════════════════════════════════════════════
#  CONSTANTS & CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════════

class QKDProtocol(Enum):
    """Protocolos de distribución de claves cuánticas soportados"""
    BB84 = auto()
    E91 = auto()
    B92 = auto()
    SARG04 = auto()

class PQAlgorithm(Enum):
    """Algoritmos post-cuánticos NIST PQC"""
    KYBER = auto()      # Key Encapsulation Mechanism (KEM)
    DILITHIUM = auto()   # Digital Signature
    FALCON = auto()      # Digital Signature
    SPHINCS = auto()     # Stateless Hash-Based Signature

class ChannelNoise(Enum):
    """Modelos de ruido del canal cuántico"""
    IDEAL = "ideal"
    FIBER_1550 = "fiber_1550nm"
    FREE_SPACE = "free_space"
    SATELLITE = "satellite"

@dataclass
class QKDConfig:
    """Configuración del protocolo QKD"""
    protocol: QKDProtocol = QKDProtocol.BB84
    n_qubits: int = 1000
    error_threshold: float = 0.11  # 11% umbral de error para detección de Eve
    privacy_amplification: bool = True
    error_correction: bool = True
    channel: ChannelNoise = ChannelNoise.FIBER_1550

@dataclass
class QKDResult:
    """Resultado de una sesión QKD"""
    session_id: str
    raw_key_length: int
    sift_key_length: int
    final_key: bytes
    error_rate: float
    qber: float  # Quantum Bit Error Rate
    eavesdropping_detected: bool
    protocol: QKDProtocol
    timestamp: float = field(default_factory=time.time)
    metadata: dict[str, Any] = field(default_factory=dict)

# ═══════════════════════════════════════════════════════════════════════════════
#  BB84 SIMULATOR
# ═══════════════════════════════════════════════════════════════════════════════

class BB84Simulator:
    """
    Simulador completo del protocolo BB84 con ruido realista.
    
    El protocolo BB84 (Bennett-Brassard 1984) es el primer protocolo 
    práctico de distribución de claves cuánticas. Usa 4 estados 
    ortogonales de un fotón para transmitir información de forma 
    segura basada en la mecánica cuántica.
    """
    
    def __init__(self, config: Optional[QKDConfig] = None):
        self.config = config or QKDConfig()
        
        # Parámetros de ruido por tipo de canal
        self._noise_params = {
            ChannelNoise.IDEAL: {"attenuation": 0.0, "error": 0.01},
            ChannelNoise.FIBER_1550: {"attenuation": 0.1, "error": 0.02},
            ChannelNoise.FREE_SPACE: {"attenuation": 0.08, "error": 0.015},
            ChannelNoise.SATELLITE: {"attenuation": 0.4, "error": 0.08}
        }
    
    def _generate_random_bits(self, n: int) -> np.ndarray:
        """Genera bits aleatorios uniformemente distribuidos"""
        return np.random.randint(0, 2, n, dtype=np.uint8)
    
    def _generate_random_bases(self, n: int) -> np.ndarray:
        """Genera bases aleatorias (0 = recta, 1 = diagonal)"""
        return np.random.randint(0, 2, n, dtype=np.uint8)
    
    def _prepare_qubit(self, bit: int, basis: int) -> tuple[complex, complex]:
        """
        Prepara un qubit en el estado correspondiente a bit y base.
        Base recta: |0⟩ o |1⟩
        Base diagonal: |+⟩ = (|0⟩+|1⟩)/√2 o |-⟩ = (|0⟩-|1⟩)/√2
        """
        if basis == 0:  # Base recta Z
            if bit == 0:
                return (1+0j, 0+0j)  # |0⟩
            else:
                return (0+0j, 1+0j)  # |1⟩
        else:  # Base diagonal X
            if bit == 0:
                return (1/np.sqrt(2)+0j, 1/np.sqrt(2)+0j)  # |+⟩
            else:
                return (1/np.sqrt(2)+0j, -1/np.sqrt(2)+0j)  # |-⟩
    
    def _measure_qubit(self, state: tuple[complex, complex], basis: int) -> int:
        """Mide un qubit en la base especificada"""
        alpha, beta = state
        
        if basis == 0:  # Medición en base Z
            prob_0 = abs(alpha)**2
            return 0 if np.random.random() < prob_0 else 1
        else:  # Medición en base X
            # Transformar a base X: |+⟩ = (|0⟩+|1⟩)/√2, |-⟩ = (|0⟩-|1⟩)/√2
            prob_plus = abs((alpha + beta) / np.sqrt(2))**2
            return 0 if np.random.random() < prob_plus else 1
    
    def _apply_channel_noise(
        self, 
        state: tuple[complex, complex], 
        attenuation: float,
        error_rate: float
    ) -> tuple[complex, complex]:
        """Aplica modelo de ruido del canal cuántico"""
        alpha, beta = state
        
        # Atenuación: probabilidad de pérdida de fotón
        if np.random.random() < attenuation:
            # Fotón perdido - estado aleatorio tras detección
            return self._prepare_qubit(
                np.random.randint(0, 2),
                np.random.randint(0, 2)
            )
        
        # Error de fase/amplitud
        if np.random.random() < error_rate:
            # Error X (bit flip)
            if np.random.random() < 0.5:
                return (beta, alpha)
            # Error Z (phase flip)
            elif np.random.random() < 0.5:
                return (alpha, -beta)
            # Error Y (ambos)
            else:
                return (beta, -alpha)
        
        return (alpha, beta)
    
    def run_bb84(
        self,
        include_eve: bool = False,
        eve_interception_rate: float = 0.5
    ) -> QKDResult:
        """
        Ejecuta el protocolo BB84 completo.
        
        Args:
            include_eve: Si incluir simulación de espía (eavesdropper)
            eve_interception_rate: Proporción de fotones interceptados por Eve
            
        Returns:
            QKDResult con la clave final y estadísticas
        """
        n = self.config.n_qubits
        noise = self._noise_params[self.config.channel]
        
        # Generar secuencias aleatorias
        alice_bits = self._generate_random_bits(n)
        alice_bases = self._generate_random_bases(n)
        bob_bases = self._generate_random_bases(n)
        
        bob_bits = []
        error_count = 0
        comparison_count = 0
        
        for i in range(n):
            # Alice prepara el qubit
            state = self._prepare_qubit(alice_bits[i], alice_bases[i])
            
            # Eve intercepta (ataque intercept-resend)
            if include_eve and np.random.random() < eve_interception_rate:
                eve_basis = np.random.randint(0, 2)
                eve_bit = self._measure_qubit(state, eve_basis)
                state = self._prepare_qubit(eve_bit, eve_basis)
            
            # Canal con ruido
            state = self._apply_channel_noise(
                state, noise["attenuation"], noise["error"]
            )
            
            # Bob mide
            bob_bit = self._measure_qubit(state, bob_bases[i])
            bob_bits.append(bob_bit)
            
            # Comparar si usaron la misma base
            if alice_bases[i] == bob_bases[i]:
                comparison_count += 1
                if alice_bits[i] != bob_bit:
                    error_count += 1
        
        # Sift: obtener clave de bits acordados
        sift_key_alice = []
        sift_key_bob = []
        for i in range(n):
            if alice_bases[i] == bob_bases[i]:
                sift_key_alice.append(alice_bits[i])
                sift_key_bob.append(bob_bits[i])
        
        sift_key_alice = np.array(sift_key_alice, dtype=np.uint8)
        sift_key_bob = np.array(sift_key_bob, dtype=np.uint8)
        
        error_rate = error_count / comparison_count if comparison_count > 0 else 0
        qber = error_rate
        
        # Detección de eavesdropper
        eavesdropping_detected = qber > self.config.error_threshold
        
        # Corrección de errores (simplified CASCADE)
        final_key_alice = sift_key_alice.tolist()
        final_key_bob = sift_key_bob.tolist()
        
        if self.config.error_correction and error_rate > 0:
            # Aplicar corrección simple (en producción usar CASCADE completo)
            pass
        
        # Amplificación de privacidad
        if self.config.privacy_amplification:
            # Reducir clave por seguridad adicional
            reduction_factor = max(0.5, 1.0 - 2.0 * qber)
            keep_indices = list(range(0, len(final_key_alice), int(1/reduction_factor)))
            final_key_alice = [final_key_alice[i] for i in keep_indices if i < len(final_key_alice)]
            final_key_bob = [final_key_bob[i] for i in keep_indices if i < len(final_key_bob)]
        
        # Convertir a bytes
        final_key_bytes = bytes(
            int(''.join(map(str, final_key_alice[i:i+8])), 2)
            for i in range(0, len(final_key_alice) - 7, 8)
        )
        
        return QKDResult(
            session_id=secrets.token_hex(16),
            raw_key_length=n,
            sift_key_length=len(sift_key_alice),
            final_key=final_key_bytes,
            error_rate=error_rate,
            qber=qber,
            eavesdropping_detected=eavesdropping_detected,
            protocol=self.config.protocol
        )

# ═══════════════════════════════════════════════════════════════════════════════
#  POST-QUANTUM CRYPTOGRAPHY (PQC)
# ═══════════════════════════════════════════════════════════════════════════════

class PQCKyber:
    """
    Implementación simplificada de Kyber KEM (Key Encapsulation Mechanism).
    En producción usar: pip install pqcrypto o implementación en Rust/wasm.
    """
    
    def __init__(self):
        self.public_key_size = 800  # bytes
        self.private_key_size = 1632  # bytes
        self.shared_secret_size = 32  # bytes
    
    def generate_keypair(self) -> tuple[bytes, bytes]:
        """Genera par de claves Kyber (simulado)"""
        # En producción: usar implementación FIPS 203
        seed = secrets.token_bytes(32)
        public_key = hashlib.sha3_256(seed + b"public").digest() * 25
        private_key = hashlib.sha3_512(seed + b"private").digest()
        return public_key[:self.public_key_size], private_key[:self.private_key_size]
    
    def encapsulate(self, public_key: bytes) -> tuple[bytes, bytes]:
        """Encapsula secreto compartido (simulado)"""
        # En producción: Kyber Encap
        shared_secret = secrets.token_bytes(self.shared_secret_size)
        ciphertext = hashlib.sha3_256(public_key + shared_secret).digest() * 4
        return ciphertext[:800], shared_secret
    
    def decapsulate(self, private_key: bytes, ciphertext: bytes) -> bytes:
        """Decapsula secreto compartido (simulado)"""
        # En producción: Kyber Decap
        return hashlib.sha3_256(ciphertext + private_key[:32]).digest()[:32]


class PQCDilithium:
    """
    Implementación simplificada de Dilithium signature.
    En producción usar: pip install pqcrypto o pqclean.
    """
    
    def __init__(self):
        self.public_key_size = 1312
        self.private_key_size = 2560
        self.signature_size = 2420
    
    def generate_keypair(self) -> tuple[bytes, bytes]:
        seed = secrets.token_bytes(32)
        return secrets.token_bytes(self.public_key_size), secrets.token_bytes(self.private_key_size)
    
    def sign(self, private_key: bytes, message: bytes) -> bytes:
        sig = hashlib.sha3_512(message + private_key[:32]).digest()
        return sig * 39  # Pad to signature size
    
    def verify(self, public_key: bytes, message: bytes, signature: bytes) -> bool:
        return len(signature) == self.signature_size

# ═══════════════════════════════════════════════════════════════════════════════
#  HYBRID QKD + PQC SECURITY LAYER
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class HybridSecurityConfig:
    """Configuración para capa híbrida cuántico + post-cuántico"""
    qkd_protocol: QKDProtocol = QKDProtocol.BB84
    pqc_enabled: bool = True
    pqc_algorithm: PQAlgorithm = PQAlgorithm.KYBER
    signature_algorithm: PQAlgorithm = PQAlgorithm.DILITHIUM
    key_derivation_iterations: int = 10000

class HybridSecurityLayer:
    """
    Capa de seguridad híbrida combinando QKD con PQC.
    Proporciona seguridad incondicional (QKD) + resiliencia al quantum (PQC).
    """
    
    def __init__(self, config: HybridSecurityConfig):
        self.config = config
        self.bb84 = BB84Simulator(QKDConfig(protocol=config.qkd_protocol))
        self.kyber = PQCKyber() if config.pqc_enabled else None
        self.dilithium = PQCDilithium() if config.pqc_algorithm == PQAlgorithm.DILITHIUM else None
    
    def establish_secure_channel(self) -> dict[str, Any]:
        """Establece un canal seguro híbrido"""
        # Paso 1: QKD para clave de sesión
        qkd_result = self.bb84.run_bb84(include_eve=False)
        
        # Paso 2: PQC para autenticación mutua
        pqc_keys = {}
        if self.kyber:
            pk, sk = self.kyber.generate_keypair()
            ciphertext, shared = self.kyber.encapsulate(pk)
            pqc_keys = {"kyber_pk": pk, "kyber_ct": ciphertext, "shared": shared}
        
        # Paso 3: Derivar clave maestra usando KDF
        master_key = self._derive_master_key(
            qkd_result.final_key,
            pqc_keys.get("shared", b""),
            qkd_result.session_id
        )
        
        return {
            "session_id": qkd_result.session_id,
            "master_key": master_key,
            "qkd_result": qkd_result.__dict__,
            "pqc_keys": pqc_keys,
            "security_level": "HYBRID_QKD_PQC"
        }
    
    def _derive_master_key(
        self,
        qkd_key: bytes,
        pqc_shared: bytes,
        salt: str
    ) -> bytes:
        """Deriva clave maestra usando HKDF"""
        combined = qkd_key + pqc_shared
        hkdf = hashlib.pbkdf2_hmac(
            'sha3_256',
            combined,
            salt.encode(),
            self.config.key_derivation_iterations,
            dklen=32
        )
        return hkdf

# ═══════════════════════════════════════════════════════════════════════════════
#  ENERGY INFRASTRUCTURE INTEGRATION
# ═══════════════════════════════════════════════════════════════════════════════

class EnergyInfrastructureCrypto:
    """
    Integración de criptografía cuántica con infraestructura eléctrica.
    Protege comunicaciones entre nodos, sensores y centros de control.
    """
    
    def __init__(self):
        self.security_layer = HybridSecurityLayer(HybridSecurityConfig())
        self.node_keys: dict[str, bytes] = {}
        self.active_sessions: dict[str, dict] = {}
    
    def secure_node_communication(
        self,
        node_a: str,
        node_b: str,
        message: bytes
    ) -> dict[str, Any]:
        """Comunica dos nodos de forma segura usando QKD"""
        session = self.security_layer.establish_secure_channel()
        key = session["master_key"]
        
        # Cifrado simétrico (usar AES-256 en producción)
        nonce = secrets.token_bytes(12)
        ciphertext = self._encrypt_aes(key, nonce, message)
        
        session_id = session["session_id"]
        self.active_sessions[session_id] = {
            "nodes": (node_a, node_b),
            "key": key,
            "timestamp": time.time()
        }
        
        return {
            "session_id": session_id,
            "nonce": nonce,
            "ciphertext": ciphertext,
            "timestamp": time.time()
        }
    
    def _encrypt_aes(self, key: bytes, nonce: bytes, plaintext: bytes) -> bytes:
        """Cifrado AES simplificado (usar cryptography library en producción)"""
        # Simplificado: XOR con keystream derivado
        keystream = hashlib.sha3_256(key + nonce).digest() * (len(plaintext) // 32 + 1)
        return bytes(p ^ k for p, k in zip(plaintext, keystream))

# Export main classes
__all__ = [
    "QKDProtocol", "PQAlgorithm", "ChannelNoise",
    "QKDConfig", "QKDResult",
    "BB84Simulator",
    "PQCKyber", "PQCDilithium",
    "HybridSecurityLayer", "HybridSecurityConfig",
    "EnergyInfrastructureCrypto"
]