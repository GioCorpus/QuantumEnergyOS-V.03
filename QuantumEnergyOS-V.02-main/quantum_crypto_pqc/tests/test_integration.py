"""
QuantumEnergyOS V.02 - PQC Tests
══════════════════════════════════

Tests de integración para el módulo PQC.
"""

import pytest
import time
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from quantum_crypto_pqc import (
    KyberKEM,
    DilithiumSignature,
    FalconSignature,
    SPHINCSSignature,
    HybridChannelManager,
    HybridMode,
    EnergyInfrastructurePQC,
    SessionManager,
    SecurityLevel,
)


class TestPQCInfrastructureIntegration:
    """Tests de integración con infraestructura."""
    
    def test_full_secure_communication_flow(self):
        """Flujo completo: establecer canal, transmitir, verificar."""
        pqc = EnergyInfrastructurePQC(SecurityLevel.HIGH)
        
        # Establecer canal
        session = pqc.establish_secure_channel("SUBESTACION_A", "SUBESTACION_B")
        
        # Transmitir mensaje
        nonce, ciphertext = pqc.secure_transmit(session.session_id, b"LOAD_BALANCE: 50% -> 70%")
        
        # Recibir mensaje
        plaintext = pqc.secure_receive(session.session_id, ciphertext, nonce)
        
        assert plaintext == b"LOAD_BALANCE: 50% -> 70%"
    
    def test_blackout_alert_flow(self):
        """Flujo de alerta de apagón firmada."""
        pqc = EnergyInfrastructurePQC(SecurityLevel.CRITICAL)
        
        signer = DilithiumSignature()
        pk, sk = signer.generate_keypair()
        
        alert = BlackoutAlert(
            alert_id="TEST_001",
            location="Mexicali Centro",
            risk_level="CRITICAL",
            predicted_time=time.time() + 1800,
            grid_load_percent=0.95,
            temperature=48.0,
            node_origin="SUBESTACION_CENTRO"
        )
        
        signed = pqc.sign_blackout_alert(alert, sk)
        assert signed.signature is not None
        
        verified = pqc.verify_blackout_alert(signed, pk)
        assert verified is True


class TestBenchmarkPerformance:
    """Tests de rendimiento y benchmarks."""
    
    def test_kyber_benchmark(self):
        """Benchmark de Kyber."""
        kyber = KyberKEM()
        bench = kyber.benchmark()
        
        assert "keypair_generation_ms" in bench
        assert "encapsulation_ms" in bench
        assert "decapsulation_ms" in bench
        assert bench["keypair_generation_ms"] < 1000  # Debe ser rápido
    
    def test_dilithium_benchmark(self):
        """Benchmark de Dilithium."""
        dilithium = DilithiumSignature()
        bench = dilithium.benchmark()
        
        assert "signing_ms" in bench
        assert "verification_ms" in bench
        assert bench["signing_ms"] < 100  # Firmas deben ser rápidas


class TestSessionRotation:
    """Tests de rotación de sesiones."""
    
    def test_forward_secrecy_rotation(self):
        """Rotación de claves con forward secrecy."""
        pqc = EnergyInfrastructurePQC(SecurityLevel.HIGH)
        
        session_old = pqc.establish_secure_channel("A", "B")
        session_new = pqc.rotate_keys(session_old.session_id)
        
        assert session_new is not None
        assert session_new.session_id != session_old.session_id
        assert session_new.master_key != session_old.master_key


# Import at module level for tests
from quantum_crypto_pqc.models import BlackoutAlert


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])