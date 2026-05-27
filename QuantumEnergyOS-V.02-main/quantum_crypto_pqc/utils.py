"""
QuantumEnergyOS V.02 - PQC Utils
═══════════════════════════════════

Utilidades de criptografía para PQC: encriptación, KDF, rate limiting.

Autor: QuantumEnergyOS Team — Mexicali, B.C.
"""

from __future__ import annotations

import hashlib
import hmac
import secrets
import time
from collections import defaultdict
from typing import Any, Optional


def secure_encrypt(plaintext: bytes, key: bytes) -> tuple[bytes, bytes]:
    """
    Encripta usando keystream derivado de la clave.
    
    Args:
        plaintext: Mensaje a encriptar
        key: Clave de encriptación
        
    Returns:
        Tuple de (nonce, ciphertext)
    """
    nonce = secrets.token_bytes(12)
    keystream = hashlib.sha3_256(key + nonce).digest() * (len(plaintext) // 32 + 2)
    ciphertext = bytes(p ^ k for p, k in zip(plaintext, keystream))
    return nonce, ciphertext


def secure_decrypt(ciphertext: bytes, key: bytes, nonce: bytes) -> bytes:
    """
    Desencripta usando keystream derivado de la clave.
    
    Args:
        ciphertext: Mensaje encriptado
        key: Clave de encriptación
        nonce: Nonce usado en encriptación
        
    Returns:
        Mensaje desencriptado
    """
    keystream = hashlib.sha3_256(key + nonce).digest() * (len(ciphertext) // 32 + 2)
    plaintext = bytes(c ^ k for c, k in zip(ciphertext, keystream))
    return plaintext


def derive_key_hkdf(input_key: bytes, salt: bytes, info: bytes = b"") -> bytes:
    """
    Deriva clave usando HKDF-SHA3-256.
    
    Args:
        input_key: Clave de entrada
        salt: Sal para el KDF
        info: Información contextual
        
    Returns:
        Clave derivada de 32 bytes
    """
    return hashlib.pbkdf2_hmac(
        'sha3_256',
        input_key,
        salt + info,
        10000,
        dklen=32
    )


class RateLimiter:
    """
    Limitador de velocidad para prevenir ataques de fuerza bruta.
    
    Protege endpoints de criptografía contra abuso.
    """
    
    def __init__(self, max_requests: int = 100, window_seconds: int = 60):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self._requests: dict[str, list[float]] = defaultdict(list)
    
    def is_allowed(self, client_id: str) -> bool:
        """
        Verifica si una petición está permitida.
        
        Args:
            client_id: Identificador del cliente (IP, node_id, etc.)
            
        Returns:
            True si la petición está permitida
        """
        now = time.time()
        requests = self._requests[client_id]
        
        # Clean old requests
        self._requests[client_id] = [t for t in requests if now - t < self.window_seconds]
        
        if len(self._requests[client_id]) >= self.max_requests:
            return False
        
        self._requests[client_id].append(now)
        return True
    
    def get_remaining(self, client_id: str) -> int:
        """Obtiene peticiones restantes en la ventana."""
        now = time.time()
        requests = self._requests.get(client_id, [])
        valid = [t for t in requests if now - t < self.window_seconds]
        return max(0, self.max_requests - len(valid))
    
    def reset(self, client_id: Optional[str] = None):
        """Resetea el contador de peticiones."""
        if client_id:
            self._requests.pop(client_id, None)
        else:
            self._requests.clear()


def constant_time_compare(a: bytes, b: bytes) -> bool:
    """Comparación constante para prevenir ataques de timing."""
    if len(a) != len(b):
        return False
    result = 0
    for x, y in zip(a, b):
        result |= x ^ y
    return result == 0


def secure_hash(data: bytes) -> bytes:
    """Hash seguro usando SHA3-256."""
    return hashlib.sha3_256(data).digest()


def secure_hash_512(data: bytes) -> bytes:
    """Hash seguro usando SHA3-512."""
    return hashlib.sha3_512(data).digest()


def generate_secure_random(length: int) -> bytes:
    """Genera bytes aleatorios seguros."""
    return secrets.token_bytes(length)


def xor_bytes(a: bytes, b: bytes) -> bytes:
    """XOR entre dos secuencias de bytes."""
    return bytes(x ^ y for x, y in zip(a, b))


def secure_zero(data: bytearray):
    """Borra seguramente un buffer de datos."""
    for i in range(len(data)):
        data[i] = 0