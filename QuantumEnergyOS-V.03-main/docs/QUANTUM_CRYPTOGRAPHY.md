# QuantumEnergyOS V.02 - Quantum Cryptography Module

## Tabla de Contenidos

1. [Visión General](#visión-general)
2. [Arquitectura](#arquitectura)
3. [Protocolos QKD Implementados](#protocolos-qkd-implementados)
4. [Post-Quantum Cryptography (PQC)](#post-quantum-cryptography-pqc)
5. [Integración con Flask API](#integración-con-flask-api)
6. [Tests](#tests)
7. [Referencias Teóricas](#referencias-teóricas)

---

## Visión General

El módulo de **Criptografía Cuántica** de QuantumEnergyOS proporciona protección de comunicaciones basada en la física cuántica para infraestructura eléctrica crítica. Combina:

- **QKD (Quantum Key Distribution)**: Distribución de claves basada en principios cuánticos
- **PQC (Post-Quantum Cryptography)**: Algoritmos resistentes a ataques cuánticos
- **Seguridad híbrida**: Combinación óptima de ambas tecnologías

### Misión

> **Nunca más apagones en Mexicali** — protegidos por la física cuántica.

---

## Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│              APLICACIONES (Grid, Sensores, Control)          │
├─────────────────────────────────────────────────────────────┤
│           Flask API  ·  WebSocket  ·  React UI               │
├─────────────────────────────────────────────────────────────┤
│        QuantumCryptography.qs  ·  quantum_crypto.py          │
│  BB84  ·  E91  ·  Kyber  ·  Dilithium  ·  Hybrid Layer       │
├──────────────────────────┬──────────────────────────────────┤
│  PhotonicQ Bridge        │      Simulación de Canales       │
│  (syscall → óptico)      │      Ruido realista              │
├──────────────────────────┴──────────────────────────────────┤
│              Núcleo Clásico Arch Linux                      │
│         Scheduling · Memoria · I/O                          │
└─────────────────────────────────────────────────────────────┘
```

---

## Protocolos QKD Implementados

### BB84 (Bennett-Brassard 1984)

El primer protocolo práctico de distribución de claves cuánticas.

#### Principio

1. **Codificación**: 4 estados de un fotón (polarizaciones)
   - Base recta: |0⟩ = horizontal, |1⟩ = vertical
   - Base diagonal: |+⟩ = diagonal, |-⟩ = antidiagonal

2. **Proceso**:
   - Alice genera bits aleatorios y bases aleatorias
   - Prepara fotones según bit y base
   - Bob mide en bases aleatorias
   - Publican bases (no bits) — "sift" de clave
   - Corrección de errores y amplificación de privacidad

#### Código Q#

```qsharp
// Preparar fotón BB84
operation PrepararFotonBB84(q : Qubit, bit : Bool, base : Bool) : Unit {
    if bit {
        X(q);  // Bit = 1
    }
    if base {
        H(q);  // Base diagonal: |+⟩ o |-⟩
    }
}
```

#### Simulación Python

```python
from quantum_crypto import BB84Simulator, QKDConfig

sim = BB84Simulator(QKDConfig(n_qubits=1000))
result = sim.run_bb84(include_eve=True)

print(f"Tasa de error: {result.error_rate:.1%}")
print(f"Eve detectado: {result.eavesdropping_detected}")
```

### E91 (Ekert 1991)

Protocolo basado en entrelazamiento cuántico y violación de la desigualdad de Bell.

#### Ventaja

- Detección de espionaje mediante violación de Bell
- Seguridad basada en principios fundamentales de QM

### B92 y SARG04

Variantes de BB84 con diferentes estrategias de sift.

---

## Post-Quantum Cryptography (PQC)

### NIST PQC Standard (2022)

| Algoritmo | Tipo | Uso en QuantumEnergyOS |
|-----------|------|------------------------|
| **Kyber** | KEM | Encapsulación de claves |
| **Dilithium** | Firma | Autenticación de comandos |
| **Falcon** | Firma | Opcional, para firmas más pequeñas |
| **SPHINCS+** | Firma | Fallback hash-based |

### Implementación

```python
from quantum_crypto import PQCKyber, PQCDilithium

# Kyber para intercambio de claves
kyber = PQCKyber()
pk, sk = kyber.generate_keypair()
ciphertext, shared_secret = kyber.encapsulate(pk)

# Dilithium para firmas
dilithium = PQCDilithium()
sig = dilithium.sign(sk, b"comando_control_red")
assert dilithium.verify(pk, b"comando_control_red", sig)
```

---

## Integración con Flask API

### Endpoints Disponibles

#### `POST /crypto/qkd/bb84`

Ejecuta protocolo BB84.

```bash
curl -X POST http://localhost:5000/crypto/qkd/bb84 \
  -H "Content-Type: application/json" \
  -d '{"n_qubits": 1000, "channel": "fiber_1550"}'
```

**Respuesta**:
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "protocol": "BB84",
  "error_rate": 0.067,
  "qber": 0.067,
  "eavesdropping_detected": false,
  "final_key_length": 198
}
```

#### `POST /crypto/qkd/hybrid`

Establece canal seguro híbrido QKD + PQC.

```bash
curl -X POST http://localhost:5000/crypto/qkd/hybrid
```

#### `POST /crypto/secure-comm`

Comunica nodos de forma segura.

```bash
curl -X POST http://localhost:5000/crypto/secure-comm \
  -H "Content-Type: application/json" \
  -d '{"node_a": "SUBESTACION_MEXICALI", "node_b": "CENTRO_CONTROL", "message": "CMD: REDUCE_LOAD_15%"}'
```

---

## Tests

### Ejecutar Tests

```bash
# Tests unitarios
python -m pytest test_quantum_crypto.py -v

# Test rápido de funcionalidad
python -c "
from quantum_crypto import BB84Simulator, QKDConfig
s = BB84Simulator(QKDConfig(n_qubits=100))
r = s.run_bb84(include_eve=False)
assert r.eavesdropping_detected == False
assert r.error_rate < 0.11
"
```

### Casos de Prueba

| Test | Descripción | Resultado Esperado |
|------|-------------|-------------------|
| `test_bb84_basic` | BB84 sin Eve | Error < 11%, sin detección |
| `test_bb84_eavesdropper` | BB84 con Eve | Error > 11%, detección activa |
| `test_quantum_crypto` | Integración híbrida | Clave maestra derivada |

---

## Referencias Teóricas

### BB84

> C. H. Bennett and G. Brassard, "Quantum cryptography: Public key distribution and coin tossing," *Proceedings of IEEE International Conference on Computers, Systems and Signal Processing*, Bangalore, India, 1984.

### E91

> A. Ekert, "Quantum cryptography based on Bell's theorem," *Physical Review Letters*, vol. 67, no. 15, pp. 661-663, 1991.

### NIST PQC

> NIST, "Post-Quantum Cryptography Standardization," *FIPS 203, 204*, 2024.

### QBER (Quantum Bit Error Rate)

El QBER se calcula como:

```
QBER = errores_detectados / bits_comparados
```

**Umbral de seguridad**: QBER > 11% indica posible espionaje (para BB84).

---

## Roadmap

- [ ] Integración con hardware fotónico real (QuiX, Xanadu)
- [ ] Implementación completa de CASCADE para corrección de errores
- [ ] Amplificación de privacidad con códigos LDPC
- [ ] ETSI 006 compliance para QKD networks
- [ ] Certificación Common Criteria para infraestructura crítica

---

**QuantumEnergyOS** — *El quantum fluye, la energía permanece*.