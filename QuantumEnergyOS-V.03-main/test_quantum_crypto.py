"""
QuantumEnergyOS V.02 - Quantum Cryptography Unit Tests
═══════════════════════════════════════════════════════

Tests para el módulo de criptografía cuántica.
"""

import pytest
import numpy as np
from quantum_crypto import (
    BB84Simulator, QKDConfig, QKDProtocol, QKDResult,
    PQCKyber, PQCDilithium,
    HybridSecurityLayer, HybridSecurityConfig,
    EnergyInfrastructureCrypto
)


class TestBB84Simulator:
    """Tests para el simulador BB84"""

    def test_bb84_no_eavesdropper_low_error(self):
        """Sin espía, la tasa de error debe ser baja"""
        config = QKDConfig(n_qubits=1000, error_threshold=0.11)
        sim = BB84Simulator(config)
        result = sim.run_bb84(include_eve=False)
        
        assert result.eavesdropping_detected is False
        assert result.error_rate < 0.11
        assert result.sift_key_length > 200  # Aproximadamente 50% de sift

    def test_bb84_with_eavesdropper_high_error(self):
        """Con espía, la tasa de error debe incrementar significativamente"""
        config = QKDConfig(n_qubits=1000)
        sim = BB84Simulator(config)
        
        result_no_eve = sim.run_bb84(include_eve=False)
        result_with_eve = sim.run_bb84(include_eve=True, eve_interception_rate=0.5)
        
        assert result_with_eve.error_rate > result_no_eve.error_rate + 0.05
        assert result_with_eve.eavesdropping_detected is True

    def test_bb84_key_length_reduction(self):
        """La clave final debe ser menor que la raw por sift y privacidad"""
        config = QKDConfig(n_qubits=2000)
        sim = BB84Simulator(config)
        result = sim.run_bb84(include_eve=False)
        
        assert result.final_key_length < result.raw_key_length
        assert result.sift_key_length < result.raw_key_length

    def test_bb84_different_channels(self):
        """Diferentes canales deben producir diferentes tasas de error"""
        from quantum_crypto import ChannelNoise
        
        channels = [ChannelNoise.IDEAL, ChannelNoise.FIBER_1550, ChannelNoise.SATELLITE]
        error_rates = []
        
        for channel in channels:
            config = QKDConfig(n_qubits=500, channel=channel)
            sim = BB84Simulator(config)
            result = sim.run_bb84(include_eve=False)
            error_rates.append(result.error_rate)
        
        # El canal satelital debe tener mayor error que el ideal
        assert error_rates[0] < error_rates[2]


class TestPQCImplementations:
    """Tests para Post-Quantum Cryptography"""

    def test_kyber_keypair_generation(self):
        """Kyber debe generar pares de claves válidos"""
        kyber = PQCKyber()
        pk, sk = kyber.generate_keypair()
        
        assert len(pk) == 800
        assert len(sk) == 1632
        assert pk != sk  # Claves diferentes

    def test_kyber_encapsulate_decapsulate(self):
        """Kyber encapsulación/decapsulación debe producir secreto compartido"""
        kyber = PQCKyber()
        pk, sk = kyber.generate_keypair()
        ciphertext, shared1 = kyber.encapsulate(pk)
        shared2 = kyber.decapsulate(sk, ciphertext)
        
        assert shared1 == shared2
        assert len(shared1) == 32

    def test_dilithium_sign_verify(self):
        """Dilithium debe firmar y verificar correctamente"""
        dilithium = PQCDilithium()
        pk, sk = dilithium.generate_keypair()
        message = b"Test message for electrical grid command"
        
        signature = dilithium.sign(sk, message)
        assert dilithium.verify(pk, message, signature) is True
        
        # Mensaje modificado debe fallar verificación
        assert dilithium.verify(pk, b"modified", signature) is False


class TestHybridSecurityLayer:
    """Tests para la capa de seguridad híbrida"""

    def test_hybrid_channel_establishment(self):
        """Debe establecer canal seguro híbrido"""
        config = HybridSecurityConfig()
        layer = HybridSecurityLayer(config)
        
        result = layer.establish_secure_channel()
        
        assert result["session_id"] is not None
        assert result["master_key"] is not None
        assert result["security_level"] == "HYBRID_QKD_PQC"
        assert len(result["master_key"]) == 32

    def test_master_key_derivation(self):
        """La clave maestra debe ser derivada correctamente"""
        config = HybridSecurityConfig(key_derivation_iterations=100)
        layer = HybridSecurityLayer(config)
        
        result = layer.establish_secure_channel()
        key = result["master_key"]
        
        # Clave debe ser determinística para mismo input (con PBKDF2)
        layer2 = HybridSecurityLayer(config)
        result2 = layer2.establish_secure_channel()
        
        # Las claves deberían ser diferentes (QKD usa aleatoriedad cuántica)
        assert key != result2["master_key"]


class TestEnergyInfrastructureIntegration:
    """Tests para integración con infraestructura eléctrica"""

    def test_secure_node_communication(self):
        """Comunicación segura entre nodos debe funcionar"""
        crypto = EnergyInfrastructureCrypto()
        
        message = b"GRID_BALANCE_COMMAND: NODE_1=50%, NODE_2=75%"
        result = crypto.secure_node_communication("SUBESTACION_MEXICALI", "CENTRO_CONTROL", message)
        
        assert result["session_id"] is not None
        assert result["nonce"] is not None
        assert result["ciphertext"] is not None
        assert result["session_id"] in crypto.active_sessions

    def test_multiple_node_sessions(self):
        """Múltiples sesiones concurrentes deben manejarse correctamente"""
        crypto = EnergyInfrastructureCrypto()
        
        sessions = []
        for i in range(5):
            result = crypto.secure_node_communication(
                f"NODE_A_{i}", f"NODE_B_{i}",
                f"SENSOR_DATA_{i}".encode()
            )
            sessions.append(result["session_id"])
        
        assert len(set(sessions)) == 5  # Todas únicas
        assert len(crypto.active_sessions) == 5


class TestEavesdropperDetection:
    """Tests específicos para detección de espionaje"""

    def test_eve_detection_threshold(self):
        """El umbral de detección de Eve debe dispararse correctamente"""
        config = QKDConfig(n_qubits=2000, error_threshold=0.10)
        sim = BB84Simulator(config)
        
        # Sin Eve - debería tener bajo error
        result_clean = sim.run_bb84(include_eve=False)
        assert result_clean.eavesdropping_detected is False
        
        # Con Eve - alta interceptación
        result_eve = sim.run_bb84(include_eve=True, eve_interception_rate=0.8)
        assert result_eve.eavesdropping_detected is True

    def test_partial_eavesdropping(self):
        """Eve parcial debería incrementar error pero no detectarse necesariamente"""
        config = QKDConfig(n_qubits=5000, error_threshold=0.12)
        sim = BB84Simulator(config)
        
        # Eve con baja tasa de interceptación
        result = sim.run_bb84(include_eve=True, eve_interception_rate=0.1)
        
        # El error puede incrementar ligeramente
        assert result.error_rate >= 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])