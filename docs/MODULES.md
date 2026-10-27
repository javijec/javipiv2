# Módulos de JaviPiV2

El archivo `catalog/modules.tsv` declara 21 áreas funcionales. Sus columnas, separadas por tabulaciones, son `id`, `group`, `dependencies`, `state` y `description`. No confundir `preview` con instalado: `preview` indica únicamente que existe algún código experimental para esa área; `planned` indica que no está implementada.

Los perfiles `profiles/{minimal,developer,full}.list` contienen una lista ordenada de identificadores del catálogo. Un módulo siempre aparece después de todas sus dependencias obligatorias. `cargo run --locked -- module list` inspecciona el catálogo embebido y `cargo run --locked -- plan full` muestra el perfil completo, **sin instalar ni habilitar componentes**.

Los módulos Rust verdaderos solo se crearán cuando aporten funcionalidad. Durante F0 existen `core/catalog.rs`, `modules/health.rs` e `interfaces/cli.rs`. Para futuras operaciones se exigirá separar instalación de habilitación y eliminación de datos, así como comprobar errores y recuperación específica.

El instalador F1 incorporará estados reales de instalación, registro de archivos, versiones, permisos, dependencias, comprobaciones y desinstalación. Esos estados aún no existen: el catálogo no debe presentarse como inventario del host.
