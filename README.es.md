# ugreen-cli

[English](README.md) · [Documentación detallada, en inglés](docs/README.md)

CLI e interfaz de terminal en Rust para configurar localmente auriculares **UGREEN Studio Pro** en Linux y Windows. Proyecto no oficial, sin afiliación ni respaldo de UGREEN.

> **Compatibilidad de hardware.** La implementación sigue el protocolo de referencia de los Studio Pro **HP206** y una respuesta capturada, verificada con firmware comercial **0.2.5** en Linux (lecturas, las nueve escrituras incl. un ciclo reversible; ver [verificación](docs/verification.md)). La elección del modelo la declaras tú, no es detección de identidad. **HiTune Max5c no es compatible:** algunos identificadores de comando tienen significados distintos.

## Inicio rápido

Instala [Rust](https://rust-lang.org/tools/install/) y los requisitos de tu sistema indicados en la [guía de compilación](docs/building.md). Desde el directorio del proyecto:

```sh
cargo build --release --locked
cargo run --release --locked -- tui
```

El ejecutable se genera en `target/release/ugreen` en Linux o `target\release\ugreen.exe` en Windows. Los ejemplos usan `ugreen`; sustituye ese nombre por la ruta completa si no está en tu `PATH`.

El ZIP de distribución también incluye ejecutables para Linux x86-64: `bin/linux-x86_64/ugreen` (TUI + CLI) y `bin/linux-x86_64/ugreen-cli-only`. Ambos requieren glibc 2.39 o posterior y `libgcc_s.so.1`; compila localmente en distribuciones anteriores. No se incluye un ejecutable de Windows; sigue sus instrucciones de compilación.

Al iniciar, la TUI se reconecta al último dispositivo confirmado (activado por defecto; `--no-autoconnect` lo desactiva). Sin destino guardado inicia desconectada: pulsa `a` para introducir la dirección, o `p` para cargar explícitamente la lista de dispositivos ya emparejados del sistema; confirma el protocolo Studio Pro con `m` y después `y`, y pulsa `c` para conectar y consultar el estado. Empareja antes los auriculares desde los ajustes del sistema. Abrir la interfaz, elegir un dispositivo o editar un valor propuesto no modifica sus ajustes.

Primera comprobación sin Bluetooth:

```sh
ugreen --help
ugreen models
ugreen --dry-run set anc ultra
```

Para consultar el dispositivo, sustituye la dirección de ejemplo por la de tus auriculares:

```sh
ugreen discover
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status
```

## Interfaz de terminal o CLI

La característica `tui` está activada por defecto y usa Ratatui 0.30.2 y Crossterm 0.29.0. `ugreen tui` abre la interfaz explícitamente. Sin argumentos, se abre solo cuando la entrada y la salida estándar son terminales; en caso contrario se muestra la ayuda. Los comandos explícitos siguen disponibles para scripts.

- `a`: editar dirección; `p`: cargar dispositivos emparejados; `m` y después `y`: confirmar protocolo
- `c`: conectar; `r`: actualizar estado; `d`: desconectar
- Arriba/Abajo: seleccionar ajuste; Izquierda/Derecha: cambiar el valor propuesto
- Intro: revisar el cambio; `y`: confirmarlo; Esc: cancelar un diálogo
- `l`: registro de la sesión; `?`: ayuda; `q`: salir

Un valor propuesto no es un valor leído del dispositivo. Solo se considera verificado después de recibir una confirmación válida y comprobar que la lectura posterior coincide. La batería procede de la respuesta de información del dispositivo; los datos ausentes permanecen como desconocidos. No se informa del códec ni se permite seleccionarlo. Consulta el [manual de uso](docs/usage.md) para ver el flujo completo y el tratamiento de errores.

Para conservar solo la CLI, que usa la biblioteca estándar, sin compilar las dependencias de la TUI:

```sh
cargo build --release --locked --no-default-features
cargo run --locked --no-default-features -- --help
```

El paquete declara Rust 1.88 o posterior. La compilación predeterminada descarga dependencias de terminal de terceros; la variante sin características predeterminadas no las compila. Una compilación sin conexión sigue necesitando las herramientas, los metadatos del registro y las dependencias necesarias en la caché.

## Ajustes disponibles

| Ajuste | Valores |
| --- | --- |
| `anc` | `off`, `ultra`, `general`, `gentle`, `adaptive`, `ambient` |
| `eq` | `classic`, `jazz`, `electronic`, `pop`, `classical`, `rock`, `bass`, `treble` |
| `game`, `spatial`, `dual`, `wind` | `on`, `off` |
| `prompts` | `voice`, `beeps` |
| `volume-up-action`, `volume-down-action` | `none`, `next`, `previous` |

Los dos últimos ajustes asignan **acciones al mantener pulsados los botones**, no cambian el volumen de reproducción. Los nombres distinguen mayúsculas y minúsculas y deben escribirse exactamente como aparecen. No hay ecualización personalizada ni selector de códec.

```sh
ugreen commands
ugreen --dry-run set eq bass
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass
ugreen profile example > my-profile.conf
ugreen --dry-run profile apply my-profile.conf
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile apply my-profile.conf
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > saved-profile.conf
```

Las opciones van antes del comando. `--channel` acepta 1–30, con 1 por defecto; `--timeout` acepta 1–60 segundos, con 3 por defecto, por operación. No es un límite total para un comando que realiza varias solicitudes ni para un perfil completo.

Los perfiles son archivos UTF-8 con líneas `clave=valor`, de hasta 16 KiB y con `model=studio-pro` obligatorio. Admiten líneas vacías y comentarios que ocupan toda la línea y empiezan por `#`. Las claves duplicadas o desconocidas y los valores inválidos se rechazan antes de acceder a Bluetooth. Los ajustes se aplican secuencialmente y **pueden aplicarse solo en parte**; no hay reversión automática. La [guía de perfiles](docs/usage.md#profiles) incluye las precauciones de codificación para Windows.

## Seguridad y límites

- Las solicitudes al dispositivo requieren una dirección y la selección explícita del protocolo Studio Pro. Seleccionar el modelo no verifica la identidad física
- `discover` consulta dispositivos ya emparejados. No realiza búsquedas activas, emparejamientos ni elección automática de destino; la TUI solo se reconecta a tu último dispositivo confirmado (activado por defecto, `--no-autoconnect` lo desactiva) y nunca elige un dispositivo desconocido
- La gestión multipunto (lista, desconexión, reconexión y cambio de dispositivo activo) se rechaza antes de enviar ningún byte hasta existir una captura verificada; la conmutación dual es un ajuste distinto y verificado
- Antes de escribir se consulta la información del dispositivo; solo se anuncia éxito cuando la lectura posterior coincide
- No se reintentan escrituras automáticamente. Un tiempo de espera agotado puede indicar que el ajuste cambió, pero se perdió la respuesta; consulta el estado antes de repetir
- No hay actualización de firmware, restablecimiento de fábrica, envío de bytes arbitrarios, sonido de localización, cambio del volumen real ni reasignación del botón ANC
- Sin telemetría, servicio de red, cuenta ni credenciales guardadas; el control es local mediante Bluetooth

Una aplicación del móvil puede ocupar el canal de control RFCOMM. Cierra esa sesión antes de reintentar; un error no justifica restablecer ni desemparejar los auriculares. Lee la [guía de problemas](docs/troubleshooting.md) antes de repetir escrituras.

## Inspección de capturas sin conexión

```sh
ugreen decode 'DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3'
```

Valida la captura de referencia sin acceder a Bluetooth. Las solicitudes usan CRC MODBUS; las respuestas, CCITT-FALSE. Una captura mal formada, con ruido, incompleta, con CRC incorrecto o con tramas `AA BB CC` sin significado RX verificado devuelve un código de salida distinto de cero. Las notificaciones de seis bytes se decodifican con `event=unknown`.

Para grabar una sesión en vivo, `capture` registra cada byte TX/RX y lo imprime como hex en estilo de archivo de prueba en la salida estándar (el contexto va a la salida de error), listo para guardarlo, volver a pasarlo a `decode` o revisarlo como posible archivo de prueba:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro capture > session.hex
ugreen decode "$(cat session.hex)"
```

`#` inicia un comentario en la entrada hexadecimal. Las [notas del protocolo](docs/protocol.md) detallan las pruebas y los límites del decodificador.

## Documentación y verificación

El [índice](docs/README.md) enlaza las guías de [compilación](docs/building.md), [uso](docs/usage.md), [arquitectura](docs/architecture.md), [protocolo y procedencia](docs/protocol.md), [verificación](docs/verification.md) y [problemas habituales](docs/troubleshooting.md).

Linux y Windows tienen implementaciones nativas de RFCOMM. El [flujo de CI](.github/workflows/ci.yml) incluye comprobaciones por plataforma; su presencia no implica que se haya ejecutado remotamente. Compilar y superar pruebas simuladas no verifica la compatibilidad de los auriculares físicos ni la ejecución nativa en Windows. macOS y otras plataformas no tienen una implementación Bluetooth en este proyecto.

[Serena](docs/serena-pilot.md) es una herramienta de desarrollo opcional. Su prueba anterior es histórica; no indica una instalación actual ni es un requisito para compilar o ejecutar el programa.

## Licencia y atribución

[MIT](LICENSE). El protocolo portado y la captura de prueba se adaptaron de [sanild/ugreen-studio-controller-macos](https://github.com/sanild/ugreen-studio-controller-macos). La investigación sobre tramas y CRC se contrastó con [tzy3454u/ugreen-headphone-control](https://github.com/tzy3454u/ugreen-headphone-control). Los avisos MIT originales se conservan en [third-party](third-party/); mantenlos al redistribuir. Las dependencias de terminal tienen sus propias licencias, recogidas en el [inventario de dependencias Rust](third-party/RUST-DEPENDENCIES.md). Los nombres de productos y las marcas pertenecen a sus respectivos propietarios.
