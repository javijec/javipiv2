# JaviPiV2

Nuevo administrador **modular, sencillo y reinstalable** de Raspberry Pi con DietPi, construido principalmente en Rust.

> **Estado: F0 / prototipo.** Este repositorio no instala nada todavía y no reemplaza ni modifica el JaviPi existente. El catálogo enumera funcionalidades planificadas, no componentes ya disponibles.

## Primera vista funcional

Con Rust instalado (se requiere Rust >= 1.99), en un entorno de desarrollo:

```bash
cargo run --locked -- help
cargo run --locked -- status
cargo run --locked -- storage status
cargo run --locked -- updates check
cargo run --locked -- developer compare 1.2.3 1.3.0
cargo run --locked -- developer status
cargo run --locked -- agents status
cargo run --locked -- remote status
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

`service plan <start|stop|restart> <unidad.service> --allow <unidad.service>` describe una operación y unidad exactas, sin ejecutarlas ni consultar systemd. Ejemplo: `javipiv2 service plan restart ejemplo.service --allow ejemplo.service`. Sin `--allow` se rechaza el plan; la CLI autoriza una sola unidad por consulta y la biblioteca admite una lista configurable, vacía por defecto. Esta autorización solo permite crear planes, no concede permisos de ejecución ni acredita que la unidad exista o pertenezca a JaviPi. Un `ServicePlan` es una descripción, no una prueba de autorización; cualquier ejecución futura deberá validar y autorizar de nuevo. Se aceptan nombres ASCII conservadores con sufijo `.service` e instancias no vacías, sin escapes ni rutas. F3-A y sus pruebas están pendientes de compilación y validación; no hay controles administrativos en la TUI ni ejecución implementada.

F3-B prepara un ejecutor con argumentos separados y una política independiente que rechaza todos los planes, incluidos los autorizados con `--allow`, antes de invocar systemctl o un ejecutor inyectado. No existe activación real ni registro de operaciones reales. Un código de salida exitoso no demuestra el estado del servicio; antes de habilitar acciones harán falta unidades administradas explícitas, autorización expresa y comprobación del resultado.

`docker status`, `docker list` y `docker inspect <contenedor>` consultan únicamente el daemon local mediante el cliente Docker, sin shell ni acciones de administración. List e inspect muestran solo nombre, ID abreviado, imagen y estado. Antes de conectar se comprueba que `docker.service` esté cargado y activo, y que `docker.socket` esté cargado (activo o inactivo) o no exista (`not-found`); estados desconocidos, errores y otros estados se rechazan. **Esta comprobación no elimina la ventana TOCTOU:** systemd podría cambiar el estado del servicio o activar el socket después de la comprobación y antes de conectar; no ofrece garantía frente a esa carrera. Las consultas requieren acceso autorizado al socket y no modifican permisos. `status`, `list` y la vista Docker de la TUI se comprobaron en el sistema real; `inspect` real no se ejecutó. No se usan contextos ni configuraciones Docker heredados.

`project list` y `project status <nombre>` consultan carpetas directas de `/srv/workspaces`; `JAVIPIV2_WORKSPACES` permite configurar otra ruta absoluta bajo `/srv`. Antes de abrirla se exige evidencia pasiva en `/proc/self/mountinfo` de ext4 con raíz `/` en `/srv`, solo o directamente sobre una capa autofs. Se rechazan montajes que afecten la ruta consultada: para `list`, todo el árbol de workspaces; para `status`, sus antecesores y el árbol del proyecto elegido. Montajes ajenos como `/srv/docker` no bloquean la consulta. Si falta evidencia se informa `unknown` y no se recorre `/srv`. No demuestra la identidad física del disco ni garantiza excluir bind mounts; la comprobación tiene una ventana de carrera frente a cambios de montaje. Las operaciones futuras con efectos requieren comprobar identidad. No se consulta capacidad. Se leen directamente metadatos Git sin ejecutar Git ni shell, sin seguir enlaces; worktrees con archivo `.git`, metadatos inaccesibles y formatos no reconocidos quedan desconocidos. La categoría `compatible (validez no verificada)` solo indica metadatos con forma reconocible: no verifica objetos, integridad ni validez real del repositorio. Una referencia suelta inaccesible o inválida no se sustituye por un valor posiblemente obsoleto de `packed-refs`. Los nombres de carpetas no UTF-8 provocan un error `unknown` en lugar de omitirlas. No se consultan cambios, remotos ni sincronización. Las pruebas sintéticas de F2-D pasaron; su funcionamiento con proyectos reales sigue pendiente de validación.

`storage status` primero comprueba `/proc/self/mountinfo` y solo consulta capacidad si confirma inequívocamente ext4 activo en `/srv`. Informa total, usado y disponible; disponible excluye los bloques reservados, que se muestran aparte. Si mountinfo es ambiguo o falla, o falla la consulta de capacidad, se muestra `desconocido` y no se reintenta. La consulta de estadísticas de `/srv` puede activar autofs: existe una ventana TOCTOU entre la instantánea de mountinfo y la consulta, por lo que el montaje puede cambiar durante ese intervalo. No monta dispositivos explícitamente ni modifica datos. También informa overlays Docker reconocidos por ruta y no clasifica otros montajes como datos. No verifica identidad física, exclusión de bind mounts, salud, UUID ni SMART, y no administra discos. La vista Storage de la TUI reutiliza esta consulta al pulsar R; errores internos no se exponen.

## Comparación Developer offline

`developer compare <instalada> <referencia>` compara exclusivamente dos valores suministrados por el usuario, no versiones instaladas ni disponibilidad de actualizaciones. Admite exactamente tres componentes numéricos ASCII separados por puntos, de 1 a 9 dígitos cada uno; entradas no admitidas producen `desconocido`. No abre rutas, usa red ni ejecuta herramientas. No se agrega vista TUI: la operación requiere dos argumentos explícitos y la interfaz actual no ofrece entrada de texto para ellos.

`developer status` y la vista Developer de la TUI (consulta manual con R) reutilizan la observación local de Agents y sus metadatos npm de Updates, sin red ni ejecutar herramientas. Cada elemento separa la evidencia de ruta y bits de ejecución de la versión procedente de metadatos npm locales, que no está vinculada al ejecutable. Los bits no garantizan acceso ni disponibilidad efectiva; esta y el estado de ejecución permanecen desconocidos. Se aplican las mismas rutas y limitaciones descritas para `agents status`; no se agregan detectores ni consultas a `/srv`.

## Evaluación Watchdog

`watchdog memory <total-kib> <available-kib> <minimum-percent>` evalúa únicamente valores suministrados por el usuario, no monitorea memoria real. Son enteros ASCII de hasta 20 dígitos dentro de `u64`; el total debe ser positivo, el disponible no puede superarlo y el mínimo debe estar entre 0 y 100. La igualdad con el mínimo es saludable; valores inválidos son `desconocido`. Este comando no tiene valores predeterminados, programación, notificaciones ni lecturas del sistema, servicios o red.

`watchdog status <minimum-percent>` consulta una vez la memoria real mediante Health y evalúa el umbral explícito del operador con las mismas reglas. Si falla Health o los datos o el umbral son inválidos, muestra `desconocido` sin exponer el error. Health también lee hostname y uptime de `/proc`; no hay umbral predeterminado, supervisión periódica ni acciones automáticas. La vista Watchdog de la TUI consulta solo al pulsar R y usa un umbral fijo ilustrativo de 20% de memoria disponible, exclusivamente para mostrar el resultado; no es diagnóstico ni evalúa la salud general. La CLI permite elegir cualquier umbral válido entre 0 y 100. No hay acciones ni monitoreo periódico.

## Consulta de componentes

`updates check` compara seis componentes del gestor original: Codex, Pi, Gentle Shell, Paseo, Gentle AI y Engram. La consulta requiere red y usa `/usr/bin/curl` con configuración y entorno vacíos, HTTPS, sin proxy ni redirecciones, y límites de tiempo y tamaño; solo contacta los endpoints públicos fijos de npm y GitHub. Una redirección de GitHub se trata como desconocida, no se sigue. No usa npm, no instala paquetes, no actualiza índices y excluye APT. La versión instalada de los cuatro paquetes npm se busca únicamente en los `package.json` bajo `/usr/local/lib/node_modules`, `/usr/lib/node_modules`, `/home/dietpi/.pi/agent/npm/node_modules` y `/home/dietpi/.npm-global/lib/node_modules`, sin seguir enlaces y con lectura limitada. Se consultan todos los candidatos accesibles: versiones numéricamente distintas o metadatos inválidos dan `desconocido`; candidatos ausentes o inaccesibles se omiten, por lo que no se garantiza detectar conflictos ocultos. Otras ubicaciones y los binarios independientes Gentle AI y Engram quedan desconocidos; no se ejecutan binarios para detectar versiones. No se leen configuraciones ni credenciales. Solo se admiten versiones numéricas `x.y.z` (con `v` opcional); versiones preliminares, errores, archivos inaccesibles y respuestas inesperadas quedan desconocidos. Las versiones admitidas se comparan numéricamente: se informa `actual`, `actualización disponible` o `instalada más reciente`; no se infiere el orden de versiones no admitidas. La vista Updates de la TUI consulta únicamente al pulsar R y reutiliza la misma operación. En ARM64 pasaron las pruebas sintéticas y se comprobó la consulta CLI real; el operador confirmó el funcionamiento de la TUI manualmente. La comprobación automatizada en pseudoterminal no confirmó la selección de la vista y no llegó a solicitar la consulta; su resultado no establece un fallo de la aplicación.

`agents status` observa Pi Agent, Gentle Shell, Codex y Engram sin ejecutar agentes ni usar red. Inspecciona metadatos de rutas fijas bajo `/usr/local/bin`, `/usr/bin` y `/home/dietpi/.local/bin`, además de `/home/dietpi/.pi/agent/bin/pi`. Solo admite dos enlaces finales exactos de `/home/dietpi/.local/bin`: `pi` hacia la ruta fija de Pi y `gentle-shell` hacia `/home/dietpi/.gentle-shell/bin/gentle-shell`; los demás enlaces no se siguen. Para Pi y Gentle Shell se priorizan esas rutas sin afirmar precedencia completa del PATH; en los demás casos varios candidatos, errores o enlaces dejan el ejecutable desconocido. No se inspeccionan destinos de agentes bajo `/srv`. Un archivo regular con bits de ejecución es evidencia, no garantía de acceso. La observación puede cambiar después de la consulta. Reutiliza la búsqueda de versiones npm de Updates: candidatos ausentes o inaccesibles se omiten y pueden ocultar conflictos. La versión npm no se vincula al ejecutable; Engram no tiene fuente de versión local autorizada. El estado de ejecución siempre es desconocido. La vista Agents de la TUI consulta manualmente con R. La CLI se comprobó con metadatos reales; la captura automatizada de TUI no observó la ruta de Pi y no permite determinar la causa. Las pruebas sintéticas de TUI pasaron y el operador confirmó posteriormente la validación manual satisfactoria de Agents-C.

`remote status` consulta `ssh.service` y `tailscaled.service` con `systemctl show`, sin red ni acciones administrativas. Observa pasivamente `/proc/net/tcp` y `/proc/net/tcp6`, con lectura limitada, y solo informa si existe algún TCP en escucha, sin atribuirlo a SSH ni mostrar direcciones, puertos o pares. Errores o datos incompletos dejan ese dato desconocido; la escucha y conectividad SSH permanecen desconocidas. La consulta GET de LocalAPI de Tailscale es de lectura, pero no se pudo demostrar que conectar al socket nunca active un servicio; por eso no se conecta ni ejecuta el cliente y su conectividad queda desconocida. Muestra carga y actividad por separado; errores y datos ausentes son desconocidos. `not-found` indica que systemd no encontró la unidad, no que el software esté ausente. El estado de un servicio no demuestra conectividad. La vista Remote consulta manualmente con R usando el mismo módulo. El usuario confirmó que Remote-A está implementado y validado, y confirmó la validación manual de Remote-B.

## Snapshots suministrados por stdin

`javipiv2 backup inspect --stdin` interpreta un array JSON de `restic snapshots --json` suministrado explícitamente (máximo 4 MiB). No ejecuta Restic ni abre rutas o configuración. Reconoce tags `profile:full`, `profile:projects` y `profile:engram`; fechas ausentes o inválidas hacen desconocido el último snapshot. El último intento y su resultado no se pueden deducir del listado. Backups-B queda pospuesto por falta de una fuente fiable autorizada; no se agregan lectores.

Ejemplo sintético:

```text
Entrada: [{"time":"2024-01-01T00:00:00Z","id":"abcdef012345","tags":["profile:full"]}]
Salida:
Snapshots: 1
Fecha: 2024-01-01T00:00:00Z
ID: abcdef01
Perfil: full
Último intento: desconocido
Resultado del último intento: desconocido
```

## TUI inicial (pendiente de validación)

`javipiv2 tui` presenta Health, Services, Docker, Projects, Storage, Updates, Agents, Remote, Watchdog y Developer usando los módulos existentes. En el menú, flechas seleccionan y Enter abre una vista; el menú se desplaza para mantener visible la selección cuando hay espacio para filas. Tab alterna el foco entre menú y listado. En el listado, flechas seleccionan una fila y Enter abre detalles; Esc regresa al listado, después al menú y finalmente sale. Q sale en cualquier contexto. R solicita una consulta; seleccionar o abrir detalles no consulta. `j`/`k` desplazan verticalmente y `h`/`l` horizontalmente. Los detalles reutilizan la última consulta completada: Services muestra carga, actividad, subestado y categoría; Docker, identificador, imagen y estado; Projects, evidencia Git, rama y HEAD. Health se consulta al abrir; Docker, Projects, Storage, Updates, Agents, Remote, Watchdog y Developer requieren abrir su vista y pulsar R. Storage muestra la evidencia confirmada de autofs, el montaje ext4 y el conteo de overlays Docker; la información desconocida se presenta como `desconocido`. No hay controles administrativos ni polling continuo. No acceder a proyectos productivos durante el desarrollo.

El operador confirmó la revisión manual satisfactoria de las vistas Watchdog, Developer y Overview.

Overview presenta únicamente los resultados ya almacenados: `no consultado`, `desconocido`, `disponible` (hay evidencia, no implica instalación ni salud) y `error`, con aviso de datos anteriores potencialmente desactualizados. Entrar y pulsar R en Overview no consultan ni crean un hilo. Health disponible indica una lectura de estado exitosa; Remote disponible indica evidencia almacenada de carga, actividad o TCP, incluso inactividad o ausencia de escucha. No implica salud ni conectividad; sin evidencia se muestra desconocido. Las vistas originales conservan sus detalles.

Las consultas corren en un único hilo: tras diez segundos se muestra un aviso, pero no se cancela la operación subyacente; no se admiten nuevas consultas hasta que termine; una solicitud rechazada no sobrescribe los resultados de la vista actual. Durante una actualización se conserva la última respuesta y se identifica como anterior; los avisos de demora y errores permanecen asociados a su vista. Una actualización exitosa cierra el detalle abierto para evitar confundir elementos reordenados. Los avisos y categorías de error aparecen al final del contenido y se recorren con `j`/`k` y `h`/`l`; no se muestra stderr ni el texto original de errores. Con foco en la tabla el desplazamiento vertical mantiene visible la selección; usá Tab para volver al foco del menú y recorrer los avisos. Services recuerda que las unidades conocidas por systemd no son un inventario completo. Salir no espera al hilo. Docker conserva su preflight y su ventana de carrera con `docker.socket`; Projects conserva el preflight de `/srv` y la validación de rutas. El manejo interactivo de terminal y las consultas reales de la vista Storage siguen pendientes de validación manual; la compilación y las pruebas automáticas pasaron en ARM64.

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