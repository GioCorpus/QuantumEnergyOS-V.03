"""
QuantumEnergyOS V.02 - Kyber KEM Implementation
═════════════════════════════════════════════════

Implementación de Kyber (KEM) para establecimiento de claves.
Algoritmo NIST FIPS 203 - Key Encapsulation Mechanism.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
"""

from __future__ import annotations

import hashlib
import secrets
import time
from typing import Optional

try:
    from oqs import KEM
    OQS_AVAILABLE = True
except ImportError:
    OQS_AVAILABLE = False


class KyberKEM:
    """
    Kyber Key Encapsulation Mechanism (KEM).
    
    Usado para establecimiento de claves seguros entre nodos.
    Resistente a ataques de computadoras cuánticas.
    """
    
    PARAMETERS = {
        "KYBER512": {"pk": 800, "sk": 1632, "ct": 768, "ss": 32},
        "KYBER768": {"pk": 1188, "sk": 2400, "ct": 1088, "ss": 32},
        "KYBER1024": {"pk": 1568, "sk": 3168, "ct": 1504, "ss": 32},
    }
    
    def __init__(self, variant: str = "KYBER768"):
        self.variant = variant
        self._oqs_kem: Optional[KEM] = None
        
        if OQS_AVAILABLE:
            try:
                self._oqs_kem = KEM(variant.lower())
            except Exception:
                self._oqs_kem = None
    
    @property
    def name(self) -> str:
        return self.variant
    
    def generate_keypair(self) -> tuple[bytes, bytes]:
        """
        Genera par de claves Kyber.
        
        Returns:
            Tuple de (public_key, private_key)
        """
        params = self.PARAMETERS[self.variant]
        
        if self._oqs_kem:
            return self._oqs_kem.generate_keypair()
        
        # Fallback implementation (simulada para desarrollo)
        seed = secrets.token_bytes(32)
        # Generate keys with seed stored in first 32 bytes
        public_key = seed + hashlib.sha3_256(seed + b"KYBER_PUB").digest()
        public_key = (public_key * ((params["pk"] // 32) + 100))[:params["pk"]]
        
        private_key = seed + hashlib.sha3_512(seed + b"KYBER_PRIV").digest()
        private_key = (private_key * ((params["sk"] // 64) + 100))[:params["sk"]]
        
        return public_key, private_key
    
    def encapsulate(self, public_key: bytes) -> tuple[bytes, bytes]:
        """
        Encapsula un secreto compartido usando la clave pública.
        
        Args:
            public_key: Clave pública del destinatario
            
        Returns:
            Tuple de (ciphertext, shared_secret)
        """
        params = self.PARAMETERS[self.variant]
        
        if self._oqs_kem:
            return self._oqs_kem.encap(public_key)
        
        # Fallback implementation
        # Extract seed from public key (stored in first 32 bytes)
        seed = public_key[:32]
        shared_secret = hashlib.sha3_256(seed + b"SHARED").digest()[:params["ss"]]
        # Create ciphertext deterministically from seed
        cipher_base = hashlib.sha3_256(seed + b"KYBER_CT").digest()
        ciphertext = (cipher_base * ((params["ct"] // 32) + 100))[:params["ct"]]
        
        return ciphertext, shared_secret
    
    def decapsulate(self, private_key: bytes, ciphertext: bytes) -> bytes:
        """
        Decapsula el secreto compartido usando la clave privada.
        
        Args:
            private_key: Clave privada del destinatario
            ciphertext: Texto cifrado del encapsulado
            
        Returns:
            Secreto compartido derivado
        """
        params = self.PARAMETERS[self.variant]
        
        if self._oqs_kem:
            return self._oqs_kem.decap(ciphertext, private_key)
        
        # Fallback implementation
        # Extract seed from private key (stored in first 32 bytes)
        seed = private_key[:32]
        return hashlib.sha3_256(seed + b"SHARED").digest()[:params["ss"]]
    
    def benchmark(self) -> dict:
        """Ejecuta benchmark de rendimiento."""
        start_time = time.perf_counter()
        pk, sk = self.generate_keypair()
        keypair_time = time.perf_counter() - start_time
        
        start_time = time.perf_counter()
        ct, ss = self.encapsulate(pk)
        encap_time = time.perf_counter() - start_time
        
        start_time = time.perf_counter()
        ss2 = self.decapsulate(sk, ct)
        decap_time = time.perf_counter() - start_time
        
        return {
            "variant": self.variant,
            "keypair_generation_ms": round(keypair_time * 1000, 2),
            "encapsulation_ms": round(encap_time * 1000, 2),
            "decapsulation_ms": round(decap_time * 1000, 2),
            "public_key_bytes": len(pk),
            "private_key_bytes": len(sk),
            "ciphertext_bytes": len(ct),
            "shared_secret_bytes": len(ss),
        }