# Instalación — estado actual y objetivo

**Aún no se puede instalar JaviPiV2 en producción.** F0 contiene únicamente un CLI de lectura y un bootstrap de preflight/plan. No lo ejecutes con sudo ni en lugar del JaviPi antiguo.

## Desarrollo

1. Clonar el repositorio independiente en una ruta de desarrollo segura.
2. Utilizar una toolchain Rust compatible (Rust 1.99 o superior).
3. Ejecutar `cargo test --locked` y `cargo run --locked -- status`.
4. Consultar `bash installer/bootstrap.sh --plan minimal`.

En un servidor con `/srv` gestionado por autofs, verificar el montaje mediante `/proc/self/mountinfo` **antes de abrir el checkout bajo `/srv`**. Estas instrucciones no autorizan forzar montajes.

## Preflight Minimal

`cargo run --locked -- preflight minimal` consulta `/etc/os-release` y el marcador `/boot/dietpi/.version`: `ID=dietpi` o un marcador legible y no vacío son evidencia de DietPi; sin evidencia el resultado es desconocido. Identifica la arquitectura del binario, no necesariamente la del host, y muestra el perfil con dependencias ordenadas. No abre `/srv` ni consulta capacidad: almacenamiento y dependencias del sistema permanecen desconocidos. La presencia de módulos y las acciones también son desconocidas. No existe todavía un plan exacto, registro de instalación, idempotencia ni reanudación. `--fixture` recibe tres líneas simuladas por stdin, como describe el README, y no representa evidencia real.

## Simulación Minimal

`cargo run --locked -- simulate minimal` reutiliza el preflight de solo lectura. Enumera los módulos en el orden del perfil y distingue para cada uno código disponible en este programa o funcionalidad pendiente, presencia instalada desconocida y acciones de instalación sin definir; también muestra comprobaciones pendientes. Con `--fixture` usa el mismo stdin simulado del preflight. Es repetible y no escribe estado ni instala; no constituye un plan exacto ni implementación de reanudación. La capacidad permanece desconocida: una futura operación que la necesite deberá comprobarla de forma segura o rechazarse sin modificar datos. F1 permanece incompleta.

La prueba de ausencia de escrituras está marcada como `ignored` para no crear archivos automáticamente: prepara un directorio aislado vacío con un único archivo `sentinel` cuyo contenido sea `unchanged`, y ejecuta `JAVIPIV2_TEST_SANDBOX=<directorio> cargo test --locked --test simulation simulation_does_not_change_isolated_state -- --ignored`. El directorio es responsabilidad de quien ejecuta la prueba.

### Simulación reanudable desde la biblioteca

`MinimalSimulation::new()` valida el perfil y crea progreso simulado. `step()` avanza una operación; `checkpoint()` devuelve bytes para conservar por separado. Tras la interrupción, `MinimalSimulation::from_checkpoint(&bytes)` valida el checkpoint antes de aceptar el progreso; `run()` procesa las pendientes y `report()` muestra estados simulados. No hay operaciones reales. La CLI descrita arriba no inicia este flujo.

El coordinador no acepta rutas ni escribe archivos. Para persistir los bytes, el llamador debe usar un mecanismo de almacenamiento seguro y aislado: la API de archivos heredada de F1-C acepta rutas arbitrarias y no debe usarse con rutas no confiables. La reanudación es solo simulada; no garantiza durabilidad ante pérdida de energía ni seguridad con escritores concurrentes.

## Objetivo F1

Un único instalador que prepare Minimal, Developer o Full en DietPi ARM64; compruebe prerequisitos, muestre el plan, instale solo módulos elegidos, registre versiones y permita reejecuciones idempotentes. La descarga deberá utilizar artefactos identificados y verificados; el instalador no ejecutará automáticamente scripts remotos.

Instalar módulos no implica habilitarlos. Las migraciones de datos y la configuración de credenciales requieren pasos propios. No habrá formato automático del HDD.
