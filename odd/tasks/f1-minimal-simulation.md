# F1 Minimal simulation
Branch: develop | Delivery: stacked-to-main (user selected; origin/HEAD -> origin/main) | No push or PR authorized
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
S11. «La capacidad del almacenamiento permanece `unknown`.» «No confundir `unknown` con disco ausente, error o capacidad cero.» «Implementá un plan de instalación legible que distinga operaciones previstas, componentes disponibles y comprobaciones pendientes.» «Comprobá que el instalador puede ejecutarse repetidamente en modo simulación sin modificar el sistema.» «No declarar F1 terminada hasta cumplir sus criterios de aceptación.»

## Tasks
- T1 [S1] inline done — ARM64 baseline: cargo test 9 passed, fmt, strict clippy, build passed; bootstrap Python 3 passed; runtime status and minimal plan work. Commit: pending.
- T2 [S2-S9] delegated partial — safe preflight and honest minimal preview committed as 06ab8e0; exact installation actions, system dependencies, storage capacity, installed state and resume remain pending.
- T3 [S3,S7,S9] dropped — user chose unknown capacity and deferred further mount research; no HDD access.
- T4 [S2,S4-S7,S11] partial — read-only Minimal simulation and repeatability committed as c659ed6; exact installation actions and installation/resumption remain pending.
- T5 [S4-S7,S11] delegated in_progress — define concrete, evidence-backed plan steps and simulated states without executing installation.
- T6 [S1-S11] pending — validate and record the next increment.

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
L5 — User explicitly authorized reading /boot/dietpi/.version; marker exists. Worker mvqqt9i8-4-af6n added bounded fixture rejection and marker detection with RED/GREEN. Actual preflight reports DietPi and ARM64 verified; storage and system dependencies unknown. Cargo test, fmt check, strict clippy, build and runtime preflight passed. Targeted verifier confirmed limits; native review review-e26e0d1a314742b6 approved and acknowledged, with informational findings R2-001 and R3-001. Partial increment committed as 06ab8e0; full F1 remains pending.
L6 — Worker mvqr8vt4-6-16id found path-based storage lookup races with unmount and cannot prove S3. Linux open_tree(2) documents AT_NO_AUTOMOUNT on the terminal component without OPEN_TREE_CLONE; fstatfs(2) supports fd-based inspection. Worker mvqrchdj-7-1ame observed a descriptor may refer to a detached mount. Worker mvqrh02g-9-iodm found ordinary mount ID reuse after unmount lacks sufficient descriptor-lifetime evidence. Storage remains unknown; no production /srv probe.
L7 — User correction verbatim:

**Decisión aprobada: mantener la capacidad del almacenamiento como `unknown` y continuar con F1.**

No investigues más por ahora la reutilización de IDs de montaje, la identidad de descriptores abiertos ni los cambios concurrentes de montajes. Registrá el límite técnico de manera breve en `PLAN.md` y continuá.

El commit `06ab8e0` conserva el comportamiento correcto: cuando la capacidad no puede comprobarse con seguridad, debe informar `unknown` y no acceder al HDD.

### Prioridades

1. Conservá el preflight seguro existente. No agregues comprobaciones que puedan activar autofs.
2. Terminá la simulación del instalador Minimal.
3. Verificá las dependencias y el orden de instalación de los módulos.
4. Implementá un plan de instalación legible que distinga operaciones previstas, componentes disponibles y comprobaciones pendientes.
5. Comprobá que el instalador puede ejecutarse repetidamente en modo simulación sin modificar el sistema.
6. Agregá pruebas automatizadas utilizando datos simulados cuando sea posible.
7. Actualizá el seguimiento en `odd/tasks/f1-minimal-simulation.md`, conservando su estado real.
8. Ejecutá las pruebas y creá un commit únicamente con los cambios propios y validados.

### Reglas

- La capacidad del almacenamiento permanece `unknown`.
- No confundir `unknown` con disco ausente, error o capacidad cero.
- No instalar paquetes ni modificar servicios durante las pruebas.
- No escribir en el HDD ni cambiar montajes.
- No implementar supervisores, frameworks o mecanismos adicionales para resolver esta cuestión.
- No hacer push ni merge sin autorización.
- No declarar F1 terminada hasta cumplir sus criterios de aceptación.

### Criterio de diseño

**Las limitaciones de observación no deben bloquear funciones independientes.**

Cuando una operación futura realmente necesite conocer el espacio disponible, deberá comprobarlo mediante un mecanismo seguro y verificable. Si esa comprobación no está disponible, la operación deberá rechazarse sin modificar datos.

Ahora continuá de forma autónoma con las demás tareas seguras de F1. No vuelvas a solicitar una decisión sobre este mismo problema salvo que aparezca información técnica nueva que cambie materialmente el riesgo.

**Objetivo: avanzar hacia un instalador Minimal funcional, modular, sencillo e idempotente.**

L8 — Read-only simulation committed as c659ed6. Tests: 15 default passed, one explicitly ignored and passed separately in user-authorized RAM sandbox; fmt, strict clippy, build and differential comparison passed. Independent verifier accepted test isolation correction; native review review-1b492dad8ded4801 approved and acknowledged, informational R2-001/R3-001. The isolated test checks sentinel and directory entries, not all possible writes elsewhere or transient writes. F1 remains incomplete.
L9 — User selected stacked-to-main for future review slices; origin/HEAD identifies main. Selection does not authorize push, PR or merge. Module details and user-required unknown storage states validated; independent verifier confirmed claims; previous help wording restored.
