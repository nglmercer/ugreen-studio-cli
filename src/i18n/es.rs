//! Spanish catalog. Starts from English and overrides every
//! translated string; `{}` placeholders keep the same order.
use super::{ShortcutSection, L};

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / control Studio Pro ";
    l.connected = "CONECTADO";
    l.disconnected = "DESCONECTADO";
    l.unknown = "DESCONOCIDO";
    l.target = "Destino: ";
    l.target_none = "sin elegir";
    l.protocol = "Protocolo: ";
    l.protocol_on = "Studio Pro HP206 (elegido por ti)";
    l.protocol_off = "sin elegir; m para confirmar Studio Pro HP206";
    l.battery = "Batería: ";
    l.battery_pct = "{}\u{a0}%";
    l.unavailable = "no disponible";
    l.codec_line = "Códec: no detectado";
    l.firmware = "Firmware: ";
    l.snapshot = "Lectura: ";
    l.snapshot_stale = "OBSOLETA / no actual";
    l.snapshot_ago = "leída hace {}\u{a0}s";
    l.snapshot_never = "sin leer";
    l.rfcomm_line = "RFCOMM {} | tiempo por operación {}\u{a0}s | fw 0.2.5 verificado";
    l.labels = [
        "ANC",
        "Ecualizador",
        "Modo juego",
        "Audio espacial",
        "Conexión dual",
        "Reducción de viento",
        "Avisos",
        "Acción subir volumen",
        "Acción bajar volumen",
    ];
    l.proposed = "  -> {} (propuesto)";
    l.settings_title = " Ajustes / lectura del dispositivo, sin escritura automática ";
    l.settings_blocked = " Ajustes / ESCRITURA BLOQUEADA: r para actualizar ";
    l.settings_stale = " Ajustes / última lectura obsoleta ";
    l.status_title = " Estado ";
    l.categories = ["Audio", "Conexión", "Entorno", "Respuestas"];
    l.picker_title = " Seleccionar valor ";
    l.picker_hint = "Arriba Abajo mueve / Enter aplica / Esc cierra";
    l.status_category = "Categoría: {}";
    l.status_working = " Trabajando / Esc cancela las siguientes etapas ";
    l.log_title = " Registro / Arriba Abajo Inicio Fin / l Esc cerrar ";
    l.shortcuts_title = " Atajos / Arriba Abajo ver / ? Esc cerrar ";
    l.shortcuts = &[
        ShortcutSection {
            title: "Conexión",
            items: &[
                ("a", "Editar dirección de destino (desconectado)"),
                ("p", "Cargar dispositivos emparejados (auto al iniciar)"),
                ("Enter", "Elegir dispositivo: confirma el protocolo o conecta"),
                ("m luego y", "Confirmar protocolo Studio Pro HP206 (conecta si la dirección es válida)"),
                ("c / r / d", "Conectar / actualizar estado / desconectar"),
                ("", "Se recuerdan destino, canal, espera e idioma"),
            ],
        },
        ShortcutSection {
            title: "Ajustes",
            items: &[
                ("Tab / S-Tab", "Cambiar la categoría de ajustes"),
                ("1-9", "Ir al enésimo ajuste de la categoría"),
                ("Arriba/Abajo", "Seleccionar ajuste (cruza categorías)"),
                ("Izq/Der", "Proponer valor (solo local, no escribe)"),
                ("Enter", "Aplicar la propuesta o abrir la lista"),
                ("", "Clic en un valor de la lista para aplicarlo ya"),
            ],
        },
        ShortcutSection {
            title: "App",
            items: &[
                ("s", "Ajustes de la app: idioma, canal, espera"),
                ("L", "Ajustes de la app, fila de idioma elegida"),
                ("l", "Registro de sesión (últimos 100 eventos)"),
                ("?", "Esta pantalla de atajos"),
                ("q", "Salir (pregunta si hay trabajo en curso)"),
            ],
        },
        ShortcutSection {
            title: "Ratón",
            items: &[
                ("Clic", "Seleccionar ajuste, dispositivo o botón"),
                ("", "Otro clic avanza el valor propuesto"),
                ("Rueda", "Mover selección o desplazar"),
                ("Clic der", "Cancelar / volver (Esc)"),
            ],
        },
        ShortcutSection {
            title: "Seguridad",
            items: &[
                ("", "Escritura con consulta previa, confirmación (audio espacial: solo lectura) y lectura coincidente."),
                ("", "Tras un resultado incierto, actualizar antes de escribir."),
                ("", "Elegir modelo no verifica la identidad del hardware."),
            ],
        },
    ];
    l.options_title = " Ajustes de la aplicación ";
    l.options_hint = "Arriba Abajo elige / Izq Der cambia / Enter elige idioma / Esc cierra";
    l.options_language = "Idioma";
    l.options_channel = "Canal RFCOMM";
    l.options_timeout = "Tiempo de espera";
    l.options_target = "Destino";
    l.options_pick_title = " Elegir idioma ";
    l.options_pick_hint = "Arriba Abajo mueve / Enter elige / Esc vuelve";
    l.status_channel_saved = "Canal: {} (se aplica en la próxima conexión)";
    l.status_timeout_saved = "Espera: {} s (se aplica en la próxima conexión)";
    l.edit_title = " Editar dirección de destino ";
    l.edit_lines = &[
        "Escribe dígitos hexadecimales y dos puntos. Retroceso borra; Ctrl-U limpia.",
        "Enter guarda localmente; Esc cancela. No se inicia conexión.",
    ];
    l.model_title = " Confirmar protocolo específico del modelo ";
    l.model_lines = &[
        "Úsalo solo con UGREEN Studio Pro HP206.",
        "Verificado en firmware 0.2.5 (una unidad); otras",
        "variantes y firmware no están verificados.",
        "HiTune Max5c usa otros identificadores y no es compatible.",
        "Esto elige un protocolo; no prueba la identidad del dispositivo.",
        "y / Enter: elegir Studio Pro HP206    n / Esc: cancelar",
    ];
    l.paired_title = " Dispositivos Bluetooth / Arriba Abajo Enter elegir / Esc cerrar ";
    l.paired_empty = "Sin dispositivos emparejados. Esc cierra.";
    l.quit_title = " ¿Salir con trabajo en curso? ";
    l.quit_lines = &[
        "Cancelación pedida; se omitirán las etapas siguientes.",
        "Una escritura en curso puede haberse aplicado.",
        "Al salir nunca se espera al trabajador.",
        "q / Enter: salir ya    Esc: quedarse y esperar el resultado",
    ];
    l.resize_to = "Agranda a al menos {}x{}";
    l.resize_quit = "Esc: cancelar  q: salir";
    l.resize_inflight = "Una escritura puede haberse aplicado. q/Enter: salir; Esc: quedarse";
    l.resize_disabled = "Acciones desactivadas con este tamaño.";
    l.footer = [
        [
            ("a", "dirección"),
            ("p", "dispositivos"),
            ("m", "modelo"),
            ("c", "conectar"),
            ("r", "actualizar"),
            ("d", "desconectar"),
        ],
        [
            ("<", "menos"),
            (">", "más"),
            ("Enter", "aplicar"),
            ("y", "confirmar"),
            ("Esc", "atrás"),
            ("Tab", "pestañas"),
        ],
        [
            ("l", "registro"),
            ("?", "atajos"),
            ("L", "idioma"),
            ("s", "opciones"),
            ("q", "salir"),
            ("", ""),
        ],
    ];
    l.status_startup =
        "Desconectado. Sin Bluetooth aún. Edita la dirección (a) o lee emparejados (p).";
    l.status_loading = "Cargando dispositivos emparejados (sin buscar)...";
    l.status_connecting = "Conectando y leyendo estado y firmware...";
    l.status_refreshing = "Actualizando estado y firmware...";
    l.status_disconnecting = "Desconectando control...";
    l.status_applying =
        "Aplicando {}={} tras consulta previa, verificando confirmación y lectura...";
    l.status_paired_empty =
        "Sin dispositivos emparejados. Empareja en los ajustes Bluetooth primero.";
    l.status_paired_pick = "Elige el dispositivo y pulsa Enter. Elegir no conecta.";
    l.status_read_ok = "Estado leído. Los ajustes mostrados vienen del dispositivo.";
    l.status_firmware_note = "Ajustes leídos; firmware no disponible: {}";
    l.status_blocked_note = " Aún se exige actualizar (r) para desbloquear escrituras.";
    l.status_verified = "Verificado {}={} con confirmación y lectura coincidente.";
    l.status_disconnected = "Control desconectado. Bluetooth no fue modificado.";
    l.status_quit_confirm = "Cancelación pedida. Una escritura puede haberse aplicado. Pulsa q o Enter para salir sin esperar; Esc para quedarse.";
    l.status_cancel_busy = "Cancelación pedida; esperando la etapa actual. Una escritura puede haberse aplicado. Se omitirán las etapas siguientes.";
    l.status_address_saved = "Dirección guardada. Pulsa c para conectar.";
    l.status_bad_address =
        "Escribe seis octetos hexadecimales con dos puntos, p. ej. AA:BB:CC:DD:EE:FF.";
    l.status_model_on = "Protocolo Studio Pro HP206 elegido por ti, sin verificar identidad.";
    l.status_device_picked =
        "Dirección elegida. Confirma el protocolo Studio Pro para conectar (y o Enter).";
    l.status_device_bad = "El dispositivo tiene una dirección inválida; edítala a mano.";
    l.status_cancel_then_disconnect =
        "Cancelación pedida; se desconectará tras la etapa actual. Una escritura puede haberse aplicado.";
    l.status_must_disconnect = "Desconecta (d) antes de cambiar destino o protocolo.";
    l.status_must_connect = "Desconectado; conecta (c) antes de actualizar.";
    l.status_proposed = "Solo propuesta. Enter lo aplica; Esc descarta.";
    l.status_unavailable_setting = "Ajuste ausente en la lectura; la escritura está desactivada.";
    l.status_already_matches = "Ese valor ya coincide con la lectura; no hace falta escribir.";
    l.status_pick_first = "Usa Izquierda/Derecha para proponer un valor antes de aplicar.";
    l.status_disabled =
        "Cambios desactivados. Conecta y actualiza tras cualquier operación incierta.";
    l.status_lang = "Idioma: {}. Pulsa L para cambiar.";
    l.err_gone = "Desconectado; conecta primero.";
    l.err_status_query = "Falló la consulta de estado: {}";
    l.err_paired_list = "Falló la lista de emparejados: {}";
    l.err_need_model = "Confirma el protocolo Studio Pro HP206 antes de conectar.";
    l.err_connected_already = "Desconecta antes de conectar otro destino.";
    l.err_connect = "Falló la conexión: {}";
    l.err_uncertain_blocked = "Escritura bloqueada tras una operación incierta. Actualiza primero.";
    l.err_preflight = "Falló la consulta previa; nada enviado: {}";
    l.err_preflight_unreadable = "La consulta no lee {}; nada enviado.";
    l.err_ack = "Quizá se envió; falló la confirmación: {}. Actualiza antes de escribir.";
    l.err_readback = "Confirmado, pero falló la lectura: {}. Actualiza antes de escribir.";
    l.err_readback_noack = "Enviado, pero falló la lectura: {}. Actualiza antes de escribir.";
    l.err_mismatch = "Confirmado, pero {} no coincide con {}. Actualiza antes de escribir.";
    l.err_mismatch_noack = "Enviado, pero {} no coincide con {}. Actualiza antes de escribir.";
    l.err_cancel_uncertain = "Etapas siguientes canceladas. Una escritura puede haberse aplicado; actualiza antes de escribir.";
    l.err_cancel = "Cancelado; no empezarán más etapas.";
    l.cli_help = "ugreen 0.1.0 — CLI Bluetooth no oficial para UGREEN Studio Pro\n\nUSO\n  ugreen [OPCIONES] COMANDO\n\nCOMANDOS SIN CONEXIÓN\n  help                          Mostrar esta ayuda\n  tui                           Abrir la interfaz de terminal opcional\n  models                        Mostrar compatibilidad y límites\n  commands                      Listar ajustes disponibles\n  decode HEX                    Validar/decodificar bytes capturados sin conexión\n  profile example               Mostrar un perfil de ejemplo\n\nCOMANDOS BLUETOOTH (empareja antes en el sistema)\n  discover                      Listar emparejados; sin búsqueda por radio\n  capture                       Grabar los bytes TX/RX brutos de una sesión como hex de captura\n  status                        Leer batería, firmware y ajustes\n  set CLAVE VALOR               Cambiar un ajuste y verificar lectura\n  profile export                Leer ajustes e imprimir un perfil reutilizable\n  profile apply ARCHIVO         Validar perfil y verificar cada ajuste\n\nOPCIONES (antes del COMANDO)\n  --address XX:XX:XX:XX:XX:XX    Dirección Bluetooth de destino explícita\n  --model studio-pro            Obligatorio para hardware; nunca se detecta solo\n  --channel 1                   Canal RFCOMM, 1–30 (1 por defecto)\n  --timeout 3                   Tiempo por operación en segundos, 1–60\n  --lang es                     Idioma de la interfaz: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (por defecto en)\n  --autoconnect                 Conectar al último dispositivo al iniciar (activado por defecto)\n  --no-autoconnect              Desactiva la conexión automática al iniciar\n  --dry-run                     Mostrar paquetes sin acceder a Bluetooth\n  --help, -h                    Mostrar ayuda\n  --version, -V                 Mostrar versión\n\nEJEMPLOS\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > mi-perfil.conf\n  ugreen --dry-run profile apply mi-perfil.conf\n\nProtocolo Studio Pro HP206 verificado en firmware 0.2.5 (una unidad);\nHiTune Max5c usa otros identificadores de comando. Sin firmware, restablecimiento,\nescritura libre ni localización. Los perfiles pueden aplicarse en parte.\n";
    l.cli_models = "studio-pro: protocolo basado en el código de referencia UGREEN Studio Pro HP206 y su captura\nEstado de prueba de hardware: VERIFICADO en firmware 0.2.5 (Linux, canal 1; ver docs/verification.md)\nHiTune Max5c: NO COMPATIBLE (otros identificadores de comando)\nOtros modelos/firmware UGREEN: SIN VERIFICAR";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Usando protocolo Studio Pro HP206 en {}, canal RFCOMM {}";
    l.cli_need_model =
        "pasa --model studio-pro para elegir explícitamente este protocolo específico";
    l.cli_need_address = "falta --address; usa discover para listar emparejados";
    l.cli_connect_fail = "Falló la conexión Bluetooth: {}. Revisa emparejamiento, encendido, adaptador y si otra app usa el canal de control";
    l.cli_preflight_fail = "falló la consulta previa; ajustes no enviados: {}";
    l.cli_verified = "Verificado {}={}";
    l.cli_apply_fail =
        "{} de {} ajustes verificados antes de fallar en {}={}: {}. Los anteriores no se revierten";
    l.cli_model_note = "model=studio-pro (elegido por ti; sin verificar identidad)";
    l.cli_unknown_value = "desconocido";
    l.cli_firmware_fail = "ajustes leídos, pero falló el firmware: {}";
    l.cli_no_paired =
        "Sin dispositivos emparejados. Empareja los auriculares en Bluetooth primero.";
    l.cli_paired_fail = "falló la lista de emparejados: {}";
    l.cli_dry_run_writes = "Simulación: sin conexión Bluetooth ni escrituras";
    l.cli_bad_model = "solo --model studio-pro está implementado; Max5c es incompatible";
    l.cli_bad_option = "opción desconocida '{}'";
    l.cli_bad_channel = "el canal debe ser 1–30";
    l.cli_bad_timeout = "el tiempo debe ser 1–60 segundos";
    l.cli_need_value = "{} necesita un valor";
    l.cli_unknown_command = "comando desconocido '{}'; ejecuta ugreen --help";
    l.cli_bad_usage = "uso: {}";
    l.cli_bad_decode = "la captura no es una secuencia limpia: {} tramas, {} CRC rechazados, {} bytes descartados, {} tramas desconocidas, {} pendientes";
    l.cli_no_tty = "La TUI exige terminales interactivos en entrada y salida; ejecuta ugreen --help para comandos CLI.";
    l.cli_no_tui_feature =
        "La TUI está desactivada en esta compilación solo-CLI; recompila sin --no-default-features";
    l.cli_tui_dry = "tui no admite --dry-run; usa --dry-run set CLAVE VALOR para ver paquetes";
    l.cli_bad_profile = "el perfil supera 16 KiB";
    l.cli_inflight_after_quit =
        "Una escritura en curso puede haberse aplicado. Actualiza el estado antes de más cambios.";
    l.set_bad_anc = "valor ANC inválido";
    l.set_bad_eq = "valor EQ inválido";
    l.set_bad_onoff = "se esperaba on u off";
    l.set_bad_prompts = "se esperaba voice o beeps";
    l.set_bad_button = "se esperaba none, next o previous";
    l.set_unknown = "ajuste desconocido '{}'\n{}";
    l.profile_big = "el perfil supera 16 KiB";
    l.profile_line = "la línea {} del perfil debe ser clave=valor";
    l.profile_dup = "clave duplicada '{}'";
    l.profile_model = "el modelo del perfil debe ser studio-pro";
    l.profile_need_model = "el perfil exige model=studio-pro";
    l.profile_empty = "el perfil no tiene ajustes";
    l.req_timeout = "sin respuesta válida antes del límite";
    l.req_closed = "conexión Bluetooth cerrada";
    l.req_rejected = "los auriculares rechazaron la instrucción 0x{:02X}";
    l.info_short = "respuesta de información muy corta: {} bytes (mínimo 8)";
    l.fw_short = "la respuesta de firmware no trae versión";
    l.set_ackfail =
        "quizá se envió, pero falló la confirmación: {}; consulta el estado antes de reintentar";
    l.set_rbfail = "confirmada la escritura, pero falló la lectura: {}";
    l.set_rbfail_noack = "escritura enviada, pero falló la lectura: {}";
    l.set_mismatch = "confirmada la escritura, pero {} no coincide con {}; consulta el estado";
    l.set_mismatch_noack = "escritura enviada, pero {} no coincide con {}; consulta el estado";
    l
}
