# JaviPiV2

Nuevo administrador **modular, sencillo y reinstalable** de Raspberry Pi con DietPi, construido principalmente en Rust.

> **Estado: F0 / prototipo.** Este repositorio no instala nada todavía y no reemplaza ni modifica el JaviPi existente. El catálogo enumera funcionalidades planificadas, no componentes ya disponibles.

## Primera vista funcional

Con Rust instalado (se requiere Rust >= 1.99), en un entorno de desarrollo:

```bash
cargo run --locked -- help
cargo run --locked -- status
cargo run --locked -- service list
cargo run --locked -- service status ssh
cargo run --locked -- docker status
cargo run --locked -- docker list
cargo run --locked -- docker inspect ejemplo
cargo run --locked -- module list
cargo run --locked -- plan minimal
cargo run --locked -- preflight minimal
cargo run --locked -- simulate minimal
cargo test --locked
```

`preflight minimal` consulta `ID=dietpi` en `/etc/os-release` o un archivo no vacío y legible en `/boot/dietpi/.version` como evidencia de DietPi; si no hay evidencia, informa desconocido. La arquitectura es la del binario, no una comprobación independiente del host. Ordena módulos y dependencias sin inspeccionar rutas de almacenamiento ni ejecutables; capacidad, presencia y acciones son desconocidas. No es un plan exacto de instalación ni implementa idempotencia o reanudación. La opción `--fixture` lee como máximo 1024 bytes y tres líneas de stdin (`dietpi`/`unknown`/`debian`/`debian-marker`, `aarch64`/`unknown`, `mounted`/`unknown`) como datos simulados, no evidencia del host. `debian-marker` simula un marcador legible y no vacío.

`simulate minimal` reutiliza el preflight y enumera los módulos en el orden del perfil; además, usa `MinimalSimulation` para mostrar el progreso y resultado exclusivamente simulados. Identifica el código disponible en este programa, la funcionalidad pendiente y la presencia instalada desconocida; las acciones de instalación siguen sin definir. `simulate minimal --fixture` acepta el mismo formato simulado; repetirlo no escribe estado ni instala nada. La capacidad sigue desconocida; una futura operación que la necesite debe comprobarla de forma segura o rechazarse sin modificar datos. No hay todavía plan exacto, idempotencia ni reanudación; F1 sigue incompleta.

La biblioteca también ofrece `core::minimal_simulation::MinimalSimulation`: `new()` prepara el plan, `step()` simula una operación, `checkpoint()` devuelve bytes y `from_checkpoint(&bytes)` valida y recupera el progreso; `run()` termina las pendientes y `report()` muestra resultados simulados. El coordinador no abre rutas ni persiste esos bytes por sí mismo. El llamador es responsable de guardarlos en almacenamiento aislado y seguro; la API de archivos heredada de F1-C acepta rutas y no es una barrera de seguridad para rutas no confiables. La CLI `simulate minimal` utiliza este coordinador en memoria y conserva su preflight de solo lectura; no guarda ni recupera checkpoints.

`status` lee hostname, uptime, memoria y promedios de carga de `/proc`, y busca la temperatura de CPU en `/sys/class/thermal`. Solo identifica una zona única de tipo `cpu-thermal`; si no hay evidencia suficiente o falla la lectura, muestra el dato como desconocido. Los promedios de carga no son porcentajes de uso de CPU. No ejecuta subprocesses, no requiere `sudo`, no consulta discos montados ni contacta servicios. F2-A está pendiente de ejecución y validación.

`service list` consulta las unidades de servicio conocidas por systemd mediante `systemctl list-units --all`; no es un inventario completo de todas las unidades instaladas. `service status <nombre>` consulta una unidad concreta con `systemctl show`; se puede omitir el sufijo `.service`. Ambos muestran carga, actividad, subestado y una categoría: activo, inactivo, fallido o desconocido. Los valores originales de systemd se conservan; una unidad no encontrada, un estado distinto o información insuficiente no se clasifican como detenidos. Los errores de consulta se informan como errores. No se modifican servicios. F2-B y su comportamiento real están pendientes de validación.

`docker status`, `docker list` y `docker inspect <contenedor>` consultan únicamente el daemon local mediante el cliente Docker, sin shell ni acciones de administración. List e inspect muestran solo nombre, ID abreviado, imagen y estado. Antes de conectar se comprueba que `docker.service` esté activo y que `docker.socket` no esté activo; si no se puede determinar, se rechaza la consulta. **Esta comprobación no elimina la ventana de carrera:** systemd podría cambiar de estado antes de la conexión y activar el socket. Las consultas requieren acceso autorizado al socket y F2-C permanece sin validar; no modifican permisos. No se usan contextos ni configuraciones Docker heredados.

`project list` y `project status <nombre>` consultan carpetas directas de `/srv/workspaces`; `JAVIPIV2_WORKSPACES` permite configurar otra ruta absoluta bajo `/srv`. Antes de abrirla se exige evidencia pasiva en mountinfo de un único montaje ext4 con raíz `/` en `/srv`, sin montajes descendientes. Si falta evidencia se informa `unknown` y no se recorre `/srv`. No demuestra la identidad física del disco ni garantiza excluir bind mounts; la comprobación tiene una ventana de carrera frente a cambios de montaje. Las operaciones futuras con efectos requieren comprobar identidad. No se consulta capacidad. Se leen directamente metadatos Git sin ejecutar Git ni shell, sin seguir enlaces; worktrees con archivo `.git`, metadatos inaccesibles y formatos no reconocidos quedan desconocidos. La categoría `compatible (validez no verificada)` solo indica metadatos con forma reconocible: no verifica objetos, integridad ni validez real del repositorio. Una referencia suelta inaccesible o inválida no se sustituye por un valor posiblemente obsoleto de `packed-refs`. Los nombres de carpetas no UTF-8 provocan un error `unknown` en lugar de omitirlas. No se consultan cambios, remotos ni sincronización. F2-D y sus pruebas están pendientes de ejecución y validación.

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