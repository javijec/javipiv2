# F1 Minimal simulation
Branch: develop | Delivery: ask-on-risk | Forecast: about 400+ authored lines
Checks: CARGO_BUILD_JOBS=1; cargo test --locked, cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, cargo build --locked

## Specs
S1. «Primero validá F0 en la Raspberry real y corregí cualquier fallo reproducible.»
S2. «Detecte DietPi, arquitectura ARM64 y dependencias.»
S3. «Verifique de manera segura el almacenamiento disponible sin activar autofs.»
S4. «Lea el catálogo y sus dependencias.»
S5. «Muestre un plan exacto de instalación.» «Distinga componentes ya presentes de los pendientes.»
S6. «No sobrescriba archivos ni configuraciones existentes.» «Prepare un mecanismo sencillo para instalación idempotente y reanudación.»
S7. «Tenga pruebas automatizadas que no modifiquen el servidor.» «Primero implementá y validá el modo de simulación. No actives todavía operaciones que instalen paquetes, modifiquen systemd, escriban en directorios del sistema o alteren la instalación productiva.»
S8. «PLAN.md será nuestra fuente de verdad.» «Las funcionalidades deberán estar separadas en módulos Rust, pero mantendremos inicialmente un solo proyecto Cargo. No crear frameworks, supervisores genéricos, capas de autorización propias o documentación innecesaria.»
S9. «El JaviPi original debe seguir funcionando sin cambios. No modificar sus servicios, Docker, SSH, Tailscale, backups, Engram Cloud productivo ni datos del HDD.»
S10. «Creá commits pequeños, claramente identificados, después de las validaciones. No hagas push ni merge sin autorización.»

## Tasks
- T1 [S1] inline done — ARM64 baseline: cargo test 9 passed, fmt, strict clippy, build passed; bootstrap Python 3 passed; runtime status and minimal plan work. Commit: pending.
- T2 [S2-S9] delegated in_progress — implement safe preflight and honest minimal preview; exact installation actions and installed state are not defined by catalog; RED/GREEN and commit pending.
- T3 [S1-S10] inline pending — independent risk verification, final checks, update PLAN, memory and commits.

## Log
L1 — User request verbatim:
Vamos a comenzar oficialmente el desarrollo de **JaviPiV2**.

## Contexto del proyecto

- **Repositorio:** `javijec/javipiv2`
- **Directorio:** `/srv/workspaces/JaviPiV2`
- **Rama de desarrollo:** `develop`
- **Lenguaje principal:** Rust
- **Sistema:** Raspberry Pi 4, DietPi ARM64
- **Gestor de agentes:** HerdR
- **Memoria compartida:** Engram Cloud

JaviPiV2 es un proyecto completamente nuevo. Su objetivo es reemplazar progresivamente el JaviPi original mediante una solución completa, modular, sencilla de mantener y capaz de reinstalarse después de formatear la Raspberry.

**No queremos repetir la complejidad de las reescrituras anteriores.**

## 1. Preparación

Antes de hacer modificaciones:

1. Comprobá que `/srv` está correctamente montado, evitando accesos que activen autofs.
2. Verificá el repositorio, el remoto y la rama `develop`. Si estamos en `main`, inspeccioná el estado de trabajo antes de cambiar de rama; no descartes modificaciones.
3. Leé `README.md`, `PLAN.md`, `docs/INSTALL.md`, `docs/MODULES.md` y `docs/RECOVERY.md`.
4. Revisá la estructura Rust y el catálogo de módulos.
5. Comprobá que Engram Cloud está configurado para `javipiv2`, que la sincronización funciona y que Pi puede consultar y registrar memoria mediante la integración disponible. No supongas que la integración de memoria de Pi está operativa solo porque el CLI de Engram funciona.
6. Ejecutá las pruebas iniciales en ARM64 con `CARGO_BUILD_JOBS=1`, `cargo test --locked`, `cargo fmt --check`, Clippy estricto y build.

## 2. Cómo vamos a trabajar

**PLAN.md será nuestra fuente de verdad. Engram Cloud conservará el contexto y las decisiones entre sesiones.**

Al comenzar cada sesión, recuperá de Engram el estado y las decisiones relevantes del proyecto. Al terminar un incremento, registrá brevemente las funcionalidades terminadas, decisiones, problemas pendientes y próximos pasos.

No guardes secretos, credenciales ni tokens en Engram o Git.

Las funcionalidades deberán estar separadas en módulos Rust, pero mantendremos inicialmente un solo proyecto Cargo. No crear frameworks, supervisores genéricos, capas de autorización propias o documentación innecesaria.

## 3. Primera tarea: completar F0 y comenzar F1

Primero validá F0 en la Raspberry real y corregí cualquier fallo reproducible.

Después avanzá con **F1: instalador modular y configuración**.

Quiero comenzar por un instalador Minimal que:

- Detecte DietPi, arquitectura ARM64 y dependencias.
- Verifique de manera segura el almacenamiento disponible sin activar autofs.
- Lea el catálogo y sus dependencias.
- Muestre un plan exacto de instalación.
- Distinga componentes ya presentes de los pendientes.
- No sobrescriba archivos ni configuraciones existentes.
- Prepare un mecanismo sencillo para instalación idempotente y reanudación.
- Tenga pruebas automatizadas que no modifiquen el servidor.

**Primero implementá y validá el modo de simulación.** No actives todavía operaciones que instalen paquetes, modifiquen systemd, escriban en directorios del sistema o alteren la instalación productiva.

Avanzá de manera autónoma con las tareas de desarrollo seguras que puedas completar. No te detengas a pedir decisiones por detalles menores: elegí la opción más sencilla, documentá la decisión y continuá.

Creá commits pequeños, claramente identificados, después de las validaciones. No hagas push ni merge sin autorización.

## 4. Restricciones

El JaviPi original debe seguir funcionando sin cambios. No modificar sus servicios, Docker, SSH, Tailscale, backups, Engram Cloud productivo ni datos del HDD.

No importar indiscriminadamente código ni planes de las ramas Rust anteriores.

No realizar limpiezas, restauraciones, actualizaciones globales, reinicios ni acciones privilegiadas.

## 5. Resultado esperado

Quiero que me informes:

- Estado de F0 en ARM64.
- Resultado de la integración con Engram Cloud.
- Qué implementaste de F1.
- Pruebas ejecutadas y resultados.
- Commits creados.
- Qué sigue según PLAN.md.

**Objetivo principal: construir un JaviPiV2 funcional, modular y fácilmente reinstalable, sin volver a perder tiempo en complejidad innecesaria.**

L2 — /proc/self/mountinfo shows ext4 mounted at /srv alongside autofs. develop matches origin/develop; .engram/ was already untracked. Engram Pi save/read succeeds; cloud import completed, 3 local and 3 remote chunks, zero pending. F0 tests pass on aarch64, runtime plan is visible. The Engram tracker copy is incomplete and must be reconciled.
L3 — Worker mvqqjrvm-1-vfxt found catalog lacks installation actions and presence evidence; no edits made. Worker mvqqmjms-2-6rh4 implemented partial read-only preflight, with RED 2 failures and GREEN 2 passes; storage capacity, dependencies, installed state, exact actions and resume remain pending.
L4 — ARM64 full cargo tests, fmt check, strict clippy and build passed. Differential comparison against HEAD for existing commands with --fixture matched stdout, stderr and exit code. Independent verification mvqqqghs-3-u8sm found DietPi detection did not recognize Debian-based DietPi and confirmed incomplete S2/S3/S5/S6. PLAN.md updated to accurately mark partial F1. Engram tracker mirror is incomplete.
L5 — User explicitly authorized reading /boot/dietpi/.version; marker exists. Worker mvqqt9i8-4-af6n added bounded fixture rejection and marker detection with RED/GREEN. Actual preflight reports DietPi and ARM64 verified; storage and system dependencies unknown. Cargo test, fmt check, strict clippy, build and runtime preflight passed. Targeted independent verification mvqqy77x-5-dvbl pending.
