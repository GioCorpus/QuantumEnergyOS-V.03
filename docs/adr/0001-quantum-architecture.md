# ADR 0001 — Arquitectura base de QuantumEnergyOS

Estado: Propuesta / pendiente de aprobación

Fecha: 2026-07-31

Autores: Equipo de Arquitectura — QuantumEnergyOS

---

## Título

ADR 0001 — Arquitectura base: kernel, HAL, drivers, init, gestor de servicios, registro de servicios, IPC, capa de transporte, framework de servicios y plataformas (Identity, Policy, Browser, Dashboard, Wayland editions).

## Contexto

QuantumEnergyOS es un sistema operativo modular cuyo objetivo es soportar múltiples ediciones de escritorio (Core con tinywl y Workstation con KDE) y un conjunto de servicios nativos del sistema: Identity, Policy Manager, Browser Manager, Dashboard Manager, Telemetry, AI, Energy, Digital Twin, etc.

La prioridad del proyecto es crear una base estable (kernel, HAL, drivers, init, gestor de servicios e IPC) antes de construir servicios de alto nivel. Todos los servicios deben ejecutarse como servicios del sistema y ser descubiertos y administrados por un Service Manager común.

## Problema

Sin una decisión arquitectónica clara sobre las capas, el transporte IPC, el formato de mensajes, la gestión de secretos y el flujo de autorización, distintas implementaciones corren el riesgo de introducir dependencias prematuras, acoplamientos indebidos, y soluciones que no escalan cuando el sistema crezca.

Es necesario definir una arquitectura base, transportes IPC abstractos y contratos de servicio para permitir implementaciones independientes y sustituibles (por ejemplo: adaptar WebKit/Brave sin integrar la autenticación en el navegador).

## Constraints / Restricciones

- El sistema debe ser modular y soportar implementaciones de usuario (userland) para servicios críticos.
- No introducir llaves privadas ni secretos en el repositorio.
- Mantener independencia entre capas: kernel no depende de servicios de usuario; UI no accede hardware directamente.
- Inicialmente priorizar implementación local-first (host único) y permitir migración a modo distribuido más adelante.
- Las APIs y formatos deben versionarse para permitir evolución.

## Decisiones clave (resumen)

1. Capas principales (de abajo hacia arriba):
   - Hardware
   - Bootloader
   - Quantum Kernel (primitivas: scheduler, memoria, IPC primitives)
   - HAL (abstracción de MMIO, PCIe, I2C, SPI, GPIO)
   - Drivers (network, gpu, input, storage)
   - Filesystem
   - Quantum Init (init process que monta FS y arranca Service Manager)
   - Service Manager / Quantum Service Framework
   - Service Registry & Discovery
   - IPC Framework (RPC + Pub/Sub)
   - Framework de servicios: trait QuantumService (lifecycle)
   - Identity Service (RS256 + JWKS, refresh tokens, RBAC/ABAC)
   - Policy Manager (ABAC/OPA-compatible)
   - Browser Platform, Dashboard Platform, Wayland Framework, Desktop Editions

2. Autenticación/Token: RS256 en producción, JWKS expuesto por Identity Service; HS256 permitido solo en dev explicitamente.

3. IPC inicial: transporte abstracto con soporte para UDS (Unix Domain Sockets), Named Pipes, Shared Memory, Local TCP y otros. Implementación inicial: adaptador transport-agnóstico con backend UDS y opción para TCP/TLS.

4. Mensajes: sobre envelopes JSON con campos: version, trace_id, timestamp, from, to (service), type/event, auth_claims(optional), payload (typed), schema_version.

5. Políticas: Policy Manager central con motor ABAC; políticas expresadas en Rego-compatible (OPA) o en formato declarativo equivalente.

6. Observabilidad: tracing distribuido (trace_id per envelope), métricas Prometheus por servicio, logs estructurados (tracing).

## Alternativas consideradas

A. Usar un message broker (RabbitMQ/Kafka) desde el inicio
   - Ventajas: escalabilidad, features avanzadas de entrega
   - Desventajas: dependencia pesada, complejidad operativa inmediata, no necesario para single-host

B. Usar gRPC/TCP como único canal
   - Ventajas: estándar, rendimiento
   - Desventajas: gRPC no cubre bien pub/sub local sin broker; UDS tiene mejor latencia en local

C. Usar HS256 exclusivamente para tokens
   - Ventajas: implementación más simple
   - Desventajas: gestión de claves más arriesgada para multi-service; rotación y reparto de claves problemático

Decisión: comenzar con UDS-based transport local (por latencia y simplicidad) implementado a través de una capa de transporte abstracta que permita cambiar a TCP/TLS o broker más adelante; usar RS256/JWKS para tokens.

## Ventajas de la decisión

- Modularidad: servicios son reemplazables y sólo dependen de contratos.
- Seguridad: RS256 + JWKS permite rotación de claves sin compartir claves privadas.
- Evolutividad: la capa de transporte abstracta permite migrar a broker/distribuido sin reescribir la lógica de negocio.

## Desventajas

- Se necesita trabajo inicial para diseñar la capa de transporte abstracta e IPC.
- La complejidad operativa aumenta (gestión de sockets, permisos y secretos).

## Seguridad (consideraciones)

- Las claves privadas no deben estar en repositorio; usar Vault/KMS/secret manager en deployments.
- Fail‑fast: servicios críticos (Identity, Policy Manager) deben negarse a arrancar sin los secretos necesarios en modo producción.
- Socket permissions restrictivas y autenticación mutua para transporte cuando sea posible.
- Tokens cortos, refresh tokens rotativos y capacidad de revocación.

## Performance (consideraciones)

- UDS ofrece baja latencia para IPC local; la serialización JSON debe ser optimizada o sustituida por CBOR/msgpack si el throughput lo exige.
- Tracing y métricas deben ser asíncronos y de bajo overhead.
- Perfilado temprano recomendado para IPC hot-paths; posibilidad de usar shared memory / memory mapped channels para casos extremos.

## Future scalability

- La capa de transporte abstracta permite cambio a brokers (Kafka, RabbitMQ) o a RPC TLS para servicios distribuidos.
- JWKS permite verificación de tokens sin necesidad de llamadas síncronas a Identity Service.
- Policy Manager puede ser desplegado en modo centralizado o federado; la caché local de decisiones mitigará latencia.

## Final recommendation

Aprobar esta arquitectura base antes de comenzar implementaciones productivas. Después de la aprobación:

1. Crear los documentos formales (este ADR y el diagrama PlantUML) en `docs/adr/` y `docs/architecture/`.
2. Implementar una PoC del Service Framework (skeleton + UDS transport adapter) y pruebas de integración smoke‑test.
3. Implementar Identity Service en modo dev (HS256) y preparar RS256/JWKS para producción; no mergear keys privadas.
4. Implementar Policy Manager skeleton y definir el contrato de autorización.

---

### Open questions (deben resolverse antes de la implementación completa)

1. ¿Microkernel completo (mover drivers a userland) o monolito con módulos? Recomendación: microkernel-ish (userland drivers) para modularidad y seguridad, pero aceptar un roadmap que comience con un monolito reducido si recursos son limitados.
2. ¿Secret manager preferido en producción? (HashiCorp Vault recomendado)
3. ¿Intentar soporte distribuido desde día cero o diseñar local-first con paths de migración? Recomendación: local-first con migración plan.

---

Se solicita revisión y aprobación por los stakeholders arquitectónicos antes de avanzar a la fase de implementación.
