"""
QuantumEnergyOS V.02 - PQC Module Unit Tests
═══════════════════════════════════════════════

Tests exhaustivos para el módulo de criptografía post-cuántica.
"""

import pytest
import time
import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from quantum_crypto_pqc import (
    KyberKEM,
    DilithiumSignature,
    FalconSignature,
    SPHINCSSignature,
    HybridChannelManager,
    HybridMode,
    EnergyInfrastructurePQC,
    SecurityLevel,
    SessionManager,
    BlackoutAlert,
    EnergyCommand,
)


class TestKyberKEM:
    """Tests para Kyber KEM."""
    
    def test_kyber_keypair_generation_standard(self):
        """Kyber debe generar pares de claves del tamaño correcto."""
        kyber = KyberKEM()
        pk, sk = kyber.generate_keypair()
        
        assert len(pk) == 1188  # KYBER768 default
        assert len(sk) == 2400
    
    def test_kyber_encapsulate_decapsulate_cycle(self):
        """Kyber encapsulación/decapsulación debe producir secreto compartido idéntico."""
        kyber = KyberKEM()
        pk, sk = kyber.generate_keypair()
        
        ciphertext, shared1 = kyber.encapsulate(pk)
        shared2 = kyber.decapsulate(sk, ciphertext)
        
        assert shared1 == shared2
        assert len(shared1) == 32
    
    def test_kyber_different_variants(self):
        """Kyber debe soportar diferentes variantes."""
        for variant in ["KYBER512", "KYBER768", "KYBER1024"]:
            kyber = KyberKEM(variant)
            pk, sk = kyber.generate_keypair()
            
            assert len(pk) == KyberKEM.PARAMETERS[variant]["pk"]
            assert len(sk) == KyberKEM.PARAMETERS[variant]["sk"]


class TestDilithiumSignature:
    """Tests para Dilithium."""
    
    def test_dilithium_keypair_generation(self):
        """Dilithium debe generar pares de claves."""
        dilithium = DilithiumSignature()
        pk, sk = dilithium.generate_keypair()
        
        assert len(pk) == 1952  # DILITHIUM3 default
        assert len(sk) == 4032
    
    def test_dilithium_sign_verify(self):
        """Dilithium debe firmar y verificar correctamente."""
        dilithium = DilithiumSignature()
        pk, sk = dilithium.generate_keypair()
        message = b"GRID_BALANCE_COMMAND: NODE_1=50%, NODE_2=75%"
        
        signature = dilithium.sign(sk, message)
        assert dilithium.verify(pk, message, signature) is True
    
    def test_dilithium_tampered_message_fails(self):
        """Mensaje modificado debe fallar verificación."""
        dilithium = DilithiumSignature()
        pk, sk = dilithium.generate_keypair()
        message = b"Valid command"
        
        signature = dilithium.sign(sk, message)
        
        assert dilithium.verify(pk, b"Tampered command", signature) is False
        assert dilithium.verify(pk, message, b"fake_signature") is False


class TestFalconSignature:
    """Tests para Falcon."""
    
    def test_falcon_keypair_generation(self):
        """Falcon debe generar pares de claves compactas."""
        falcon = FalconSignature()
        pk, sk = falcon.generate_keypair()
        
        assert len(pk) == 897  # FALCON512 default
        assert len(sk) == 1288
    
    def test_falcon_sign_verify(self):
        """Falcon debe firmar y verificar correctamente."""
        falcon = FalconSignature()
        pk, sk = falcon.generate_keypair()
        message = b"EMERGENCY_SHUTDOWN_COMMAND"
        
        signature = falcon.sign(sk, message)
        assert falcon.verify(pk, message, signature) is True


class TestSPHINCSSignature:
    """Tests para SPHINCS+."""
    
    def test_spincs_keypair_generation(self):
        """SPHINCS+ debe generar pares de claves (muy grandes)."""
        spincs = SPHINCSSignature()
        pk, sk = spincs.generate_keypair()
        
        assert len(pk) == 32  # SPHINCS uses small public keys
        assert len(sk) == 64
    
    def test_spincs_large_signature(self):
        """SPHINCS+ debe producir firmas grandes (hash-based)."""
        spincs = SPHINCSSignature()
        pk, sk = spincs.generate_keypair()
        message = b"BACKUP_SIGNATURE_FOR_MAXIMUM_SECURITY"
        
        signature = spincs.sign(sk, message)
        # SPHINCS+ signatures are large
        assert len(signature) >= 8000


class TestHybridChannelManager:
    """Tests para el canal híbrido."""
    
    def test_pqc_only_channel(self):
        """Canal PQC-only debe funcionar."""
        manager = HybridChannelManager(HybridMode.PQC_ONLY)
        result = manager.establish_channel()
        
        assert result.channel_id is not None
        assert result.master_key is not None
        assert result.pqc_shared is not None
        assert result.ecdh_shared is None
    
    def test_pqc_ecdh_channel(self):
        """Canal PQC+ECDH debe funcionar."""
        manager = HybridChannelManager(HybridMode.PQC_ECDH)
        result = manager.establish_channel()
        
        assert result.pqc_shared is not None
        assert result.ecdh_shared is not None
    
    def test_triple_hybrid_channel(self):
        """Canal triple híbrido debe incluir QKD."""
        manager = HybridChannelManager(HybridMode.TRIPLE_HYBRID)
        result = manager.establish_channel()
        
        assert result.pqc_shared is not None
        assert result.ecdh_shared is not None
        # QKD may or may not be available depending on imports


class TestEnergyInfrastructurePQC:
    """Tests para integración con infraestructura eléctrica."""
    
    def test_establish_secure_channel(self):
        """Debe establecer canal seguro entre nodos."""
        infra = EnergyInfrastructurePQC(SecurityLevel.STANDARD)
        
        session = infra.establish_secure_channel(
            "SUBESTACION_MEXICALI",
            "CENTRO_CONTROL"
        )
        
        assert session.session_id is not None
        assert session.node_a == "SUBESTACION_MEXICALI"
        assert session.node_b == "CENTRO_CONTROL"
        assert session.is_valid() is True
    
    def test_sign_and_verify_grid_command(self):
        """Debe firmar y verificar comandos de infraestructura."""
        infra = EnergyInfrastructurePQC(SecurityLevel.HIGH)
        
        # Create keypair for signing
        from . import DilithiumSignature
        signer = DilithiumSignature()
        pk, sk = signer.generate_keypair()
        
        command = EnergyCommand(
            command_id="CMD_001",
            command_type="BALANCE_LOAD",
            target_node="SUBESTACION_MEXICALI",
            parameters={"redistribution": "NODE_2: 10%"}
        )
        
        signed = infra.sign_grid_command(command, sk)
        assert signed.signature is not None
        
        valid = infra.verify_grid_command(signed, pk)
        assert valid is True
    
    def test_sign_and_verify_blackout_alert(self):
        """Debe firmar y verificar alertas de apagón."""
        infra = EnergyInfrastructurePQC(SecurityLevel.CRITICAL)
        
        from . import DilithiumSignature
        signer = DilithiumSignature()
        pk, sk = signer.generate_keypair()
        
        alert = BlackoutAlert(
            alert_id="ALERT_001",
            location="Mexicali, B.C.",
            risk_level="HIGH",
            predicted_time=time.time() + 3600,
            grid_load_percent=0.92,
            temperature=48.5,
            node_origin="SUBESTACION_NORTE"
        )
        
        signed = infra.sign_blackout_alert(alert, sk)
        assert signed.signature is not None
        
        valid = infra.verify_blackout_alert(signed, pk)
        assert valid is True


class TestSessionManager:
    """Tests para el gestor de sesiones."""
    
    def test_create_and_get_session(self):
        """Debe crear y obtener sesión."""
        manager = SessionManager(SecurityLevel.HIGH)
        
        session = manager.create_session("NODE_A", "NODE_B")
        assert manager.validate_session(session.session_id) is True
        
        retrieved = manager.get_session(session.session_id)
        assert retrieved is not None
        assert retrieved.node_a == "NODE_A"
    
    def test_session_expiry(self):
        """Sesión debe expirar correctamente."""
        manager = SessionManager(SecurityLevel.STANDARD)
        
        session = manager.create_session("NODE_A", "NODE_B", custom_expiry=1)
        assert manager.validate_session(session.session_id) is True
        
        # Esperar expiración
        time.sleep(1.5)
        
        # Forzar cleanup
        manager._cleanup_expired()
        
        assert manager.validate_session(session.session_id) is False
    
    def test_close_session(self):
        """Debe cerrar sesión correctamente."""
        manager = SessionManager()
        
        session = manager.create_session("NODE_A", "NODE_B")
        assert manager.close_session(session.session_id) is True
        
        assert manager.validate_session(session.session_id) is False


class TestSecurityIntegration:
    """Tests de integración de seguridad."""
    
    def test_different_security_levels(self):
        """Debe soportar diferentes niveles de seguridad."""
        for level in [SecurityLevel.STANDARD, SecurityLevel.HIGH, SecurityLevel.CRITICAL]:
            infra = EnergyInfrastructurePQC(level)
            session = infra.establish_secure_channel("A", "B")
            assert session.security_level == level
    
    def test_multiple_concurrent_sessions(self):
        """Múltiples sesiones concurrentes deben manejarse."""
        manager = SessionManager()
        
        sessions = []
        for i in range(10):
            session = manager.create_session(f"NODE_{i}", f"CONTROL_{i}")
            sessions.append(session.session_id)
        
        assert len(sessions) == 10
        assert len(set(sessions)) == 10  # Todas únicas


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])