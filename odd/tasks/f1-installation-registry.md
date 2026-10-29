# F1 installation registry
Work on develop; isolate fixtures and preserve unrelated files.

## Specs
S1. "utilizaremos un registro propio de instalación como fuente autoritativa."
S2. "Core, Health, Storage y Services son capacidades internas del mismo binario; no las trates como paquetes instalados independientemente."
S3. "Diseñá un registro sencillo en `/var/lib/javipiv2/installation.json`, creado únicamente por el futuro instalador, con versión, ubicación del ejecutable e información para verificar su integridad."
S4. "Implementá su modelo, validación y lectura segura mediante fixtures aislados. El diagnóstico debe distinguir disponible en desarrollo, instalado verificado y desconocido. No deduzcas ausencia solo porque falta el registro."
S5. "Reutilizá el código F1 existente. No crees otro motor de instalación ni agregues complejidad innecesaria."
S6. "Integrá CLI/TUI, ejecutá pruebas ARM64 y creá commits locales separados."
S7. "No escribas en `/var/lib`, no instales nada, no uses sudo, no modifiques JaviPi original y no hagas push. Conservá archivos ajenos."
S8. "Después proponé los pasos necesarios para pasar a una primera instalación real y reversible, sin ejecutarla todavía."

## Tasks
- T1 [S1-S5,S7] done: reviewed existing diagnosis and descriptor-based safe reading; see L2
- T2 [S1-S5,S7] done: fixed-path descriptor-relative verifier, sha256, 5 fixture tests and Clippy; commit: 1f70a8a
- T3 [S2,S4-S7] done: single binary status and capabilities, existing shared CLI/TUI diagnosis, fixture integration; commit: 246a699
- T4 [S6-S8] done: ARM64 164 passed, 1 ignored; docs/F1_INSTALLATION_REGISTRY.md proposal; commit: closing documentation

## Log
L3. ARM64 final validation: 164 passed, 0 failed, 1 ignored; fmt/check/strict Clippy/build passed, with target-contained TMPDIR. Initial valid fixture failed because umask 0002 made files group writable; explicit fixture permissions fixed it. A new test exposed acceptance of a non-executable file; it was fixed and retested. Initial final fmt check failed and was corrected before the full passing suite. The writing subagent returned partial output, completed inline. No production registry or binary was read or written by the tests; no installation was executed. CLI/TUI share one binary diagnosis; TUI interactive confirmation and independent security review remain unverified. Trust is root ownership/non-writability, not proof of origin; version is declared, and metadata checks do not prevent all privileged races. Reversible installation remains a proposal only.
L2. Existing Minimal diagnosis reports per-module Unknown; replace that with one binary status. Catalog modules are capabilities, not separately installed packages. A matching SHA-256 demonstrates byte agreement with the registry only under a trustworthy root-owned, non-writable registry and executable path; neither a hash nor metadata alone proves installer provenance. Missing, malformed, inaccessible or mismatched evidence must never imply absence. Descriptor-relative no-follow bounded reads should reject symlinks and special files; hash the opened file, never execute it. Do not claim immunity to concurrent modification of the same inode. Tests use isolated fixture trees; no production writes.
L1. JaviPiV2 — Decisión F1: utilizaremos un registro propio de instalación como fuente autoritativa.

Core, Health, Storage y Services son capacidades internas del mismo binario; no las trates como paquetes instalados independientemente.

Diseñá un registro sencillo en `/var/lib/javipiv2/installation.json`, creado únicamente por el futuro instalador, con versión, ubicación del ejecutable e información para verificar su integridad.

Implementá su modelo, validación y lectura segura mediante fixtures aislados. El diagnóstico debe distinguir disponible en desarrollo, instalado verificado y desconocido. No deduzcas ausencia solo porque falta el registro.

Reutilizá el código F1 existente. No crees otro motor de instalación ni agregues complejidad innecesaria.

Integrá CLI/TUI, ejecutá pruebas ARM64 y creá commits locales separados.

No escribas en `/var/lib`, no instales nada, no uses sudo, no modifiques JaviPi original y no hagas push. Conservá archivos ajenos.

Después proponé los pasos necesarios para pasar a una primera instalación real y reversible, sin ejecutarla todavía.
