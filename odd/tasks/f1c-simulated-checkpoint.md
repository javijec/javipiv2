# F1-C simulated checkpoints
Branch: develop | Delivery: stacked-to-main | Runner: CARGO_BUILD_JOBS=1 cargo test --locked
Scope: simulated persistence only; no F1-D, push or merge.

## Specs
S1. «Guardar un checkpoint del progreso con estados y eventos ordenados.» «Reconstruir una instancia de progreso desde ese checkpoint, simulando un nuevo proceso.»
S2. «Preservar las operaciones `Completed` y `Skipped` sin repetirlas.» «Tratar una operación `Running` interrumpida como pendiente de reconciliación. En esta simulación puede volver a `Pending`, pero no debe trasladarse automáticamente ese comportamiento a futuras operaciones reales con efectos externos.»
S3. «Conservar las operaciones `Failed` y su error. Cualquier reintento debe ser explícito.» «Rechazar checkpoints corruptos, incompletos o incompatibles con el plan esperado.» «Impedir que una recuperación inválida produzca resultados ficticios o modifique parcialmente el estado.»
S4. «Reutilizá F1-A y F1-B.» «Mantené una sola implementación de la máquina de estados.» «Utilizá un formato de persistencia simple y versionado.» «Usá identificadores estables de operaciones para verificar la compatibilidad del plan.»
S5. «Para probar la persistencia, utilizá directorios temporales aislados, nunca `/srv` ni rutas productivas.» «Si hace falta una biblioteca de serialización estándar del ecosistema Rust, incorporarla está permitido; no desarrolles un formato complejo propio sin necesidad.» «Mantené el alcance limitado a operaciones simuladas. No agregues todavía una CLI de instalación ni un motor para ejecutar comandos.»
S6. «Interrupción después de completar varias operaciones.» «Recuperación con una operación `Running`.» «Preservación de operaciones `Failed` y `Skipped`.» «Reanudación sin repetir operaciones completadas.» «Rechazo de checkpoints corruptos o planes incompatibles.» «Persistencia sin dejar un checkpoint parcial válido ante una escritura fallida simulada, usando una estrategia sencilla de reemplazo atómico donde sea aplicable.» «Ausencia de acciones reales sobre el servidor.»
S7. «Separá los resultados de pruebas unitarias de cualquier garantía de durabilidad física: el uso de rename atómico no demuestra resistencia a pérdida de energía.» «La capacidad de `/srv` sigue siendo `unknown`.» «No hacer push ni merge sin autorización.»
S8. «Validá en ARM64 con `CARGO_BUILD_JOBS=1`, pruebas específicas, suite aplicable, rustfmt, Clippy estricto y build.» «Creá un commit aislado de F1-C cuando las pruebas pasen.» «No comiences F1-D todavía.»
S9. «Decisión: establecer Rust 1.99 como versión mínima del proyecto.» «Cerrá R3 cuando se demuestre que F1-C compila y sus pruebas pasan con Rust 1.99.»
S10. «ningún checkpoint externo debe poder reconstruir un `SimulatedProgress` aceptado sin validar las invariantes del modelo.» «Utilizar un DTO privado para representar el checkpoint deserializable.» «Reconstruir `SimulatedProgress` exclusivamente mediante una API que valide versión, plan, identificadores y eventos.»

## Tasks
- T1 [S1-S7] inline done — JSON checkpoint, ordered event replay and recovery; RED then GREEN; ARM64 checks passed.
- T2 [S6-S10] inline in_progress — resolve R3/R4 with RED/GREEN and Rust 1.99, then focused review and commit.

## Log
L1 — User request verbatim:
**Decisión: F1-B aceptado en su alcance. Avanzá con F1-C: persistencia y reanudación de operaciones simuladas.**

### Objetivo

A partir de `SimulatedProgress`, implementar el mecanismo mínimo necesario para conservar el progreso y reconstruirlo después de una interrupción simulada.

La implementación debe preparar el modelo para un futuro instalador reanudable, **sin ejecutar instalaciones reales ni modificar JaviPi productivo**.

### Comportamiento requerido

1. Guardar un checkpoint del progreso con estados y eventos ordenados.
2. Reconstruir una instancia de progreso desde ese checkpoint, simulando un nuevo proceso.
3. Preservar las operaciones `Completed` y `Skipped` sin repetirlas.
4. Tratar una operación `Running` interrumpida como pendiente de reconciliación. En esta simulación puede volver a `Pending`, pero no debe trasladarse automáticamente ese comportamiento a futuras operaciones reales con efectos externos.
5. Conservar las operaciones `Failed` y su error. Cualquier reintento debe ser explícito.
6. Rechazar checkpoints corruptos, incompletos o incompatibles con el plan esperado.
7. Impedir que una recuperación inválida produzca resultados ficticios o modifique parcialmente el estado.

### Diseño

- Reutilizá F1-A y F1-B.
- Mantené una sola implementación de la máquina de estados.
- Utilizá un formato de persistencia simple y versionado.
- Usá identificadores estables de operaciones para verificar la compatibilidad del plan.
- Para probar la persistencia, utilizá directorios temporales aislados, nunca `/srv` ni rutas productivas.
- Si hace falta una biblioteca de serialización estándar del ecosistema Rust, incorporarla está permitido; no desarrolles un formato complejo propio sin necesidad.
- Mantené el alcance limitado a operaciones simuladas. No agregues todavía una CLI de instalación ni un motor para ejecutar comandos.

### Pruebas necesarias

Demostrá mediante pruebas deterministas:

- Interrupción después de completar varias operaciones.
- Recuperación con una operación `Running`.
- Preservación de operaciones `Failed` y `Skipped`.
- Reanudación sin repetir operaciones completadas.
- Rechazo de checkpoints corruptos o planes incompatibles.
- Persistencia sin dejar un checkpoint parcial válido ante una escritura fallida simulada, usando una estrategia sencilla de reemplazo atómico donde sea aplicable.
- Ausencia de acciones reales sobre el servidor.

Separá los resultados de pruebas unitarias de cualquier garantía de durabilidad física: el uso de rename atómico no demuestra resistencia a pérdida de energía.

### Reglas operativas

- La capacidad de `/srv` sigue siendo `unknown`.
- No volver a investigar la identidad de montajes en este incremento.
- No modificar Docker, systemd, servicios, agentes ni montajes.
- No instalar paquetes globales ni ejecutar operaciones privilegiadas.
- No introducir frameworks, coordinadores o mecanismos adicionales.
- No hacer push ni merge sin autorización.

Inspeccioná primero el estado Git. Conservá `.engram/`, `tests/__pycache__/` y cualquier modificación ajena. Incorporá las actualizaciones de seguimiento de F1-B únicamente si representan correctamente el estado confirmado.

Validá en ARM64 con `CARGO_BUILD_JOBS=1`, pruebas específicas, suite aplicable, rustfmt, Clippy estricto y build. Registrá cualquier límite de las comprobaciones independientes.

Creá un commit aislado de F1-C cuando las pruebas pasen.

Actualizá `PLAN.md` y registrá en Engram Cloud una síntesis de las decisiones relevantes, sin credenciales.

**No comiences F1-D todavía.**

Al finalizar, entregame un informe breve con la semántica de recuperación, pruebas, commit y limitaciones pendientes.

**Prioridad: una reanudación simulada realmente demostrable, con la menor complejidad posible.**

L2 — F1-B tracking delta matches confirmed df5f6ce; .engram/ and tests/__pycache__/ are excluded. High-risk item 1: checkpoint data and atomic replacement.
L3 — Focused RED failed for missing checkpoint API; GREEN passed. ARM64 targeted tests, full suite (23 passed, 1 ignored), fmt, strict Clippy and build passed after fixing missing PathBuf import. Failure-injection test preserves the prior checkpoint. Temporary test files remain in RAM; rename is not a power-loss durability guarantee.
L4 — Independent verifier confirmed tested recovery cases but did not verify Rust 1.74 or concurrent writers. Native review review-0f52aaae6109fcd5 escalated unknown causality R3-msrv-test and R4-deserialization-bypass; no approval or commit. Cloud sync reported nothing new, but status showed 61 pending imports and did not prove availability of the summary.
L5 — User correction (verbatim excerpts): «Decisión: conservar la implementación de F1-C y resolver los dos hallazgos de la revisión antes de crear el commit.» «Decisión: establecer Rust 1.99 como versión mínima del proyecto.» «ningún checkpoint externo debe poder reconstruir un `SimulatedProgress` aceptado sin validar las invariantes del modelo.» «Utilizar un DTO privado para representar el checkpoint deserializable.» «Solicitá una revisión focalizada sobre R3 y R4.» «No empieces F1-D hasta finalizar F1-C.» Full correction retained in the session prompt.
L6 — ARM64 rustc/cargo 1.99.1; rust-version 1.99. R4 compile-fail RED confirmed direct Deserialize was exposed; private DTO and replay removed the bypass, compile-fail GREEN. Focused 3 tests, full suite 25 passed and 1 ignored, fmt, strict Clippy and build passed. Concurrent writers and power-loss durability remain unproven.
