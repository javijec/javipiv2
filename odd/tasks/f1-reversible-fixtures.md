# F1 reversible installer fixtures
Fixture-only work; production execution remains disabled.

## Specs
S1. "acepto el contrato F1 de rutas y confianza como diseño, pero NO autorizo instalación real."
S2. "instalador reversible con preflight de colisiones, publicación atómica y recuperación ante fallos."
S3. "Implementá y probá únicamente con fixtures y directorios temporales aislados, sin tocar `/usr/local/bin`, `/var/lib`, servicios ni JaviPi original."
S4. "Revisá los riesgos de seguridad concretos, sin agregar complejidad innecesaria. Mantené la ejecución productiva deshabilitada."
S5. "Validá en ARM64, creá commits locales pequeños y no hagas push ni inicies otra etapa."
S6. "La validación interactiva de Installation la realizará el operador."

## Tasks
- T1 [S1-S4] done: inspected F1, fixed test-owned root and per-file atomicity boundary; see L2
- T2 [S2-S4] done: fixture-only collision preflight, no-replace publication; commit: pending
- T3 [S2-S4] blocked: recovery after journal implemented; failure before journal leaves ambiguous temporaries and requires a safe recovery design; commit: pending
- T4 [S4-S6] in progress: security review, ARM64 tests and documentation; commit: pending

## Log
L3. The writer subagent returned blocked, then partial without implementation; parent implemented a private Linux test-only fixture module. Preflight rejects collisions; two individually atomic no-replace renames publish fixture binary and F1-style record. In a private 0700 root with no hostile same-UID concurrent writer, recovery verifies recorded device/inode and bytes before deleting and refuses changed or foreign entries. Failure before journal remains ambiguous and does not self-recover. No power-loss durability or production security is established; checked identity and unlink are not an atomic operation. Six focused tests and Clippy passed; full ARM64 suite pending.
L2. Restrict writer to code compiled only during tests, with internally created private fixture root and fixed relative names; no caller-supplied destination, CLI or production execution. Existing path-taking simulated_progress API is not a secure writer. Preflight alone cannot avoid collisions: publish must be no-replace. Two atomic renames do not form one transaction; sync and injected failures do not prove power-loss durability. Rollback must not remove altered or replaced objects. Preserve unknown states rather than claim recovery when evidence is insufficient.
L1. JaviPiV2: acepto el contrato F1 de rutas y confianza como diseño, pero NO autorizo instalación real.

Prepará F1 siguiente: instalador reversible con preflight de colisiones, publicación atómica y recuperación ante fallos. Implementá y probá únicamente con fixtures y directorios temporales aislados, sin tocar `/usr/local/bin`, `/var/lib`, servicios ni JaviPi original.

Revisá los riesgos de seguridad concretos, sin agregar complejidad innecesaria. Mantené la ejecución productiva deshabilitada. Validá en ARM64, creá commits locales pequeños y no hagas push ni inicies otra etapa.

La validación interactiva de Installation la realizará el operador.
