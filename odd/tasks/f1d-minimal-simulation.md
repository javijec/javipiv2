# F1-D Minimal simulated integration
Branch: develop | Delivery: stacked-to-main | CARGO_BUILD_JOBS=1
Scope: simulated only, no arbitrary checkpoint paths, no F1-E, push or merge.

## Specs
S1. «Cargar el perfil Minimal desde los archivos y el catálogo existentes.» «Validar su selección, identificadores, dependencias y orden.» «Construir una lista determinista de operaciones simuladas.»
S2. «Inicializar el progreso usando F1-A y F1-B.» «Simular la ejecución en el orden establecido.» «Registrar eventos y estados sin ejecutar operaciones reales.» «Generar y recuperar checkpoints utilizando F1-C.»
S3. «Reanudar una simulación interrumpida sin repetir operaciones previamente completadas u omitidas.» «Presentar un resultado legible que distinga claramente las operaciones simuladas de las realmente ejecutadas, que seguirán siendo ninguna.»
S4. «Los identificadores de operación deben mantenerse estables entre la simulación inicial y la recuperación.» «No inferir que un módulo está instalado, habilitado o funcionando a partir de un resultado simulado.»
S5. «Preferí una pequeña función coordinadora para convertir el perfil Minimal en operaciones simuladas y alimentar `SimulatedProgress`.» «La lógica principal debe ser comprobable desde pruebas sin depender de la CLI.»
S6. «`Completed` y `Skipped`: no repetir.» «`Failed`: conservar el error y no reintentar automáticamente.» «`Pending`: permitir su procesamiento simulado.» «`Running` interrumpida: reconciliar como `Pending` exclusivamente para esta simulación.» «Checkpoint corrupto o plan incompatible: rechazar antes de avanzar.»
S7. «Demostrá que una ejecución recuperada produce un resultado coherente con una simulación completa equivalente.» «No extrapolar estas garantías a operaciones reales que produzcan efectos externos.»
S8. «No aceptar rutas arbitrarias de checkpoint que permitan sobrescribir archivos del sistema o seguir enlaces simbólicos peligrosos.» «Utilizar directorios temporales aislados para las pruebas de persistencia.» «No agregar sincronización de escritores concurrentes ni garantías de durabilidad ante pérdida de energía.»
S9. «Solicitá una revisión independiente focalizada en la integración, reanudación y ausencia de efectos reales.» «Creá un commit aislado de F1-D únicamente después de completar las comprobaciones y cerrar los hallazgos bloqueantes.» «No comenzar F1-E sin autorización.»

## Tasks
- T1 [S1-S8] inline done — byte-only Minimal coordinator, restricted accessors, RED/GREEN and ARM64 checks passed.
- T2 [S1-S9] inline in_progress — review corrected candidate, native review and isolated commit.

## Log
L1 — User request verbatim:
# JaviPiV2 — Implementar F1-D: integración del instalador Minimal simulado

**Decisión:** F1-C queda aceptado dentro de su alcance demostrado. Autorizo comenzar F1-D.

Último commit confirmado localmente: `4a9ffa9 feat: persist simulated operation checkpoints`.

No hacer push ni merge. No ejecutar instalaciones reales.

## 1. Objetivo

Integrar F1-A, F1-B y F1-C con el perfil `minimal` para demostrar un flujo completo de instalación simulada y reanudable.

Quiero aprovechar el código existente. No crear otro motor de estados, otro sistema de progreso ni otro mecanismo de persistencia.

## 2. Funcionalidad requerida

Implementá el flujo siguiente:

1. Cargar el perfil Minimal desde los archivos y el catálogo existentes.
2. Validar su selección, identificadores, dependencias y orden.
3. Construir una lista determinista de operaciones simuladas.
4. Inicializar el progreso usando F1-A y F1-B.
5. Simular la ejecución en el orden establecido.
6. Registrar eventos y estados sin ejecutar operaciones reales.
7. Generar y recuperar checkpoints utilizando F1-C.
8. Reanudar una simulación interrumpida sin repetir operaciones previamente completadas u omitidas.
9. Presentar un resultado legible que distinga claramente las operaciones simuladas de las realmente ejecutadas, que seguirán siendo ninguna.

Los identificadores de operación deben mantenerse estables entre la simulación inicial y la recuperación.

No inferir que un módulo está instalado, habilitado o funcionando a partir de un resultado simulado.

## 3. Integración y arquitectura

Usá los módulos Rust existentes.

Preferí una pequeña función coordinadora para convertir el perfil Minimal en operaciones simuladas y alimentar `SimulatedProgress`.

No crear:

- Un framework de workflows.
- Un supervisor de procesos.
- Una segunda máquina de estados.
- Una base de datos.
- Un sistema de plugins dinámicos.
- Nuevos comandos que no aporten una función demostrable.

Si resulta útil exponer una única operación de simulación desde la CLI actual, podés hacerlo, manteniendo explícita su condición de simulación y sin modificar el comportamiento seguro de los comandos existentes.

La lógica principal debe ser comprobable desde pruebas sin depender de la CLI.

## 4. Reanudación

La recuperación debe conservar las garantías de F1-C.

- `Completed` y `Skipped`: no repetir.
- `Failed`: conservar el error y no reintentar automáticamente.
- `Pending`: permitir su procesamiento simulado.
- `Running` interrumpida: reconciliar como `Pending` exclusivamente para esta simulación.
- Checkpoint corrupto o plan incompatible: rechazar antes de avanzar.

Demostrá que una ejecución recuperada produce un resultado coherente con una simulación completa equivalente.

No extrapolar estas garantías a operaciones reales que produzcan efectos externos.

## 5. Seguridad y persistencia

No modificar la Raspberry productiva.

- La capacidad de `/srv` sigue siendo `unknown`.
- No activar autofs ni consultar nuevamente la capacidad del HDD.
- No modificar paquetes, Docker, systemd, montajes ni servicios.
- No escribir checkpoints de prueba en ubicaciones productivas.
- Utilizar directorios temporales aislados para las pruebas de persistencia.
- No aceptar rutas arbitrarias de checkpoint que permitan sobrescribir archivos del sistema o seguir enlaces simbólicos peligrosos.
- No agregar sincronización de escritores concurrentes ni garantías de durabilidad ante pérdida de energía.

F1-D sigue siendo una simulación, no una autorización para instalar JaviPiV2.

## 6. Pruebas de aceptación

Necesito pruebas deterministas que demuestren:

1. Construcción correcta del plan Minimal desde su fuente declarativa.
2. Orden válido de operaciones y dependencias.
3. Simulación completa con progreso y resultado final coherentes.
4. Interrupción y recuperación desde checkpoint.
5. Ausencia de repetición de operaciones completadas u omitidas.
6. Conservación de los fallos simulados.
7. Rechazo de checkpoint corrupto o incompatible.
8. Igualdad del estado final entre ejecución continua y recuperada cuando corresponda.
9. Ausencia de acciones reales sobre el sistema.

Si alguna operación no puede demostrarse, reportala como pendiente o desconocida, nunca como exitosa.

## 7. Estado Git y seguimiento

Antes de comenzar, revisá los cambios staged, unstaged y untracked.

La actualización pendiente de `odd/tasks/f1c-simulated-checkpoint.md` debe conservarse e incorporarse únicamente si refleja fielmente el resultado confirmado.

No agregar `.engram/`, `tests/__pycache__/`, tokens ni credenciales.

Actualizá `PLAN.md` y el seguimiento de F1-D con el estado comprobado.

Registrá en Engram la decisión de integración y los resultados importantes. La disponibilidad remota del resumen anterior sigue sin demostrarse: verificá esa condición mediante los medios de consulta disponibles, sin modificar los servicios de Engram Cloud. Si no puede verificarse, dejalo expresamente pendiente.

## 8. Validación

Usá la toolchain ARM64 disponible, actualmente Rust/Cargo 1.99.1.

Con `CARGO_BUILD_JOBS=1`, ejecutá las pruebas específicas, la suite completa, rustfmt, Clippy estricto y build con `--locked` donde corresponda.

Solicitá una revisión independiente focalizada en la integración, reanudación y ausencia de efectos reales.

Si aparecen defectos demostrados, corregilos. No bloquear la entrega por hipótesis nuevas que no afecten este incremento.

Creá un commit aislado de F1-D únicamente después de completar las comprobaciones y cerrar los hallazgos bloqueantes.

No hacer push ni merge.

## 9. Informe final

Entregame:

- Funcionalidad integrada.
- Cómo se inicia y observa la simulación Minimal.
- Cómo se interrumpe y recupera.
- Pruebas ARM64 y revisión.
- Commit creado.
- Cambios pendientes.
- Estado real de Engram Cloud.
- Limitaciones para avanzar hacia F1-E.

**No comenzar F1-E sin autorización.**

Objetivo principal: terminar una simulación Minimal útil, sencilla y verificable. No quiero más arquitectura que la necesaria para hacer funcionar las piezas existentes.

L2 — Git inspection confirms only F1-C tracking update and untracked .engram/ and tests/__pycache__/; preserved. ARM64 rustc/cargo 1.99.1. High-risk items 1 and 2: checkpoint safety and filesystem paths.
L3 — F1-D coordinator consumes and produces F1-C checkpoint bytes, without accepting or opening paths; callers own safe storage. F1-C's existing generic file API remains unsuitable for untrusted paths and is documented. Focused RED was unresolved integration module; GREEN: 3 tests passed. ARM64 focused and full checks passed after the skipped-event assertion (28 tests passed, 1 ignored), as did fmt, strict Clippy and build. Cloud status lists 75 remote chunks and 72 pending imports, not proof that a specific summary exists remotely.
L4 — Independent review found no blocker in the earlier candidate. Afterwards, the coordinator's progress accessor was removed because it indirectly exposed F1-C's path-taking save API; state and events are exposed read-only instead. ARM64 focused tests, full suite, fmt, strict Clippy and build passed again. The independent review predates this accessor restriction.
