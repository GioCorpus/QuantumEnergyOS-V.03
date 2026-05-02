# Contribuir a QuantumEnergyOS

¡Gracias por considerar contribuir a **QuantumEnergyOS**!  
Su participación es fundamental para avanzar en el desarrollo de la primera plataforma cuántica-fotónica dedicada a la optimización energética desde Mexicali, Baja California.

---

## Código de Conducta

Este proyecto se rige por el [Código de Conducta](CODE_OF_CONDUCT.md). Esperamos que todos los participantes mantengan un ambiente respetuoso, inclusivo y profesional.

---

## Cómo Contribuir

### 1. Reportar un problema (Issues)

- Antes de crear un nuevo issue, verifique que no exista ya uno similar.
- Utilice un título claro y descriptivo.
- Incluya:
  - Descripción detallada del problema o sugerencia.
  - Pasos para reproducir el error (si aplica).
  - Versión del sistema operativo y entorno utilizado.
  - Capturas de pantalla o logs relevantes.

### 2. Proponer cambios (Pull Requests)

1. **Fork** el repositorio.
2. Cree una rama con un nombre descriptivo:
   ```bash
   git checkout -b feature/qaoa-improvement
   # o
   git checkout -b bugfix/dashboard-translation

   Realice sus cambios siguiendo las convenciones del proyecto.
Asegúrese de que las pruebas existentes sigan pasando (make test o pytest).
Actualice la documentación si es necesario.
Envíe un Pull Request con una descripción clara del cambio realizado.

3. Estándares de Código

Rust: Seguir las convenciones de rustfmt y clippy.
Python: Cumplir con PEP 8 y utilizar ruff para linting.
TypeScript/React: Seguir las reglas de ESLint y Prettier configuradas en el dashboard.
Q#: Mantener un estilo limpio y bien comentado.
Todos los commits deben tener mensajes claros y en inglés (preferentemente).


Áreas donde más se necesita ayuda
Actualmente buscamos contribuciones prioritarias en las siguientes áreas:
🔬 Algoritmos Cuánticos

Mejora de algoritmos Q# (VQE, QAOA y sus variantes)
Implementación de nuevas optimizaciones para balanceo de red eléctrica
Algoritmos de corrección de errores (GKP, surface codes, etc.)

🐍 Integración con Hardware Cuántico

Soporte para más backends de Qiskit (IonQ, Quantinuum, Rigetti, etc.)
Optimización de ejecución en simuladores y hardware real
Manejo de sesiones y tokens de forma segura

🌐 Dashboard y Localización

Traducción completa al inglés del dashboard (alta prioridad)
Mejoras en la interfaz React + TypeScript
Optimización de rendimiento y accesibilidad

🧪 Pruebas e Integración

Tests de integración con hardware cuántico real
Pruebas de carga y estrés del sistema
Validación de resultados contra simuladores clásicos de alta fidelidad

📚 Documentación

Documentación técnica detallada de la arquitectura fotónica
Diagramas de flujo del PhotonicQ Bridge
Guías de despliegue y configuración avanzada
Documentación de la API qcall


Configuración de Desarrollo
Consulte la sección Desarrollo del README.md para instrucciones detalladas de instalación y configuración del entorno completo (Rust, Python, Q#, Node.js, etc.).

Preguntas o Dudas

Abra un issue con la etiqueta question o help-wanted.
Puede contactar directamente al mantenedor a través de GitHub Discussions.


Agradecimientos
Toda contribución, por pequeña que sea, es valiosa. Los contribuyentes aparecerán en la sección de Agradecimientos del README.

QuantumEnergyOS — Llevando computación cuántica aplicada desde el desierto de Mexicali al mundo.
Gracias por formar parte de esta misión.
