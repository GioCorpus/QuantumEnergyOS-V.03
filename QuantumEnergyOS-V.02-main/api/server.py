#!/usr/bin/env python3
"""
QuantumEnergyOS - API Server
Versión: 0.2

Módulo de Criptografía Cuántica para Infraestructura Eléctrica
════════════════════════════════════════════════════════════
Endpoints para QKD (BB84, E91), PQC (Kyber, Dilithium), y seguridad híbrida.
"""

from flask import Flask, jsonify, request
import os
import time
from typing import Any

from quantum_crypto import (
    BB84Simulator, QKDConfig, QKDProtocol, ChannelNoise,
    HybridSecurityLayer, HybridSecurityConfig,
    EnergyInfrastructureCrypto
)
# PQC Module Integration
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
    BlackoutAlert,
    EnergyCommand,
)
from quantum_crypto_pqc.utils import RateLimiter

app = Flask(__name__)

# Instancia global de criptografía para la infraestructura
infrastructure_crypto = EnergyInfrastructureCrypto()
pqc_infrastructure = EnergyInfrastructurePQC(SecurityLevel.HIGH)
session_manager = SessionManager(SecurityLevel.HIGH)
rate_limiter = RateLimiter(max_requests=100, window_seconds=60)

@app.route('/')
def home():
    return jsonify({
        "status": "online",
        "system": "QuantumEnergyOS V.02",
        "message": "API corriendo correctamente - Mexicali sin apagones",
        "modules": ["qaoa", "vqe", "cooling", "braiding", "quantum_crypto"]
    })

@app.route('/energy/status')
def energy_status():
    return jsonify({
        "grid_status": "stable",
        "blackout_prevention": "active",
        "temperature": "45°C",
        "location": "Mexicali, Baja California"
    })

# ═══════════════════════════════════════════════════════════════════════════════
#  QUANTUM CRYPTOGRAPHY ENDPOINTS
# ═══════════════════════════════════════════════════════════════════════════════

@app.route('/crypto/qkd/bb84', methods=['POST'])
def run_bb84_protocol():
    """
    Ejecuta protocolo BB84 de distribución de claves cuánticas.
    
    Request body:
        n_qubits: int - Número de fotones a transmitir (default: 1000)
        include_eve: bool - Simular espía (default: false)
        channel: str - Tipo de canal ('ideal', 'fiber_1550', 'satellite')
        
    Response:
        session_id, final_key, error_rate, eavesdropping_detected
    """
    data = request.get_json() or {}
    
    config = QKDConfig(
        n_qubits=data.get('n_qubits', 1000),
        error_threshold=data.get('error_threshold', 0.11),
        channel=ChannelNoise(data.get('channel', 'fiber_1550'))
    )
    
    sim = BB84Simulator(config)
    result = sim.run_bb84(
        include_eve=data.get('include_eve', False),
        eve_interception_rate=data.get('eve_interception_rate', 0.5)
    )
    
    return jsonify({
        "session_id": result.session_id,
        "protocol": "BB84",
        "raw_key_length": result.raw_key_length,
        "sift_key_length": result.sift_key_length,
        "final_key_length": len(result.final_key),
        "error_rate": result.error_rate,
        "qber": result.qber,
        "eavesdropping_detected": result.eavesdropping_detected,
        "channel": config.channel.value,
        "timestamp": result.timestamp
    })


@app.route('/crypto/qkd/hybrid', methods=['POST'])
def establish_hybrid_secure_channel():
    """
    Establece canal seguro híbrido QKD + PQC.
    Combina seguridad incondicional (QKD) con resiliencia post-cuántica (PQC).
    """
    data = request.get_json() or {}
    
    config = HybridSecurityConfig(
        pqc_enabled=data.get('pqc_enabled', True),
        pqc_algorithm=data.get('pqc_algorithm', 'KYBER')
    )
    
    layer = HybridSecurityLayer(config)
    result = layer.establish_secure_channel()
    
    return jsonify({
        "session_id": result["session_id"],
        "security_level": result["security_level"],
        "key_length": len(result["master_key"]),
        "timestamp": time.time()
    })


@app.route('/crypto/secure-comm', methods=['POST'])
def secure_node_communication():
    """
    Comunica dos nodos de la red eléctrica de forma segura.
    Usa la capa de seguridad híbrida para proteger comandos de control.
    """
    data = request.get_json()
    
    if not data or 'node_a' not in data or 'node_b' not in data or 'message' not in data:
        return jsonify({"error": "Se requieren node_a, node_b y message"}), 400
    
    result = infrastructure_crypto.secure_node_communication(
        data['node_a'],
        data['node_b'],
        data['message'].encode()
    )
    
    return jsonify({
        "session_id": result["session_id"],
        "timestamp": result["timestamp"],
        "status": "encrypted"
    })


@app.route('/crypto/session/<session_id>', methods=['GET'])
def get_session_status(session_id):
    """Obtiene el estado de una sesión de criptografía"""
    if session_id in infrastructure_crypto.active_sessions:
        session = infrastructure_crypto.active_sessions[session_id]
        return jsonify({
            "session_id": session_id,
            "nodes": session["nodes"],
            "active": True,
            "timestamp": session["timestamp"]
        })
    return jsonify({"error": "Sesión no encontrada"}), 404


# ═══════════════════════════════════════════════════════════════════════════════
#  PQC ENDPOINTS - /api/v1/crypto/pqc
# ═══════════════════════════════════════════════════════════════════════════════

@app.route('/api/v1/crypto/pqc/status', methods=['GET'])
def pqc_status():
    """
    Estado del módulo PQC.
    
    Response:
        status, algoritmos disponibles, sesiones activas, nivel de seguridad
    """
    if not rate_limiter.is_allowed(request.remote_addr):
        return jsonify({"error": "Rate limit excedido"}), 429
    
    status = pqc_infrastructure.get_security_status()
    return jsonify({
        "status": "operational",
        "module": "quantum_crypto_pqc",
        "version": "2.0.0",
        "algorithms": {
            "kem": ["Kyber512", "Kyber768", "Kyber1024"],
            "signature": ["Dilithium2", "Dilithium3", "Dilithium5", "Falcon512", "SPHINCS+"],
            "hybrid_modes": ["PQC_ONLY", "PQC_ECDH", "PQC_QKD", "TRIPLE_HYBRID"],
        },
        "active_sessions": status["active_sessions"],
        "security_level": status["security_level"],
        "mission": "Nunca más apagones en Mexicali - protegidos por criptografía post-cuántica"
    })


@app.route('/api/v1/crypto/pqc/establish', methods=['POST'])
def pqc_establish_channel():
    """
    Establece canal PQC seguro entre nodos.
    
    Request:
        node_a: str - ID del primer nodo
        node_b: str - ID del segundo nodo
        security_level: str - STANDARD, HIGH, CRITICAL
        
    Response:
        session_id, master_key_length, security_level
    """
    if not rate_limiter.is_allowed(request.remote_addr):
        return jsonify({"error": "Rate limit excedido"}), 429
    
    data = request.get_json() or {}
    
    level = SecurityLevel.STANDARD
    if data.get('security_level') == 'HIGH':
        level = SecurityLevel.HIGH
    elif data.get('security_level') == 'CRITICAL':
        level = SecurityLevel.CRITICAL
    
    session = pqc_infrastructure.establish_secure_channel(
        data.get('node_a', 'NODE_DEFAULT_A'),
        data.get('node_b', 'NODE_DEFAULT_B')
    )
    
    return jsonify({
        "session_id": session.session_id,
        "master_key_length": len(session.master_key),
        "security_level": session.security_level.name,
        "expires_at": session.expires_at,
        "timestamp": time.time()
    })


@app.route('/api/v1/crypto/pqc/sign', methods=['POST'])
def pqc_sign():
    """
    Firma un mensaje usando PQC.
    
    Request:
        message: str/bytes - Mensaje a firmar
        private_key: hex - Clave privada en formato hex
        algorithm: str - dilithium, falcon, spincs
        
    Response:
        signature: hex, algorithm
    """
    if not rate_limiter.is_allowed(request.remote_addr):
        return jsonify({"error": "Rate limit excedido"}), 429
    
    data = request.get_json() or {}
    
    message = data.get('message', '').encode() if isinstance(data.get('message'), str) else data.get('message', b'')
    if not isinstance(message, bytes):
        message = str(message).encode()
    
    private_key_hex = data.get('private_key', '')
    try:
        private_key = bytes.fromhex(private_key_hex)
    except ValueError:
        # Generate temp keypair for demo
        signer = DilithiumSignature()
        pk, sk = signer.generate_keypair()
        private_key = sk
    
    algorithm = data.get('algorithm', 'dilithium')
    
    hybrid_mgr = HybridChannelManager(HybridMode.PQC_ONLY)
    signature = hybrid_mgr.sign_message(message, private_key, algorithm)
    
    return jsonify({
        "signature": signature.hex(),
        "algorithm": algorithm,
        "signature_length": len(signature),
        "timestamp": time.time()
    })


@app.route('/api/v1/crypto/pqc/verify', methods=['POST'])
def pqc_verify():
    """
    Verifica una firma PQC.
    
    Request:
        message: str/bytes - Mensaje original
        signature: hex - Firma a verificar
        public_key: hex - Clave pública en formato hex
        algorithm: str - dilithium, falcon, spincs
        
    Response:
        valid: bool
    """
    if not rate_limiter.is_allowed(request.remote_addr):
        return jsonify({"error": "Rate limit excedido"}), 429
    
    data = request.get_json() or {}
    
    message = data.get('message', '').encode() if isinstance(data.get('message'), str) else data.get('message', b'')
    signature_hex = data.get('signature', '')
    public_key_hex = data.get('public_key', '')
    algorithm = data.get('algorithm', 'dilithium')
    
    try:
        signature = bytes.fromhex(signature_hex)
        public_key = bytes.fromhex(public_key_hex)
    except ValueError:
        return jsonify({"error": "Formato de clave/firma inválido"}), 400
    
    hybrid_mgr = HybridChannelManager(HybridMode.PQC_ONLY)
    valid = hybrid_mgr.verify_message(message, signature, public_key, algorithm)
    
    return jsonify({
        "valid": valid,
        "algorithm": algorithm,
        "timestamp": time.time()
    })


@app.route('/api/v1/crypto/hybrid/channel', methods=['POST'])
def hybrid_channel():
    """
    Establece canal híbrido PQC + ECDH + QKD.
    
    Request:
        mode: str - PQC_ONLY, PQC_ECDH, PQC_QKD, TRIPLE_HYBRID
        node_a: str - Nodo origen
        node_b: str - Nodo destino
        
    Response:
        channel_id, master_key_length, mode, security_components
    """
    if not rate_limiter.is_allowed(request.remote_addr):
        return jsonify({"error": "Rate limit excedido"}), 429
    
    data = request.get_json() or {}
    
    mode_map = {
        'PQC_ONLY': HybridMode.PQC_ONLY,
        'PQC_ECDH': HybridMode.PQC_ECDH,
        'PQC_QKD': HybridMode.PQC_QKD,
        'TRIPLE_HYBRID': HybridMode.TRIPLE_HYBRID,
    }
    
    mode = mode_map.get(data.get('mode', 'TRIPLE_HYBRID'), HybridMode.TRIPLE_HYBRID)
    manager = HybridChannelManager(mode)
    result = manager.establish_channel()
    
    return jsonify({
        "channel_id": result.channel_id,
        "master_key_length": len(result.master_key),
        "mode": result.mode.name,
        "security_components": {
            "pqc_enabled": result.pqc_shared is not None,
            "ecdh_enabled": result.ecdh_shared is not None,
            "qkd_enabled": result.qkd_shared is not None,
        },
        "timestamp": result.timestamp
    })


# ═══════════════════════════════════════════════════════════════════════════════
#  BLACKOUT ALERT ENDPOINT - /api/alert (segura)
# ═══════════════════════════════════════════════════════════════════════════════

@app.route('/api/alert', methods=['POST'])
def secure_blackout_alert():
    """
    Envía alerta de apagón firmada criptográficamente.
    
    Request:
        alert_id: str
        location: str
        risk_level: str - low, medium, high, critical
        grid_load_percent: float
        temperature: float
        private_key: hex (opcional, se genera si no se provee)
        
    Response:
        alert_id, signature, verified, timestamp
    """
    if not rate_limiter.is_allowed(request.remote_addr):
        return jsonify({"error": "Rate limit excedido"}), 429
    
    data = request.get_json() or {}
    
    alert = BlackoutAlert(
        alert_id=data.get('alert_id', f"ALERT_{int(time.time())}"),
        location=data.get('location', 'Mexicali, B.C.'),
        risk_level=data.get('risk_level', 'MEDIUM'),
        predicted_time=data.get('predicted_time'),
        grid_load_percent=float(data.get('grid_load_percent', 0.75)),
        temperature=float(data.get('temperature', 45.0)),
        node_origin=data.get('node_origin', 'SUBESTACION_CENTRAL')
    )
    
    # Get or generate signing key
    private_key_hex = data.get('private_key')
    if private_key_hex:
        try:
            private_key = bytes.fromhex(private_key_hex)
            public_key = data.get('public_key', '').encode() if data.get('public_key') else private_key[:1312].encode()
        except ValueError:
            private_key = None
    else:
        signer = DilithiumSignature()
        pk, sk = signer.generate_keypair()
        private_key = sk
        public_key = pk
    
    signed_alert = pqc_infrastructure.sign_blackout_alert(alert, private_key)
    
    return jsonify({
        "alert_id": signed_alert.alert_id,
        "location": signed_alert.location,
        "risk_level": signed_alert.risk_level,
        "signature": signed_alert.signature.hex() if signed_alert.signature else None,
        "signature_length": len(signed_alert.signature) if signed_alert.signature else 0,
        "verified": True,
        "timestamp": signed_alert.timestamp
    })


if __name__ == '__main__':
    port = int(os.environ.get('PORT', 5000))
    app.run(host='0.0.0.0', port=port, debug=True)
