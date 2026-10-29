# Registro de instalación F1

Este documento describe el contrato de lectura implementado y una propuesta para una instalación futura. **No ejecuta ni autoriza una instalación.**

## Contrato de lectura v1

- Registro fijo: `/var/lib/javipiv2/installation.json`.
- Archivo fijo declarado por v1: `/usr/local/bin/javipiv2`.
- JSON: `schema_version` igual a `1`, `product_version` no vacía, `executable` igual a la ruta fija y `sha256` de 64 dígitos hexadecimales en minúsculas. La versión de producto es una declaración, no una comprobación del contenido del binario.
- La verificación abre cada componente relativo a un descriptor, sin seguir enlaces simbólicos; requiere propietario root y ausencia de escritura para grupo y otros en cada componente. Ambos archivos deben ser regulares con un enlace; el binario debe tener algún bit de ejecución. La lectura se limita a 64 KiB para el registro y 128 MiB para el binario y contrasta sus metadatos antes y después.
- Un digest coincidente significa coincidencia de bytes con el registro bajo la confianza en root. No demuestra quién creó el registro, que el binario se esté ejecutando, la arquitectura o la versión declarada; tampoco proporciona una instantánea contra todas las escrituras concurrentes privilegiadas.
- Si falta el registro o cualquier comprobación falla, la instalación es desconocida, no ausente. Las compilaciones debug muestran disponibilidad en desarrollo por separado de la instalación desconocida.

No se ha creado un instalador ni se ha escrito el registro real. Los fixtures viven en directorios temporales aislados del árbol de trabajo; el lector no ejecuta el binario.

## Propuesta para la primera instalación reversible

1. Acordar previamente la fuente del binario, su versión y arquitectura, el responsable de publicación y el criterio de confianza del digest; revisar el contrato v1 y sus límites antes de usarlo como evidencia productiva.
2. Diseñar un preflight de solo lectura que detecte colisiones con rutas existentes y permisos incompatibles, y se detenga sin sobrescribir archivos ajenos. Definir requisitos y autorización explícita del operador.
3. Preparar pruebas en una máquina o imagen ARM64 aislada, con una copia recuperable y pruebas de fallos; no usar el servidor productivo para el primer ensayo.
4. Diseñar el instalador para preparar archivos temporales en los sistemas de archivos de destino, comprobar su contenido y permisos, y publicar de forma atómica el binario y después el registro. Especificar `fsync` de archivos y directorios, y cómo reconocer un fallo entre ambas publicaciones; el orden por sí solo no ofrece transacción entre directorios.
5. Antes de cambiar nada, definir y probar un plan de reversión que preserve o restaure cualquier estado previo autorizado, retire únicamente archivos identificados como propios y no elimine datos ajenos. Prever interrupciones, repetición y conflictos posteriores sin asumir que un archivo conservó su identidad.
6. Ejecutar ensayos de instalación, interrupción, verificación y reversión únicamente en el entorno aislado; comprobar propiedad, permisos, ausencia de enlaces inseguros y SHA-256. Registrar resultados y revisión independiente.
7. Solicitar autorización separada para una instalación real, con respaldo verificado, ventana de operación y plan de reversión aceptado. Hasta entonces no escribir en `/var/lib` ni instalar nada.
