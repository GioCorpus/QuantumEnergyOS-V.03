# Guía de Construcción — QuantumEnergyOS V.03 ISO

> **Especificación Técnica de Compilación, Empaquetado y Generación de Imagen ISO**  
> **Versión del Documento:** 2.0.3  
> **Sistema Operativo Objetivo:** Linux x86_64  
> **Perfil de Arquitectura:** ArchISO / LiveCD / LiveUSB  
> **Estado:** Documento Estándar de Despliegue  

---

## 1. Resumen Ejecutivo y Arquitectura Generales

**QuantumEnergyOS V.03** es una distribución Linux especializada de alto rendimiento, diseñada para la orquestación híbrida de energía cuántico-fotónica, balanceo de redes climáticas y optimización de redes inteligentes (*smart grids*) en tiempo real. 

Esta guía establece el protocolo estandarizado de construcción para compilar los binarios nativos en Rust, configurar el entorno de orquestación en Python, empaquetar el panel de control web en React/TypeScript y ensamblar la imagen ejecutable **ISO (LiveCD/LiveUSB)** mediante el motor de construcción `mkarchiso`.

> ⚠️ **Nota Crítica de Seguridad y Permisos:**  
> La generación del espacio de nombres *chroot* y del sistema de archivos comprimido *SquashFS* requiere **privilegios de superusuario (`root` o `sudo`)** en un entorno anfitrión basado en Arch Linux o mediante contenedores privilegiados Docker.

---

## 2. Requisitos Previos del Sistema e Infraestructura

Antes de iniciar la tubería de compilación (*build pipeline*), el nodo anfitrión debe cumplir con las siguientes especificaciones técnicas de hardware y software:

### 2.1 Especificaciones de Hardware

| Recurso / Componente | Requisito Mínimo | Especificación Recomendada |
| :--- | :--- | :--- |
| **Arquitectura de CPU** | x86_64 con soporte de virtualización (VT-x / AMD-V) | Procesador multi-núcleo x86_64 (8+ núcleos físicos) |
| **Memoria RAM** | 8 GB RAM | 16 GB+ RAM (Optimiza compilación en RAM-disk) |
| **Almacenamiento Libre** | 25 GB en disco duro | 50 GB NVMe SSD (Para workspace dinámico) |

### 2.2 Herramientas de Software y Toolchains

* **Gestión de Entorno ISO:** `archiso`, `mkarchiso`, `git`
* **Entorno de Compilación Rust:** `rustc` 1.70+, `cargo`, toolchain estable `x86_64-unknown-linux-gnu`
* **Entorno Python & IA:** `python` 3.10+, `python-pip`, `python-numpy`, `python-scipy`
* **Entorno Web & Dashboard:** `nodejs` 18+ (LTS recomendado), `npm`
* **Virtualización & Prueba:** `qemu-system-x86_64`, `kvm`
* **Contenedores (Opcional):** `docker` o `podman`

---

## 3. Estructura del Repositorio de Código

El proceso de empaquetado ISO depende de la organización jerárquica de carpetas y archivos dentro del repositorio principal del proyecto:

```text
QuantumEnergyOS-V.03-main/
├── AppData/
│   └── Local/Programs/Microsoft VS Code/QuantumEnergyOS/
│       ├── buildsystem/
│       │   └── build.ps1             # Script de automatización para Windows/PowerShell
│       ├── climate_orchestrator/     # Módulo Python de orquestación térmica y climática
│       ├── livecd/
│       │   └── profiledef/
│       │       └── packages.x86_64   # Lista maestra de paquetes de Arch Linux para la ISO
│       ├── photonic-bridge/          # Interfaz de bajo nivel Rust para circuitos ópticos
│       ├── photonic-core/            # Núcleo de cálculo y optimización fotónica en Rust
│       ├── web-dashboard/            # Interfaz de usuario React / TypeScript
│       ├── Dockerfile                # Configuración de entorno de construcción aislado
│       └── Makefile                  # Tubería unificada de automatización en Linux
├── Build-ISO.ps1                     # Script principal de ejecución en PowerShell
├── Makefile                          # Punto de entrada principal para 'make iso'
└── GUIA-CONSTRUCCION-ISO.pdf         # Documentación técnica de referencia
```

---

## 4. Precompilación de Módulos y Ensamblado de Componentes

Antes de inyectar las aplicaciones en la plantilla del sistema de archivos raíz (`airootfs`), es obligatorio realizar la compilación previa de cada servicio.

### 4.1 Compilación de los Núcleos Fotónicos (Rust)

Compilación en modo *Release* con optimizaciones de rendimiento a nivel de microarquitectura:

```bash
# 1. Compilar el optimizador fotónico (Photonic Core)
cd photonic-core
cargo build --release --target x86_64-unknown-linux-gnu

# 2. Compilar el puente de enlace óptico (Photonic Bridge)
cd ../photonic-bridge
cargo build --release --target x86_64-unknown-linux-gnu
```

### 4.2 Empaquetado del Dashboard Web (React & TypeScript)

Generación de los artefactos estáticos de la interfaz web de monitoreo:

```bash
cd ../web-dashboard
npm install
npm run build
```

---

## 5. Configuración del Perfil ISO (`livecd/profiledef`)

### 5.1 Declaración de Dependencias (`packages.x86_64`)

Edite o verifique el archivo `livecd/profiledef/packages.x86_64` para incluir las dependencias requeridas por el runtime de la ISO:

```ini
# Sistema Base y Kernel
linux
linux-firmware
base
base-devel

# Red y Comunicaciones
dhcpcd
iwd
networkmanager
openssh
curl
git

# Runtimes y Librerías de QuantumEnergyOS
python
python-pip
python-numpy
python-scipy
rust
hwloc
opencl-icd-loader
```

### 5.2 Inyección de Binarios y Artefactos en la Plantilla LiveCD

Copie los binarios precompilados dentro de la estructura `airootfs` del perfil `livecd`:

```bash
# Crear directorios de destino dentro del sistema de archivos de la ISO
mkdir -p livecd/airootfs/usr/local/bin
mkdir -p livecd/airootfs/var/www/quantum-dashboard

# Copiar ejecutable binario de Rust
cp photonic-core/target/release/photonic-core livecd/airootfs/usr/local/bin/
cp photonic-bridge/target/release/photonic-bridge livecd/airootfs/usr/local/bin/

# Copiar artefactos frontend compilados
cp -r web-dashboard/build/* livecd/airootfs/var/www/quantum-dashboard/
```

---

## 6. Ejecución del Proceso de Construcción de la Imagen ISO

Seleccione el flujo de ejecución que mejor se adapte a su entorno de desarrollo:

### Opción A: Automatización vía Makefile (Recomendado para Linux / Arch)

```bash
# Limpiar entornos previos
make clean

# Iniciar compilación completa y construcción de ISO
make iso
```

### Opción B: Ejecución Manual con `mkarchiso` (Línea de Comandos)

```bash
# Crear el directorio de salida
mkdir -p out/

# Ejecutar el motor mkarchiso indicando la carpeta de trabajo y perfil
sudo mkarchiso -v -w work/ -o out/ livecd/profiledef/
```

> **Resultado Esperado:**  
> Al finalizar el proceso correctamente, la imagen ejecutable estará disponible en:  
> `out/QuantumEnergyOS-v0.03-x86_64.iso`

### Opción C: Construcción en Contenedor (Docker / Entorno Aislado)

Para construir la ISO en distribuciones que no son Arch Linux (Ubuntu, Debian, Fedora, etc.):

```bash
# 1. Construir la imagen contenedora de compilación
docker build -t quantum-iso-builder -f Dockerfile .

# 2. Ejecutar el contenedor en modo privilegiado para generar el archivo .iso
docker run --privileged -v $(pwd)/out:/build/out quantum-iso-builder
```

---

## 7. Verificación, Pruebas y Validación en Entorno Virtual

Antes de grabar la imagen en un medio físico (USB/DVD), valide el arranque y el correcto funcionamiento de los servicios utilizando QEMU con aceleración KVM:

```bash
qemu-system-x86_64     -enable-kvm     -m 4096     -smp 4     -cdrom out/QuantumEnergyOS-v0.03-x86_64.iso     -boot d     -vga virtio     -net nic,model=virtio     -net user,hostfwd=tcp::8080-:80
```

---

## 8. Cumplimiento de Estándares ISO y Calidad de Software

El proceso de empaquetado y los componentes integrados en **QuantumEnergyOS V.03** están alineados con normativas internacionales de la industria:

* **ISO/IEC 27001 (Seguridad de la Información):** Aislamiento de permisos de superusuario, validación de integridad criptográfica y purga estricta de claves privadas predeterminadas.
* **ISO 9001 (Gestión de la Calidad):** Proceso de compilación reproducible mediante scripts automatizados (`Makefile` y `mkarchiso`).
* **ISO 14001 (Gestión Ambiental):** Algoritmos de optimización integrados en `photonic-core` diseñados para reducir la huella de carbono de los centros de datos.
* **ISO/IEC 25010 (Calidad de Software):** Garantías de seguridad de memoria nativa provistas por el código fuente en Rust.

---

## 9. Matriz de Solución de Problemas (Troubleshooting)

| Causa / Falla | Causa Raíz Probable | Procedimiento de Solución |
| :--- | :--- | :--- |
| `pacman lock file error` | Cancelación abrupta de un *build* previo que dejó el archivo de bloqueo activo. | Ejecute: `sudo rm -f work/x86_64/airootfs/var/lib/pacman/db.lck` y reinicie el proceso. |
| `No space left on device` | Espacio insuficiente en el directorio temporal `/tmp` o en el directorio `work/`. | Redirija el directorio de trabajo a una partición con 30GB+ de espacio libre usando la bandera `-w /ruta/al/disco`. |
| Error de librería dinámica (`libopencl.so`) | Omisión de dependencias en el archivo de perfil. | Añada el paquete faltante en `livecd/profiledef/packages.x86_64` y vuelva a compilar. |
| Fallo en la arquitectura objetivo de Rust | Incompatibilidad de *target triple* entre la máquina anfitriona y la ISO. | Asegúrese de especificar el *target* exacto: `cargo build --target x86_64-unknown-linux-gnu --release`. |

---

> 🔒 **Advertencia Final de Despliegue:** Asegúrese de reemplazar o purgar cualquier archivo `.env`, llaves privadas SSH de prueba o certificados temporales antes de distribuir comercial u oficialmente la imagen ISO final.
