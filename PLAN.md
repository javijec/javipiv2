# JaviPiV2 — Plan maestro

**Objetivo:** reemplazar JaviPi anterior con un administrador personal completo para Raspberry Pi / DietPi ARM64: sencillo, modular, reinstalable y recuperable.

## Principios

- Proyecto independiente; no fusionar ramas ni incorporar automáticamente las arquitecturas Rust previas.
- Un paquete Rust inicial y módulos internos por responsabilidad; separar crates solo cuando haya un beneficio comprobable.
- Un motor compartido para CLI, Ratatui, Dashboard y Telegram, sin backends duplicados.
- Integrar software probado: Linux/systemd, Docker/Compose, SSH/Tailscale, Restic, Git, Codex, Pi, Gentle Shell y Engram.
- No inventar protocolos criptográficos ni supervisores de procesos genéricos.
- Consultas sin efectos secundarios; acciones tipadas, privilegios mínimos, validación y confirmación cuando corresponda.
- Instalar != habilitar; deshabilitar != desinstalar; eliminar código != borrar datos.
- El instalador debe ser repetible, con perfiles y estado verificable; la restauración es parte del producto.
- Nunca formatear discos o modificar servicios, red, montajes ni datos automáticamente durante el desarrollo.
- `main` estable, trabajo en `develop`, commits pequeños y pruebas acordes al riesgo.

## Cobertura funcional

Las 21 áreas del catálogo en `catalog/modules.tsv` son: core, health, storage, services, docker, projects, deploy, developer, agents, ai, remote, engram, engram-cloud, backups, updates, watchdog, notifications, clean, tui, dashboard y telegram. La cobertura es una lista de requisitos: `planned` no significa implementado ni instalado.

## Almacenamiento deseado

- microSD: DietPi, binarios y configuración del sistema.
- HDD USB (`/srv`): `workspaces`, `docker`, `stacks`, `tools`, `cache`, `engram`, `data` y `backups`.
- Configuración `/etc/javipiv2`; estado `/var/lib/javipiv2`; logs de servicios vía journald.
- No seguir rutas bajo `/srv` si autofs no demuestra un montaje de datos activo; no formatear automáticamente.
- Backups fuera del propio HDD para tolerar su falla física.

## Etapas y entregables

| Fase | Entrega | Criterio de terminación |
| --- | --- | --- |
| F0 — Base | Repo independiente, Rust CLI, catálogo y perfiles, bootstrap sin efectos | Rust compila, pruebas pasan, plan de instalación visible; ninguna modificación al host |
| F1 — Instalación | Bootstrap idempotente, selección de módulos, dependencias, configuración y activación separada | Minimal instalado dos veces en DietPi de prueba, con logs y recuperación de fallos |
| F2 — Observación | CPU, RAM, red, montajes, SMART opcional, systemd y Docker solo lectura | Mediciones reales verificadas, unknown ante falta de evidencia, sin activar automount |
| F3 — Administración | Services, Docker, Projects, Deploy, Developer, Agents, AI, Remote, Engram/Cloud | Cada módulo instalable/habilitable por separado y operaciones tipadas comprobadas |
| F4 — Mantenimiento | Restic/restore, Updates, Watchdog, Notifications y Clean | Backup/restore ensayados; rollback solo donde esté realmente verificado |
| F5 — Interfaces | Ratatui, Dashboard, pantalla táctil y Telegram | Misma lógica compartida; Dashboard autenticado; ningún comando shell libre |
| F6 — Migración | Matriz de equivalencias, migrador de configuración y pruebas de reinstalación | Se reconstruye el servidor desde DietPi limpio y backup externo sin perder funciones del alcance |

## Perfiles

- `minimal`: núcleo, salud, almacenamiento y servicios (la selección no activa servicios automáticamente).
- `developer`: Minimal más proyectos, toolchains, agentes, IA, acceso remoto, Engram, backup y actualización.
- `full`: los 21 módulos, incluidas integraciones que necesiten credenciales y configuración posterior.

Los archivos `profiles/*.list` son la fuente de verdad inicial para la selección; el orden incorpora las dependencias. Las dependencias e identidad de módulos se encuentran en `catalog/modules.tsv`.

## Reinstalación y recuperación: requisitos de producto

1. **MicroSD formateada, HDD intacto:** instalar DietPi y JaviPiV2, reconocer el disco por UUID, reutilizar datos y restaurar configuración/módulos sin borrar información.
2. **MicroSD + HDD perdidos:** reinstalar, recuperar desde repositorio Restic externo y restaurar bases de datos y volúmenes mediante procedimientos verificados.
3. **Migración del JaviPi antiguo:** importar configuraciones y datos por componente con copia previa y consentimiento; dejar el viejo sistema intacto hasta validar la sustitución.

El instalador inicial no implementa estos flujos: son gates obligatorios para F1/F4/F6.

## Estado actual y siguiente trabajo

- **F0:** CLI, perfiles y bootstrap validados en Raspberry ARM64 con pruebas, formato, Clippy estricto y build; CI remoto no comprobado.
- **F1:** preflight y simulación Minimal de solo lectura en desarrollo: muestran perfil, dependencias, código disponible y comprobaciones pendientes, sin instalar. Existe un modelo de cinco estados de instalación, todavía no conectado a operaciones. La capacidad de `/srv` permanece `unknown`: no implica ausencia ni capacidad cero y no se consulta el HDD ante falta de verificación segura. Una operación futura que necesite espacio deberá rechazar sin modificar datos si no puede comprobarlo con seguridad. Todavía no determina presencia real ni acciones exactas, y no implementa instalación, idempotencia ni reanudación de operaciones reales.
- **F2-A:** `status` incorpora promedios de carga del kernel y temperatura de una zona `cpu-thermal` identificada de forma única; ausencia, ambigüedad o datos inválidos se muestran como desconocidos. La implementación y sus pruebas están pendientes de ejecución y validación; no acredita instalación ni funcionamiento de módulos administrativos.
- **F2-B:** consultas puntuales de servicios systemd en `service list` y `service status <nombre>`, sin administrar servicios. El listado cubre unidades conocidas por el gestor, no todas las instaladas; implementación y comportamiento real pendientes de validación.
- **F2-C:** observación puntual de Docker local: requiere `docker.service` cargado y activo; admite `docker.socket` cargado y activo o inactivo, o no encontrado, y rechaza estados inciertos/no admitidos. La consulta es de solo lectura, pero persiste una ventana TOCTOU en la que systemd puede cambiar estados o activar el socket tras comprobarlos; no hay garantía contra esa carrera. `status`, `list` y vista Docker de TUI comprobados con contenedores reales; `inspect` real no se ejecutó.
- **F2-D:** inventario Git directo y de solo lectura, con gate pasivo ext4 en `/srv` que admite una capa autofs subyacente y rechaza montajes relevantes para la consulta; la validación operativa real sigue pendiente. Los metadatos Git compatibles no prueban integridad ni validez; no verifica identidad física ni capacidad. La siguiente prioridad es la TUI Ratatui, sin iniciar automáticamente otra etapa.
- **TUI-1:** primera interfaz Ratatui de solo lectura para Health, Services, Docker, Projects y Storage, pendiente de compilación, pruebas y validación de terminal y consultas. Docker y Projects requieren selección y solicitud explícitas; no consultar proyectos productivos durante el desarrollo. `Cargo.lock` regenerado para las dependencias de la TUI; su compilación y comportamiento permanecen sin verificar. TUI-2 agrega selección y detalles de solo lectura con datos de la última consulta, pendiente de compilación, pruebas y validación real. No iniciar TUI-3 automáticamente.
- **F3-A:** planificación tipada Start/Stop/Restart compartida en Services, con lista exacta configurable vacía por defecto y autorización exclusiva para planes. La CLI permite autorizar una unidad por consulta; no ejecuta acciones ni acredita existencia o propiedad de la unidad. Implementación y pruebas pendientes de compilación y validación; TUI sin controles administrativos. No iniciar F3-B automáticamente.
- **F3-B:** ejecutor con política independiente que rechaza todos los planes antes de invocar systemctl o el ejecutor inyectado. La salida del comando no acredita el estado del servicio; no hay registro de operaciones reales ni controles administrativos. El registro de unidades administradas, la comprobación del estado y la activación real siguen pendientes de diseño y autorización expresa.
- **Backups-A:** observación de JSON de snapshots explícito por stdin, sin acceso a Restic ni repositorios; pruebas sintéticas, compilación y verificaciones Rust aprobadas en ARM64. No se consultaron backups reales. **Backups-B propuesto:** diseñar una fuente explícita y segura de evidencia del último intento y resultado, sin asumir que un snapshot demuestra éxito; requiere autorización y diseño, no está implementado.
- **Storage-A:** observación pasiva de mountinfo para `/srv` y overlays Docker; no consulta dispositivos ni capacidad. La TUI reutiliza el módulo Storage-A en una vista bajo consulta manual; no activa autofs, abre `/srv` ni administra discos. La compilación y las pruebas automáticas pasaron en ARM64; la comprobación manual de la TUI está pendiente.
- **Storage-B:** CLI y TUI consultan capacidad total, usada, disponible y reservada solo tras confirmar ext4 activo en `/srv` mediante mountinfo; errores y ambigüedad se presentan como desconocidos. La consulta tiene una carrera TOCTOU y puede activar autofs si cambia el montaje tras la comprobación. Pruebas sintéticas y CLI real validadas en ARM64; el operador confirmó el funcionamiento manual de la TUI. La aserción literal de captura automatizada falló con actualizaciones parciales del terminal. No monta dispositivos explícitamente ni modifica datos.
- **Updates-A:** consulta manual de seis componentes del JaviPi original mediante endpoints HTTPS públicos fijos, compartida entre CLI y TUI; no instala ni consulta APT. Solo detecta versiones npm en dos prefijos públicos de `/usr`; los binarios independientes y otras rutas quedan desconocidos. Las versiones numéricas se comparan por componentes; fallos y versiones no admitidas quedan desconocidos. Pruebas ARM64 y consulta CLI real aprobadas; el operador confirmó la TUI manualmente. La prueba PTY no confirmó la selección de Updates y no llegó a consultar; no demuestra un fallo de la aplicación.
- **Updates-B:** amplía la búsqueda local de los cuatro paquetes npm a dos prefijos conocidos de DietPi y rechaza versiones candidatas contradictorias; Gentle AI y Engram siguen desconocidos sin una invocación de versión cuya seguridad esté verificada. CLI y TUI reutilizan la consulta existente. Validación ARM64 y consulta CLI real completadas; Gentle AI y Engram permanecen desconocidos.
- **F2 restante y F3-C–F6:** pendientes.
- **F1-B:** progreso simulado en memoria con eventos ordenados, estados explícitos y consultas; no ejecuta instalaciones ni persiste datos.
- **F1-C:** checkpoint JSON versionado y reanudación exclusivamente simulada: valida el plan y reproduce eventos antes de aceptar estados; conserva resultados y errores, y reconcilia como pendiente una operación simulada interrumpida. Un reemplazo mediante rename no garantiza durabilidad frente a pérdida de energía; se presupone un solo escritor y un fallo puede dejar un archivo temporal. No se ejecutan instalaciones ni se presupone cómo reconciliar efectos reales.
- **F1-D:** la biblioteca integra el perfil Minimal validado con progreso y checkpoint simulado en bytes; permite avanzar paso a paso, reconstruir desde un checkpoint y mostrar resultados sin operaciones reales. No acepta rutas de checkpoint en el coordinador ni almacena automáticamente esos bytes. La API de archivos heredada de F1-C acepta rutas y no debe usarse con entradas no confiables.
- **PAR-1:** la CLI conecta `simulate minimal` con el coordinador en memoria y muestra progreso simulado; cambio pendiente de validación. La ejecución manual como usuario normal y la convivencia operativa no se han comprobado. No se crean servicios, binarios globales ni rutas productivas.
- **Trabajo aplazado:** F1-E, pruebas de instalación y pruebas de PAR-1, F2-A, F2-B y F2-C quedan aplazadas por decisión del operador. La recuperación persistente de operaciones reales sigue pendiente. Futuras rutas `/etc/javipiv2` y `/var/lib/javipiv2`, nombres de servicios y puertos serán independientes; no se crean todavía.

Mantené solo este plan vivo; la documentación adicional se limita a instalación, operación, módulos y recuperación. Cada commit actualiza el estado real sin simular aceptación.