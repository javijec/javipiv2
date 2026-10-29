# F1/F3 simulated installation block
Work on develop with isolated simulation; preserve all unrelated changes.

## Specs
S1. "Revisá el estado actual del instalador y detectá funcionalidades faltantes."
S2. "Implementá un inventario verificable de módulos y dependencias."
S3. "Prepará planes de instalación concretos y reproducibles."
S4. "Incorporá ejecución simulada en un sandbox aislado."
S5. "Mejorá checkpoints, idempotencia y recuperación de operaciones simuladas."
S6. "Prepará un registro de unidades administradas exclusivamente por JaviPiV2."
S7. "Integrá planificación y diagnóstico en CLI/TUI, reutilizando el código existente."
S8. "Ejecutá pruebas ARM64, corregí defectos y actualizá la documentación."
S9. "No ejecutar instalaciones ni acciones administrativas reales, modificar JaviPi original, usar sudo, hacer push o merge. Conservá archivos ajenos. Si una etapa requiere autorización productiva, documentá el bloqueo y continuá con otra segura."

## Tasks
- T1 [S1,S9] done: existing catalog, preflight, progress and services reviewed; gaps recorded in L2
- T2 [S2,S9] done: whole-graph validation, 6 focused tests, 8 CLI differential cases; commit: 7cfbf85
- T3 [S3,S9] done: deterministic declarative plans and 3 focused tests; commit: 30250bb
- T4 [S4,S9] done: isolated in-memory sandbox, 3 tests and Clippy passed; commit: 5851ccf
- T5 [S5,S9] in progress: bounded in-memory sandbox checkpoints and recovery; commit: pending
- T6 [S6,S9] JaviPiV2-only managed unit registry; commit: pending
- T7 [S7,S9] CLI/TUI planning and diagnostics; commit: pending
- T8 [S8,S9] final ARM64 and documentation; commit: pending

## Log
L2. Existing catalog validates references and profile order but not whole-graph cycles; preflight is not installed-state evidence. F1 simulation and versioned checkpoints exist, but plans lack action identity and sandbox. Legacy checkpoint file API follows arbitrary paths and is not safe for untrusted inputs. F3 allowlist only authorizes planning and execution remains disabled; service ownership is not verified. Real installation/recovery requires separate authority and environment, and is outside this block.
L1. JaviPiV2 — Iniciá un bloque autónomo de 8 etapas centrado en F1 y F3.

1. Revisá el estado actual del instalador y detectá funcionalidades faltantes.
2. Implementá un inventario verificable de módulos y dependencias.
3. Prepará planes de instalación concretos y reproducibles.
4. Incorporá ejecución simulada en un sandbox aislado.
5. Mejorá checkpoints, idempotencia y recuperación de operaciones simuladas.
6. Prepará un registro de unidades administradas exclusivamente por JaviPiV2.
7. Integrá planificación y diagnóstico en CLI/TUI, reutilizando el código existente.
8. Ejecutá pruebas ARM64, corregí defectos y actualizá la documentación.

Trabajá autónomamente, con commits locales separados y pruebas por incremento. Priorizá funcionalidades utilizables y evitá sobreingeniería.

No ejecutar instalaciones ni acciones administrativas reales, modificar JaviPi original, usar sudo, hacer push o merge. Conservá archivos ajenos. Si una etapa requiere autorización productiva, documentá el bloqueo y continuá con otra segura.

Entregá un informe final con commits, pruebas, funcionalidades implementadas y pendientes. No superar las ocho etapas.
