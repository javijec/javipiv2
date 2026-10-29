# Módulos de JaviPiV2

El archivo `catalog/modules.tsv` declara 21 áreas funcionales. Sus columnas, separadas por tabulaciones, son `id`, `group`, `dependencies`, `state` y `description`. No confundir `preview` con instalado: `preview` indica únicamente que existe algún código experimental para esa área; `planned` es un estado declarativo histórico y no significa necesariamente ausencia de implementación parcial.

Los perfiles `profiles/{minimal,developer,full}.list` contienen una lista ordenada de identificadores del catálogo. Un módulo siempre aparece después de todas sus dependencias obligatorias. `cargo run --locked -- module list` inspecciona el catálogo embebido y `cargo run --locked -- plan full` muestra el perfil completo, **sin instalar ni habilitar componentes**.

El catálogo no es un inventario de implementación: existen módulos Rust para Health, Storage, Services, Projects, Docker, Backups, Updates, Agents y Remote, entre otros. Consulta `PLAN.md` y el código para conocer el alcance y la validación de cada uno. Para futuras operaciones se exigirá separar instalación de habilitación y eliminación de datos, así como comprobar errores y recuperación específica.

`preflight minimal` muestra las dependencias declaradas en orden, pero informa presencia y acciones como desconocidas; no deduce instalación del catálogo. El plan exacto y el mecanismo de idempotencia y reanudación siguen pendientes.

`simulate minimal` enumera los módulos en orden y distingue por módulo el código disponible en el programa, la funcionalidad pendiente y la presencia instalada desconocida; las acciones de instalación siguen sin definir. No acredita instalación ni convierte `preview` o `planned` en estado del host.

Existe un modelo de estados de instalación simulado, sin operaciones reales ni evidencia de presencia en el host. Registro real de archivos, versiones, permisos, dependencias, comprobaciones y desinstalación siguen pendientes; el catálogo no es un inventario del host.
