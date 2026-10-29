# F1 reversible installer fixtures
Fixture-only work; production execution remains disabled.

## Specs
S1. "acepto el contrato F1 de rutas y confianza como diseño, pero NO autorizo instalación real."
S2. "instalador reversible con preflight de colisiones, publicación atómica y recuperación ante fallos."
S3. "Implementá y probá únicamente con fixtures y directorios temporales aislados, sin tocar `/usr/local/bin`, `/var/lib`, servicios ni JaviPi original."
S4. "Revisá los riesgos de seguridad concretos, sin agregar complejidad innecesaria. Mantené la ejecución productiva deshabilitada."
S5. "Validá en ARM64, creá commits locales pequeños y no hagas push ni inicies otra etapa."
S6. "La validación interactiva de Installation la realizará el operador."
S7. "Resolvé el caso de interrupción anterior al registro de identidades mediante un journal preparado antes de crear los archivos temporales, con estados explícitos y recuperación idempotente."
S8. "Agregá pruebas de interrupción en cada etapa y verificá que nunca se sobrescriban ni eliminen archivos ajenos. Si no puede demostrarse la propiedad de un archivo, conserválo y reportá intervención manual."
S9. "Mantené el código exclusivamente en fixtures aislados. Sin instalación real, sudo, cambios productivos, push ni nuevas capas innecesarias."
S10. "Validá en ARM64, creá un commit local e informá los límites que permanezcan."

## Tasks
- T1 [S1-S4] done: inspected F1, fixed test-owned root and per-file atomicity boundary; see L2
- T2 [S2-S4] done: fixture-only collision preflight, no-replace publication; commit: 86c76ef
- T3 [S2-S4,S7-S10] done: journal before temporaries, explicit states and fail-closed recovery; commit: closing work unit
- T4 [S4-S6,S10] done: local security review, ARM64 174 passed/1 ignored, documentation; commit: closing work unit

## Log
L6. RED: prepared-journal test failed because journal was absent before staged files. GREEN: 10 focused tests pass across all injected logical interruption points, modified entries, collisions, journal updates and idempotence; unrecorded files or invalid/pending journal require manual intervention and remain untouched. ARM64 suite 174 passed, 0 failed, 1 ignored; fmt/check/strict Clippy/build passed. Root remains fixture-exclusive, no hostile same-UID concurrent writer; no atomic conditional unlink, cross-file transaction or power-loss durability is claimed. No independent review or manual TUI validation was performed. No real install or productive writes.
L5. JaviPiV2 — F1: completá la recuperación del instalador reversible.

Resolvé el caso de interrupción anterior al registro de identidades mediante un journal preparado antes de crear los archivos temporales, con estados explícitos y recuperación idempotente.

Agregá pruebas de interrupción en cada etapa y verificá que nunca se sobrescriban ni eliminen archivos ajenos. Si no puede demostrarse la propiedad de un archivo, conserválo y reportá intervención manual.

Mantené el código exclusivamente en fixtures aislados. Sin instalación real, sudo, cambios productivos, push ni nuevas capas innecesarias.

Validá en ARM64, creá un commit local e informá los límites que permanezcan.

L4. ARM64 suite 170 passed, 0 failed, 1 ignored; fmt/check/strict Clippy/build passed with isolated target TMPDIR. Six focused fixture tests cover publication, collision/symlink, replay after interruptions, replacement/content changes, and pre-journal ambiguity; post-journal recovery reconstructs state from fixture journal. Local security review documented same-UID check/unlink race, lack of joint atomicity and lack of directory fsync/power-loss proof. No independent security review was performed. Production code remains disabled. Operator will perform TUI validation separately. Code commit 86c76ef; pending recovery blocker unchanged.
L3. The writer subagent returned blocked, then partial without implementation; parent implemented a private Linux test-only fixture module. Preflight rejects collisions; two individually atomic no-replace renames publish fixture binary and F1-style record. In a private 0700 root with no hostile same-UID concurrent writer, recovery verifies recorded device/inode and bytes before deleting and refuses changed or foreign entries. Failure before journal remains ambiguous and does not self-recover. No power-loss durability or production security is established; checked identity and unlink are not an atomic operation. Six focused tests and Clippy passed; full ARM64 suite pending.
L2. Restrict writer to code compiled only during tests, with internally created private fixture root and fixed relative names; no caller-supplied destination, CLI or production execution. Existing path-taking simulated_progress API is not a secure writer. Preflight alone cannot avoid collisions: publish must be no-replace. Two atomic renames do not form one transaction; sync and injected failures do not prove power-loss durability. Rollback must not remove altered or replaced objects. Preserve unknown states rather than claim recovery when evidence is insufficient.
L1. JaviPiV2: acepto el contrato F1 de rutas y confianza como diseño, pero NO autorizo instalación real.

Prepará F1 siguiente: instalador reversible con preflight de colisiones, publicación atómica y recuperación ante fallos. Implementá y probá únicamente con fixtures y directorios temporales aislados, sin tocar `/usr/local/bin`, `/var/lib`, servicios ni JaviPi original.

Revisá los riesgos de seguridad concretos, sin agregar complejidad innecesaria. Mantené la ejecución productiva deshabilitada. Validá en ARM64, creá commits locales pequeños y no hagas push ni inicies otra etapa.

La validación interactiva de Installation la realizará el operador.
