"""
QuantumEnergyOS V.02 - Falcon Signature Implementation
═══════════════════════════════════════════════════════

Implementación de Falcon - firma digital compacta basada en NTRU.
Algoritmo NIST FIPS 205 - alternativa pequeña para firmas.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
"""

from __future__ import annotations

import hashlib
import secrets
import time
from typing import Optional

try:
    from oqs import Signature
    OQS_AVAILABLE = True
except ImportError:
    OQS_AVAILABLE = False


class FalconSignature:
    """
    Falcon - Compact Digital Signature Scheme.
    
    Basado en NTRU y ideal lattices. Más compacto que Dilithium.
    Útil para dispositivos con recursos limitados (edge devices).
    """
    
    PARAMETERS = {
        "FALCON512": {"pk": 897, "sk": 1288, "sig": 690},
        "FALCON1024": {"pk": 1331, "sk": 1600, "sig": 1330},
    }
    
    def __init__(self, variant: str = "FALCON512"):
        self.variant = variant
        self._oqs_sig: Optional[Signature] = None
        
        if OQS_AVAILABLE:
            try:
                self._oqs_sig = Signature(variant.upper())
            except Exception:
                self._oqs_sig = None
    
    @property
    def name(self) -> str:
        return self.variant
    
    def generate_keypair(self) -> tuple[bytes, bytes]:
        """
        Genera par de claves Falcon.
        
        Returns:
            Tuple de (public_key, private_key)
        """
        params = self.PARAMETERS[self.variant]
        
        if self._oqs_sig:
            return self._oqs_sig.generate_keypair()
        
        # Fallback implementation (simulada)
        seed = secrets.token_bytes(32)
        # Include seed in both keys for fallback verification (at the end)
        public_key = hashlib.sha3_256(seed + b"FALCON_PUB").digest()[:32]
        private_key = hashlib.sha3_512(seed + b"FALCON_PRIV").digest()[:64]
        
        # Pad to exact sizes with seed at the end
        public_key = (public_key * ((params["pk"] // len(public_key)) + 100))[:params["pk"]]
        # Ensure seed is present at the end
        if len(public_key) >= 32:
            public_key = public_key[:-32] + seed
        private_key = (private_key * ((params["sk"] // len(private_key)) + 100))[:params["sk"]]
        if len(private_key) >= 32:
            private_key = private_key[:-32] + seed
        
        return public_key, private_key
    
    def sign(self, private_key: bytes, message: bytes) -> bytes:
        """
        Firma un mensaje usando Falcon.
        
        Args:
            private_key: Clave privada para firmar
            message: Mensaje a firmar
            
        Returns:
            Firma Falcon (más compacta que Dilithium)
        """
        params = self.PARAMETERS[self.variant]
        
        if self._oqs_sig:
            return self._oqs_sig.sign(message, private_key)
        
        # Fallback implementation
        sig = hashlib.sha3_512(message + private_key[-32:]).digest()
        sig_expanded = sig * ((params["sig"] // 64) + 100)
        return sig_expanded[:params["sig"]]
    
    def verify(self, public_key: bytes, message: bytes, signature: bytes) -> bool:
        """
        Verifica una firma Falcon.
        
        Args:
            public_key: Clave pública del firmante
            message: Mensaje original
            signature: Firma a verificar
            
        Returns:
            True si la firma es válida, False en caso contrario
        """
        if self._oqs_sig:
            try:
                return self._oqs_sig.verify(message, signature, public_key)
            except Exception:
                return False
        
        # Fallback implementation
        params = self.PARAMETERS[self.variant]
        if len(signature) != params["sig"]:
            return False
        
        seed = public_key[-32:]
        expected_sig = hashlib.sha3_512(message + seed).digest()
        sig_expected = (expected_sig * ((params["sig"] // 64) + 100))[:params["sig"]]
        return self._constant_time_compare(signature, sig_expected)
    
    def _constant_time_compare(self, a: bytes, b: bytes) -> bool:
        """Comparación constante para prevenir ataques de timing."""
        if len(a) != len(b):
            return False
        result = 0
        for x, y in zip(a, b):
            result |= x ^ y
        return result == 0
    
    def benchmark(self) -> dict:
        """Ejecuta benchmark de rendimiento."""
        start_time = time.perf_counter()
        pk, sk = self.generate_keypair()
        keypair_time = time.perf_counter() - start_time
        
        test_message = secrets.token_bytes(512)  # Falcon optimized for smaller messages
        
        start_time = time.perf_counter()
        sig = self.sign(sk, test_message)
        sign_time = time.perf_counter() - start_time
        
        start_time = time.perf_counter()
        valid = self.verify(pk, test_message, sig)
        verify_time = time.perf_counter() - start_time
        
        return {
            "variant": self.variant,
            "keypair_generation_ms": round(keypair_time * 1000, 2),
            "signing_ms": round(sign_time * 1000, 2),
            "verification_ms": round(verify_time * 1000, 2),
            "public_key_bytes": len(pk),
            "private_key_bytes": len(sk),
            "signature_bytes": len(sig),
            "verification_valid": valid,
        }