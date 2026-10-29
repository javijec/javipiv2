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
- T2 [S1-S5,S7] in progress: implement isolated registry model, validation and read-only integrity verification; commit: pending
- T3 [S2,S4-S7] integrate status CLI/TUI with safe fixtures; commit: pending
- T4 [S6-S8] ARM64 tests, documentation, reversible installation proposal; commit: pending

## Log
L2. Existing Minimal diagnosis reports per-module Unknown; replace that with one binary status. Catalog modules are capabilities, not separately installed packages. A matching SHA-256 demonstrates byte agreement with the registry only under a trustworthy root-owned, non-writable registry and executable path; neither a hash nor metadata alone proves installer provenance. Missing, malformed, inaccessible or mismatched evidence must never imply absence. Descriptor-relative no-follow bounded reads should reject symlinks and special files; hash the opened file, never execute it. Do not claim immunity to concurrent modification of the same inode. Tests use isolated fixture trees; no production writes.
L1. JaviPiV2 — Decisión F1: utilizaremos un registro propio de instalación como fuente autoritativa.

Core, Health, Storage y Services son capacidades internas del mismo binario; no las trates como paquetes instalados independientemente.

Diseñá un registro sencillo en `/var/lib/javipiv2/installation.json`, creado únicamente por el futuro instalador, con versión, ubicación del ejecutable e información para verificar su integridad.

Implementá su modelo, validación y lectura segura mediante fixtures aislados. El diagnóstico debe distinguir disponible en desarrollo, instalado verificado y desconocido. No deduzcas ausencia solo porque falta el registro.

Reutilizá el código F1 existente. No crees otro motor de instalación ni agregues complejidad innecesaria.

Integrá CLI/TUI, ejecutá pruebas ARM64 y creá commits locales separados.

No escribas en `/var/lib`, no instales nada, no uses sudo, no modifiques JaviPi original y no hagas push. Conservá archivos ajenos.

Después proponé los pasos necesarios para pasar a una primera instalación real y reversible, sin ejecutarla todavía.
