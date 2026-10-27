# JaviPiV2

Nuevo administrador **modular, sencillo y reinstalable** de Raspberry Pi con DietPi, construido principalmente en Rust.

> **Estado: F0 / prototipo.** Este repositorio no instala nada todavía y no reemplaza ni modifica el JaviPi existente. El catálogo enumera funcionalidades planificadas, no componentes ya disponibles.

## Primera vista funcional

Con Rust instalado (se requiere Rust >= 1.99), en un entorno de desarrollo:

```bash
cargo run --locked -- help
cargo run --locked -- status
cargo run --locked -- module list
cargo run --locked -- plan minimal
cargo run --locked -- preflight minimal
cargo run --locked -- simulate minimal
cargo test --locked
```

`preflight minimal` consulta `ID=dietpi` en `/etc/os-release` o un archivo no vacío y legible en `/boot/dietpi/.version` como evidencia de DietPi; si no hay evidencia, informa desconocido. La arquitectura es la del binario, no una comprobación independiente del host. Ordena módulos y dependencias sin inspeccionar rutas de almacenamiento ni ejecutables; capacidad, presencia y acciones son desconocidas. No es un plan exacto de instalación ni implementa idempotencia o reanudación. La opción `--fixture` lee como máximo 1024 bytes y tres líneas de stdin (`dietpi`/`unknown`/`debian`/`debian-marker`, `aarch64`/`unknown`, `mounted`/`unknown`) como datos simulados, no evidencia del host. `debian-marker` simula un marcador legible y no vacío.

`simulate minimal` reutiliza el preflight y enumera los módulos en el orden del perfil; además, usa `MinimalSimulation` para mostrar el progreso y resultado exclusivamente simulados. Identifica el código disponible en este programa, la funcionalidad pendiente y la presencia instalada desconocida; las acciones de instalación siguen sin definir. `simulate minimal --fixture` acepta el mismo formato simulado; repetirlo no escribe estado ni instala nada. La capacidad sigue desconocida; una futura operación que la necesite debe comprobarla de forma segura o rechazarse sin modificar datos. No hay todavía plan exacto, idempotencia ni reanudación; F1 sigue incompleta.

La biblioteca también ofrece `core::minimal_simulation::MinimalSimulation`: `new()` prepara el plan, `step()` simula una operación, `checkpoint()` devuelve bytes y `from_checkpoint(&bytes)` valida y recupera el progreso; `run()` termina las pendientes y `report()` muestra resultados simulados. El coordinador no abre rutas ni persiste esos bytes por sí mismo. El llamador es responsable de guardarlos en almacenamiento aislado y seguro; la API de archivos heredada de F1-C acepta rutas y no es una barrera de seguridad para rutas no confiables. La CLI `simulate minimal` utiliza este coordinador en memoria y conserva su preflight de solo lectura; no guarda ni recupera checkpoints.

`status` solo lee `/proc` (hostname, uptime y memoria). No ejecuta subprocesses, no requiere `sudo`, no consulta discos montados ni contacta servicios.

## Convivencia durante el desarrollo

JaviPiV2 es un programa independiente y no instala binarios, servicios ni configuración global. Cuando el operador autorice la ejecución, se podrá compilar en el checkout con `cargo build --locked` y lanzar manualmente `./target/debug/javipiv2 simulate minimal` como usuario normal. PAR-1 y sus pruebas quedan pendientes de ejecución y validación; no se afirma compatibilidad operativa con el sistema productivo.

Las rutas previstas `/etc/javipiv2` y `/var/lib/javipiv2`, así como futuros nombres de servicio y puertos independientes, son objetivos de diseño: este incremento no los crea ni configura. No se modifica el JaviPi original.

## Primer paso hacia el instalador

```bash
bash installer/bootstrap.sh --help
bash installer/bootstrap.sh --check
bash installer/bootstrap.sh --plan developer
```

El bootstrap **solo inspecciona y presenta perfiles**. No descarga ni instala paquetes, no escribe en `/etc`, no toca `/srv`, no prepara particiones y no usa `sudo`. La instalación automatizada está planificada para F1, no existe aún.

Los perfiles están en `profiles/`, y el catálogo único de módulos está en `catalog/modules.tsv`. El mismo catálogo se compila en Rust para el CLI. Las 21 áreas cubren las funcionalidades del proyecto original, pero casi todas siguen en estado `planned`.

## Alcance y diseño

- Un paquete Cargo con módulos internos. Se crean subcrates solo si surge una necesidad real.
- CLI, futura TUI, API y Telegram reutilizarán las mismas operaciones.
- Permisos Linux y herramientas existentes; sin protocolo criptográfico propio.
- Instalar, habilitar, deshabilitar y retirar módulos serán operaciones distintas.
- Instalar en DietPi recién formateado y restaurar datos/configuración es un objetivo central.
- Datos del HDD y accesos SSH/Tailscale preservados durante desarrollo y migración.

Consulta [PLAN.md](PLAN.md), [instalación](docs/INSTALL.md) y [restauración](docs/RECOVERY.md).

**Este repositorio es público:** nunca subir tokens, claves, contraseñas ni configuraciones personales reales.