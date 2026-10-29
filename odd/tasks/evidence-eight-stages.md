# Evidence and safe modules: eight stages
Work on develop; preserve unrelated changes and avoid production effects.

## Specs
S1. "registrar la validación manual de Watchdog, Developer y Overview. Revisar el estado real de los 21 módulos."
S2. "corregir la representación de Health y Remote utilizando evidencia ya almacenada. Distinguir no consultado, desconocido y error, sin consultas automáticas."
S3. "mejorar navegación, presentación de datos y coherencia entre vistas. Corregir defectos reales sin rediseñar toda la interfaz."
S4. "estudiar la implementación original desde GitHub `javijec/JaviPi`. Preparar e implementar un inventario de solo lectura si existe una fuente segura, sin acceder a `/srv` ni ejecutar despliegues."
S5. "estudiar las notificaciones del administrador original desde GitHub. Implementar observación funcional solo si existe una fuente local segura y verificable."
S6. "revisar las funciones de limpieza originales y preparar un diagnóstico pasivo útil, sin borrar archivos ni ejecutar operaciones de limpieza."
S7. "conectar las funcionalidades seguras implementadas con CLI y TUI, reutilizando módulos existentes. Corregir inconsistencias y evitar código duplicado."
S8. "ejecutar pruebas ARM64, formato, check, Clippy estricto y build. Actualizar README.md, PLAN.md y docs/MODULES.md. Registrar resultados en Engram."
S9. "Sin sudo, push, merge ni GitHub Actions." "No modificar JaviPi original." "No instalar ni actualizar software." "No modificar servicios, contenedores, montajes o datos productivos." "No ejecutar backups, restauraciones, despliegues ni limpiezas." "No acceder a rutas protegidas ni credenciales." "No asumir nuevos riesgos de autofs bajo `/srv`." "Conservar los cambios y archivos ajenos." "No implementar acciones administrativas reales."

## Tasks
- T1 [S1,S9] done: 21-area review, 4 catalog tests, operator manual validation recorded; commit: d616437
- T2 [S2,S9] done: typed cached evidence, 26 TUI and 7 Remote tests passed; commit: 608ea2b
- T3 [S3,S9] done: short-terminal menu selection visibility, 27 TUI tests; commit: 8ead82e
- T4 [S4,S9] done: original GitHub Deploy documented; no safe source without /srv or protected registry; implementation omitted
- T5 [S5,S9] done: original GitHub Notifications documented; state under /srv, no safe local source; implementation omitted
- T6 [S6,S9] in progress: passive root filesystem capacity diagnostic using Storage arithmetic; commit: pending
- T7 [S7,S9] integration and consistency; commit: pending
- T8 [S8,S9] ARM64 final checks and Engram; commit: pending

## Log
L2. GitHub original docs/DEPLOY-MANAGER.md: four Compose names in /srv/workspaces, registry /etc/javipi/deployments.json. docs/SMART-NOTIFICATIONS.md: incidents in /srv/javipi/state/notifications.json. No authorized safe runtime source, so implementations omitted. docs/CLEANUP.md: status includes disk usage, caches and Docker; implement only root filesystem capacity as passive diagnostic, never inspect protected paths or cleanup candidates.
L1. # JaviPiV2 — Bloque autónomo de desarrollo (8 etapas)

El operador confirmó la revisión manual satisfactoria de las nuevas vistas TUI. Continuá autónomamente durante estas 8 etapas, sin solicitar confirmación entre tareas seguras.

**Objetivo:** acercar JaviPiV2 al reemplazo funcional de JaviPi original, manteniendo el código modular, sencillo y reutilizable.

1. **Consolidación:** registrar la validación manual de Watchdog, Developer y Overview. Revisar el estado real de los 21 módulos.

2. **Overview-B:** corregir la representación de Health y Remote utilizando evidencia ya almacenada. Distinguir no consultado, desconocido y error, sin consultas automáticas.

3. **TUI:** mejorar navegación, presentación de datos y coherencia entre vistas. Corregir defectos reales sin rediseñar toda la interfaz.

4. **Deploy-A:** estudiar la implementación original desde GitHub `javijec/JaviPi`. Preparar e implementar un inventario de solo lectura si existe una fuente segura, sin acceder a `/srv` ni ejecutar despliegues.

5. **Notifications-A:** estudiar las notificaciones del administrador original desde GitHub. Implementar observación funcional solo si existe una fuente local segura y verificable.

6. **Clean-A:** revisar las funciones de limpieza originales y preparar un diagnóstico pasivo útil, sin borrar archivos ni ejecutar operaciones de limpieza.

7. **Integración:** conectar las funcionalidades seguras implementadas con CLI y TUI, reutilizando módulos existentes. Corregir inconsistencias y evitar código duplicado.

8. **Validación final:** ejecutar pruebas ARM64, formato, check, Clippy estricto y build. Actualizar README.md, PLAN.md y docs/MODULES.md. Registrar resultados en Engram.

## Autonomía

- Continuar automáticamente después de cada etapa satisfactoria.
- Crear commits locales independientes por funcionalidad.
- Si una etapa está bloqueada por un riesgo nuevo, documentarlo y continuar con la siguiente.
- No crear funciones ficticias para completar etapas.
- No detenerse por problemas inciertos de captura PTY o revisiones externas no disponibles.
- Corregir defectos concretos antes de avanzar.
- No superar las ocho etapas.

## Restricciones

- Sin sudo, push, merge ni GitHub Actions.
- No modificar JaviPi original.
- No instalar ni actualizar software.
- No modificar servicios, contenedores, montajes o datos productivos.
- No ejecutar backups, restauraciones, despliegues ni limpiezas.
- No acceder a rutas protegidas ni credenciales.
- No asumir nuevos riesgos de autofs bajo `/srv`.
- Conservar los cambios y archivos ajenos.
- No implementar acciones administrativas reales.

## Entrega

Al finalizar, presentar un informe único con commits, funcionalidades incorporadas, pruebas aprobadas, etapas omitidas y validaciones manuales pendientes.

**Prioridad: funcionalidades reales, mínima complejidad y avance continuo hacia un administrador completo.**
