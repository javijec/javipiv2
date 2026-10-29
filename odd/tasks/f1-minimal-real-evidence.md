# F1 Minimal installation evidence
Work on develop, keep production unchanged and preserve unrelated files.

## Specs
S1. "Revisar F1 y los módulos actuales. Evitar duplicar funcionalidades existentes."
S2. "Estudiar los requisitos reales del perfil Minimal: Core, Health, Storage y Services."
S3. "Implementar detección pasiva de componentes instalados usando evidencia verificable."
S4. "Distinguir instalado, ausente y desconocido, sin inferir instalación del catálogo."
S5. "Comparar el estado observado con el perfil solicitado e identificar requisitos pendientes."
S6. "Integrar el diagnóstico en CLI y TUI, reutilizando la arquitectura actual."
S7. "Probar planificación, errores y recuperación usando fixtures y directorios temporales aislados."
S8. "Ejecutar validación integral ARM64, actualizar documentación y entregar commits separados."
S9. "sin sudo, instalaciones reales, cambios productivos, modificaciones de montajes, push, merge ni GitHub Actions. No modificar JaviPi original. Conservar archivos ajenos."

## Tasks
- T1 [S1,S9] done: F1 and existing modules reviewed, no duplicated engine
- T2 [S2,S9] done: no authoritative installation artifact or owned-unit contract found; see L2
- T3 [S3,S9] blocked: actual installed/absent detection awaits an authoritative installation contract; passive environment observations implemented in 6a72773
- T4 [S4,S9] done: three states exist, real results remain unknown without a trusted source; commit: 6a72773
- T5 [S5,S9] done: catalog Minimal requirements reported as unverified; commit: 6a72773
- T6 [S6,S9] done: diagnose minimal and manual Installation TUI; commit: 6a72773
- T7 [S7,S9] done: 13 focused tests, isolated target-contained TMPDIR, fixture error/recovery and existing planning/checkpoint tests; commit: 6a72773
- T8 [S8,S9] done: ARM64 156 passed, 1 ignored; fmt/check/strict Clippy/build passed; commit: final documentation

## Log
L3. ARM64: 156 tests passed, 0 failed, 1 ignored; fmt/check/strict Clippy/build passed. Thirteen focused fixture/planning/checkpoint tests also passed with a target-contained isolated TMPDIR. Real read-only diagnostic reported Health, Storage and Services environmental signals available, while all four installation states remain unverified. TUI tested synthetically; interactive confirmation pending. Worker failed after partial changes and parent completed them; initial menu-navigation test failed after adding the view and passed after its expected wrap was updated. The test for precise non-executable plan wording also failed before correction and passed afterward. No real installation evidence source is defined.
L2. Minimal declares core, health, storage, services and their catalog dependencies. No real install manifest, owned-unit declaration, installed artifact contract or authorized complete source exists. Runtime code and host Health/mountinfo/systemd evidence do not establish JaviPiV2 installation or absence; all four installed states remain unknown pending an authoritative source. Avoid /srv statvfs and never treat systemd not-found as software absence. A safe diagnostic may report independent environment evidence and unverified requirements without inferring installation.
L1. # JaviPiV2 — Bloque autónomo F1 (8 etapas)

El bloque anterior terminó con 152 pruebas aprobadas. Continuá desarrollando autónomamente durante ocho etapas seguras.

**Objetivo: pasar de la simulación a identificar el estado real de instalación de JaviPiV2, sin modificar el servidor.**

1. Revisar F1 y los módulos actuales. Evitar duplicar funcionalidades existentes.
2. Estudiar los requisitos reales del perfil Minimal: Core, Health, Storage y Services.
3. Implementar detección pasiva de componentes instalados usando evidencia verificable.
4. Distinguir instalado, ausente y desconocido, sin inferir instalación del catálogo.
5. Comparar el estado observado con el perfil solicitado e identificar requisitos pendientes.
6. Integrar el diagnóstico en CLI y TUI, reutilizando la arquitectura actual.
7. Probar planificación, errores y recuperación usando fixtures y directorios temporales aislados.
8. Ejecutar validación integral ARM64, actualizar documentación y entregar commits separados.

**Criterios:**
- Priorizar funcionalidad real sobre nuevas abstracciones.
- No crear otro motor de simulación.
- Reutilizar catálogo, planes, Health, Storage y Services.
- No presentar planes como ejecutables si todavía faltan acciones definidas.
- Continuar automáticamente entre etapas seguras.
- Corregir defectos concretos y ejecutar pruebas por incremento.
- Si una etapa requiere acceso productivo no autorizado, documentarla y continuar.

**Restricciones:** sin sudo, instalaciones reales, cambios productivos, modificaciones de montajes, push, merge ni GitHub Actions. No modificar JaviPi original. Conservar archivos ajenos.

Crear commits locales individuales y entregar un informe final con pruebas, funcionalidades reales comprobadas y pendientes. No iniciar una novena etapa.
