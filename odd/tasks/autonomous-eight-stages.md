# JaviPiV2 autonomous eight stages
Work on develop; preserve all unrelated changes and remain within the requested eight stages.

## Specs
S1. "Registrar la validación manual de Remote-B." "Revisar el estado de los 21 módulos y corregir documentación desactualizada." "No confundir funcionalidades parciales con módulos completos."
S2. "Conectar la evaluación de memoria existente con las mediciones reales de Health." "Incorporar un comando de consulta, sin iniciar supervisión permanente." "Informar desconocido ante datos insuficientes."
S3. "Incorporar Watchdog a la TUI reutilizando el módulo anterior." "Actualización manual con R, sin monitoreo periódico ni acciones automáticas."
S4. "Implementar una vista útil del entorno de desarrollo reutilizando los datos locales de Updates y Agents." "Mostrar versiones y disponibilidad verificadas, con su fuente de evidencia." "No ejecutar herramientas ni duplicar los detectores existentes." "Integrar CLI y TUI."
S5. "Incorporar una vista general a la TUI con el estado de los módulos principales." "Reutilizar resultados ya consultados." "Distinguir claramente no consultado, desconocido, disponible y error." "No consultar Docker, Projects u otros módulos automáticamente al abrir Overview."
S6. "Revisar cómo JaviPi original administra stacks y despliegues." "Implementar un inventario pasivo y acotado de stacks o manifiestos Compose únicamente si puede hacerse mediante rutas autorizadas y sin afectar los montajes." "No ejecutar Docker Compose ni desplegar nada." "Si requiere aceptar riesgos nuevos, documentarlo y omitir la implementación."
S7. "Elegir un incremento adicional útil entre los módulos pendientes, basándote en el catálogo y el administrador original." "Priorizar observación real y reutilización de código." "No crear funcionalidades ficticias únicamente para aumentar la cobertura."
S8. "Revisar coherencia de CLI, TUI y módulos compartidos." "Corregir defectos concretos." "Ejecutar pruebas de integración y suite completa ARM64." "Actualizar README.md, PLAN.md y docs/MODULES.md." "Registrar los resultados en Engram Cloud."
S9. "No sudo ni cambios de privilegios." "No modificar JaviPi original ni servicios productivos." "No iniciar, detener o reiniciar contenedores o servicios." "No modificar montajes, datos o configuraciones." "No instalar ni actualizar paquetes." "No activar autofs ni asumir riesgos nuevos de acceso a /srv." "No ejecutar backups ni restauraciones." "No habilitar acciones administrativas reales." "No usar GitHub Actions." "No hacer push ni merge." "Conservar todos los cambios y archivos ajenos." "No comprometer HerdR, Pi Agent ni Engram Cloud."

## Tasks
- T1 [S1,S9] done: documented operator validation and all 21 areas; commit: 9505214
- T2 [S2,S9] done: 5 focused tests and CLI Health query passed; commit: bb37069
- T3 [S3,S9] done: 22 synthetic TUI tests, formatting and Clippy passed; commit: 9d1cc4a
- T4 [S4,S9] done: local CLI, 2 focused tests, 23 TUI tests, Clippy and build; commit: 0d5aba6
- T5 [S5,S9] in progress: TUI overview without automatic queries; commit: pending
- T6 [S6,S9] route: read-only investigation or omit; commit: pending
- T7 [S7,S9] route: choose safe real observation or omit; commit: pending
- T8 [S8,S9] route: integration and final ARM64; commit: pending

## Log
L1. # JaviPiV2 — Bloque autónomo de desarrollo (8 etapas)

Quiero que avances de forma autónoma por las siguientes 8 etapas, sin solicitar autorización entre tareas seguras.

**Remote-B ya fue validado manualmente por el operador.**

Trabajá sobre `develop`, revisando primero `AGENTS.md` si existe, `PLAN.md`, `docs/MODULES.md` y el código actual. Reutilizá la arquitectura Rust existente.

## Etapas

**1. Consolidación**
- Registrar la validación manual de Remote-B.
- Revisar el estado de los 21 módulos y corregir documentación desactualizada.
- No confundir funcionalidades parciales con módulos completos.

**2. Watchdog-B**
- Conectar la evaluación de memoria existente con las mediciones reales de Health.
- Incorporar un comando de consulta, sin iniciar supervisión permanente.
- Informar desconocido ante datos insuficientes.

**3. Watchdog-TUI**
- Incorporar Watchdog a la TUI reutilizando el módulo anterior.
- Actualización manual con R, sin monitoreo periódico ni acciones automáticas.

**4. Developer-B**
- Implementar una vista útil del entorno de desarrollo reutilizando los datos locales de Updates y Agents.
- Mostrar versiones y disponibilidad verificadas, con su fuente de evidencia.
- No ejecutar herramientas ni duplicar los detectores existentes.
- Integrar CLI y TUI.

**5. Overview**
- Incorporar una vista general a la TUI con el estado de los módulos principales.
- Reutilizar resultados ya consultados.
- Distinguir claramente no consultado, desconocido, disponible y error.
- No consultar Docker, Projects u otros módulos automáticamente al abrir Overview.

**6. Deploy-A**
- Revisar cómo JaviPi original administra stacks y despliegues.
- Implementar un inventario pasivo y acotado de stacks o manifiestos Compose únicamente si puede hacerse mediante rutas autorizadas y sin afectar los montajes.
- No ejecutar Docker Compose ni desplegar nada.
- Si requiere aceptar riesgos nuevos, documentarlo y omitir la implementación.

**7. Siguiente funcionalidad**
- Elegir un incremento adicional útil entre los módulos pendientes, basándote en el catálogo y el administrador original.
- Priorizar observación real y reutilización de código.
- No crear funcionalidades ficticias únicamente para aumentar la cobertura.

**8. Integración y cierre**
- Revisar coherencia de CLI, TUI y módulos compartidos.
- Corregir defectos concretos.
- Ejecutar pruebas de integración y suite completa ARM64.
- Actualizar README.md, PLAN.md y docs/MODULES.md.
- Registrar los resultados en Engram Cloud.

## Forma de trabajo

- Un commit local por incremento funcional completo.
- Ejecutar pruebas focalizadas durante cada etapa.
- Ejecutar formato, check, Clippy y build según corresponda, además de la suite completa al finalizar.
- Continuar automáticamente después de cada etapa satisfactoria.
- No detenerse por fallos inciertos de captura PTY: registrar la limitación y utilizar pruebas sintéticas.
- No solicitar revisores externos ni repetir evaluaciones nativas bloqueadas por archivos ajenos.
- Si encontrás un defecto concreto, corregilo antes de continuar.
- Si una etapa no puede completarse de forma segura, documentá el motivo y avanzá con la siguiente.
- No crear commits vacíos ni implementaciones innecesarias.

## Restricciones permanentes

- No sudo ni cambios de privilegios.
- No modificar JaviPi original ni servicios productivos.
- No iniciar, detener o reiniciar contenedores o servicios.
- No modificar montajes, datos o configuraciones.
- No instalar ni actualizar paquetes.
- No activar autofs ni asumir riesgos nuevos de acceso a /srv.
- No ejecutar backups ni restauraciones.
- No habilitar acciones administrativas reales.
- No usar GitHub Actions.
- No hacer push ni merge.
- Conservar todos los cambios y archivos ajenos.
- No comprometer HerdR, Pi Agent ni Engram Cloud.

## Autonomía

Tenés autorización para completar las ocho etapas seguras consecutivamente, creando los commits y realizando sus pruebas sin pedirme confirmación intermedia.

No amplíes el alcance a una novena etapa.

Al finalizar, entregá un único informe con commits, funcionalidades, pruebas, validaciones pendientes y próximos pasos.

**Objetivo: convertir JaviPiV2 en un administrador cada vez más funcional y útil, manteniendo una arquitectura sencilla, modular y mantenible.**
