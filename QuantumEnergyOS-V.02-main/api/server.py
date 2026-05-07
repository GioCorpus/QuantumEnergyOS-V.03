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

app = Flask(__name__)

# Instancia global de criptografía para la infraestructura
infrastructure_crypto = EnergyInfrastructureCrypto()

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


@app.route('/crypto/status', methods=['GET'])
def crypto_status():
    """Estado general del módulo de criptografía"""
    return jsonify({
        "module": "Quantum Cryptography",
        "version": "1.0.0",
        "status": "operational",
        "protocols": ["BB84", "E91", "B92", "SARG04"],
        "pqc_algorithms": ["Kyber", "Dilithium"],
        "active_sessions": len(infrastructure_crypto.active_sessions),
        "mission": "Nunca más apagones en Mexicali - protegidos por física cuántica"
    })

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 5000))
    app.run(host='0.0.0.0', port=port, debug=True)
