# F1-B simulated progress
Branch: develop | Delivery: stacked-to-main | Runner: CARGO_BUILD_JOBS=1 cargo test --locked
No push or merge. F1-C is outside this increment.

## Specs
S1. «Cada operación simulada debe tener un identificador estable y un estado.»
S2. «Registrar el inicio y resultado de cada operación, conservando el orden de los acontecimientos.»
S3. «Poder consultar cuántas operaciones están pendientes, ejecutándose, completadas, fallidas u omitidas.»
S4. «Permitir conocer qué operación está activa y cuál fue la última operación finalizada.»
S5. «No atribuir éxito a una operación que no terminó.» «Registrar explícitamente los errores simulados, sin inventar resultados.» «Distinguir claramente **progreso simulado** de instalación real.»
S6. «Reutilizar el modelo de F1-A.» «Mantener la implementación dentro del módulo correspondiente.» «Utilizar estructuras Rust sencillas.» «Empezar con estado en memoria y operaciones sintéticas.» «Agregar únicamente las transiciones mínimas necesarias para representar el progreso.»
S7. «No implementar todavía persistencia, recuperación después de reinicio ni reanudación: eso corresponde a F1-C.» «No agregar nuevas opciones de CLI salvo que sean necesarias para demostrar y probar el progreso.»
S8. «Varias operaciones completadas en orden.» «Una operación que falla.» «Una operación omitida.» «Consulta de progreso antes, durante y después de la simulación.» «Rechazo de identificadores inválidos, operaciones inexistentes o transiciones incoherentes.» «Contabilización correcta de todos los estados.» «Ausencia de ejecución de acciones reales.»
S9. «No utilizar Docker, systemctl, sudo ni instalaciones productivas durante las pruebas.» «No hacer push ni merge sin autorización.»
S10. «Ejecutá las pruebas específicas, la suite ARM64 aplicable, rustfmt, Clippy estricto y build con `CARGO_BUILD_JOBS=1` y `--locked`.» «Creá un commit aislado con F1-B cuando las verificaciones correspondientes pasen.»

## Tasks
- T1 [S1-S9] inline done — in-memory simulation, focused RED unresolved module and GREEN 2 tests; no real actions.
- T2 [S8-S10] inline in_progress — validate, review, update PLAN, commit.

## Log
L1 — User request verbatim:
**Decisión: aceptar F1-A en su alcance limitado y comenzar F1-B.**

El commit `183be5c` queda aceptado como implementación del modelo de estados. No modificarlo innecesariamente ni ampliar su alcance.

### Objetivo de F1-B

Implementar un sistema sencillo para **registrar y consultar el progreso de operaciones simuladas**, reutilizando los estados existentes:

- `Pending`
- `Running`
- `Completed`
- `Failed`
- `Skipped`

### Comportamiento esperado

1. Cada operación simulada debe tener un identificador estable y un estado.
2. Registrar el inicio y resultado de cada operación, conservando el orden de los acontecimientos.
3. Poder consultar cuántas operaciones están pendientes, ejecutándose, completadas, fallidas u omitidas.
4. Permitir conocer qué operación está activa y cuál fue la última operación finalizada.
5. No atribuir éxito a una operación que no terminó.
6. Registrar explícitamente los errores simulados, sin inventar resultados.
7. Distinguir claramente **progreso simulado** de instalación real.

### Alcance técnico

- Reutilizar el modelo de F1-A.
- Mantener la implementación dentro del módulo correspondiente.
- Utilizar estructuras Rust sencillas.
- Empezar con estado en memoria y operaciones sintéticas.
- Agregar únicamente las transiciones mínimas necesarias para representar el progreso.
- No implementar todavía persistencia, recuperación después de reinicio ni reanudación: eso corresponde a F1-C.
- No agregar nuevas opciones de CLI salvo que sean necesarias para demostrar y probar el progreso.

### Pruebas obligatorias

Cubrir de manera determinista:

- Varias operaciones completadas en orden.
- Una operación que falla.
- Una operación omitida.
- Consulta de progreso antes, durante y después de la simulación.
- Rechazo de identificadores inválidos, operaciones inexistentes o transiciones incoherentes.
- Contabilización correcta de todos los estados.
- Ausencia de ejecución de acciones reales.

No utilizar Docker, systemctl, sudo ni instalaciones productivas durante las pruebas.

### Estado del repositorio

Antes de modificar archivos, inspeccioná `git status` y conservá cualquier cambio ajeno.

La actualización de seguimiento pendiente debe incorporarse únicamente si refleja correctamente el estado confirmado y no contiene cambios ajenos.

No incluyas `.engram/`, archivos `__pycache__/`, credenciales ni datos personales en los commits. Tampoco elimines archivos sin autorización.

La copia del borrador rechazado en `/dev/shm` es volátil. No dependas de ella para continuar; si realmente se necesita conservarla, trasladala a una ubicación recuperable y segura, sin agregarla al repositorio.

### Validación

Ejecutá las pruebas específicas, la suite ARM64 aplicable, rustfmt, Clippy estricto y build con `CARGO_BUILD_JOBS=1` y `--locked`.

Si una revisión independiente no puede ejecutarse por problemas de entorno, registrá ese límite sin confundirlo con un defecto demostrado del código ni afirmar una aprobación inexistente.

Creá un commit aislado con F1-B cuando las verificaciones correspondientes pasen.

**No hacer push ni merge sin autorización.**

### Próximos pasos

Al finalizar F1-B, informá:

1. Qué estructura se utiliza para registrar el progreso.
2. Cómo se representan las operaciones y sus cambios de estado.
3. Qué pruebas pasaron.
4. Qué commit se creó.
5. Qué resta para F1-C.

No empieces F1-C hasta terminar y validar F1-B.

**Prioridad: código simple, modular y funcional. Sin introducir frameworks, nuevos supervisores ni infraestructura innecesaria.**

L2 — Existing tracking delta corresponds to confirmed 183be5c and is preserved. .engram/ and tests/__pycache__/ remain untracked and excluded.
