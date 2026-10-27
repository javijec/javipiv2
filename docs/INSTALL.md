# Instalación — estado actual y objetivo

**Aún no se puede instalar JaviPiV2 en producción.** F0 contiene únicamente un CLI de lectura y un bootstrap de preflight/plan. No lo ejecutes con sudo ni en lugar del JaviPi antiguo.

## Desarrollo

1. Clonar el repositorio independiente en una ruta de desarrollo segura.
2. Instalar una toolchain Rust compatible (Rust 1.74 o superior).
3. Ejecutar `cargo test --locked` y `cargo run --locked -- status`.
4. Consultar `bash installer/bootstrap.sh --plan minimal`.

En un servidor con `/srv` gestionado por autofs, verificar el montaje mediante `/proc/self/mountinfo` **antes de abrir el checkout bajo `/srv`**. Estas instrucciones no autorizan forzar montajes.

## Objetivo F1

Un único instalador que prepare Minimal, Developer o Full en DietPi ARM64; compruebe prerequisitos, muestre el plan, instale solo módulos elegidos, registre versiones y permita reejecuciones idempotentes. La descarga deberá utilizar artefactos identificados y verificados; el instalador no ejecutará automáticamente scripts remotos.

Instalar módulos no implica habilitarlos. Las migraciones de datos y la configuración de credenciales requieren pasos propios. No habrá formato automático del HDD.
