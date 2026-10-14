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
                ("F", "Comandos del auricular (requieren verificación)"),
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
    l.cli_models = "studio-pro: protocolo basado en el código de referencia UGREEN Studio Pro HP206 y su captura\nEstado de prueba de hardware: VERIFICADO en firmware 0.2.5 (Linux, canal 1; ver docs/verification.md)\nHiTune Max5c: NO COMPATIBLE (otros identificadores de comando)\nOtros modelos/firmware UGREEN: SIN VERIFICAR";
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
    // Fragmentos de ayuda generada; la pantalla
    // se renderiza desde el registro de comandos.
    l.help_subtitle = "CLI Bluetooth no oficial para UGREEN Studio Pro";
    l.help_usage = "USO";
    l.help_offline = "COMANDOS SIN CONEXIÓN";
    l.help_bluetooth = "COMANDOS BLUETOOTH (empareja primero en los ajustes del sistema)";
    l.help_capture_gated =
        "CONSULTAS CON PUERTA DE ENLACE (actualmente se rechazan antes de conectar)";
    l.help_options = "OPCIONES (antes del COMANDO)";
    l.help_examples = "EJEMPLOS";
    l.help_footer = "Protocolo Studio Pro HP206 verificado en firmware retail 0.2.5 (una unidad);\nHiTune Max5c usa identificadores de comando en conflicto. No se ofrecen acciones de\nfirmware, reinicio, escritura sin procesar ni búsqueda de auriculares. Los perfiles pueden aplicarse parcialmente ante un error.";
    l.opt_address = "Dirección Bluetooth explícita del destino";
    l.opt_model = "Obligatorio para solicitudes de hardware; nunca se detecta automáticamente";
    l.opt_channel = "Canal RFCOMM, 1–30 (por defecto 1)";
    l.opt_timeout = "Tiempo de espera por operación en segundos, 1–60";
    l.opt_lang =
        "Idioma de la interfaz: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (por defecto en)";
    l.opt_autoconnect = "Conectar al último dispositivo al iniciar (por defecto activado)";
    l.opt_no_autoconnect = "Desactivar la conexión automática al iniciar";
    l.opt_json = "JSON versionado para comandos de solo lectura compatibles";
    l.opt_dry_run = "Imprimir los paquetes de ajustes sin acceso Bluetooth";
    l.opt_help = "Mostrar ayuda";
    l.opt_version = "Mostrar versión";
    l.cmd_help = "Mostrar esta ayuda";
    l.cmd_tui = "Abrir la interfaz de terminal interactiva opcional";
    l.cmd_models = "Mostrar compatibilidad de protocolo y límites";
    l.cmd_commands = "Listar los ajustes admitidos";
    l.cmd_decode = "Validar/decodificar bytes de respuesta capturados sin conexión";
    l.cmd_analyze = "Interpretar hallazgos estáticos del APK sin conexión";
    l.cmd_candidate = "Imprimir un candidato de paquete estático; nunca se envía";
    l.cmd_profile = "Gestionar perfiles de ajustes";
    l.cmd_profile_example = "Imprimir un perfil de ajustes de ejemplo";
    l.cmd_profile_export = "Leer los ajustes e imprimir un perfil reutilizable";
    l.cmd_profile_apply = "Validar el perfil, aplicar y verificar cada ajuste";
    l.cmd_discover = "Listar dispositivos emparejados en caché; sin escaneo de radio";
    l.cmd_capture = "Grabar los bytes TX/RX brutos de una sesión como hexadecimal de prueba";
    l.cmd_status = "Leer batería, firmware y ajustes";
    l.cmd_set = "Cambiar un ajuste y luego verificar la lectura";
    l.cmd_query = "Consulta con puerta de enlace; sin verificar en este firmware";
    l.cmd_control = "Control con puerta de enlace; --dry-run imprime los bytes";
    l.cmd_find = "Tono de búsqueda con puerta de enlace";
    l.headset_connections = ["Conexiones del auricular", "Son los dispositivos conectados al auricular, distintos de los emparejados con este PC.", "La consulta de pares HP206 (0x0D) necesita capturas de la aplicación oficial para este firmware. No se envió ninguna consulta; la cantidad y las identidades son desconocidas.", "Desconectar, reconectar y cambiar pares remotos no está soportado. Las acciones Bluetooth del host solo afectan a este PC.", "r: comprobar disponibilidad · d: conexión dual · Esc: volver"];
    l.page_labels = [
        "Dispositivos",
        "Audio",
        "Ajustes del auricular",
        "Multipunto",
        "Funciones experimentales",
        "Diagnóstico",
    ];
    l.page_hint = "↑/↓ Enter: abrir · r: actualizar · Esc: volver";
    l.host_commands = [
        "Observar transporte de audio del sistema",
        "Listar salidas de audio del host",
        "Mostrar evidencia de funciones",
        "Leer diagnóstico del host",
        "Conectar enlace Bluetooth del host",
        "Desconectar enlace Bluetooth del host",
    ];
    l.cmd_audio_devices = "Listar salidas de audio de Windows";
    l.features_title = " Funciones ";
    l.features_search = "Buscar: ";
    l.features_all = "Todo";
    l.feature_categories = [
        "Audio",
        "Conexión",
        "Entorno",
        "Respuestas",
        "Información del dispositivo",
        "Multipunto",
        "Códecs",
        "Audio avanzado",
        "Controles",
        "Uso compartido de audio",
        "Búsqueda",
        "Gestión del dispositivo",
        "Diagnóstico",
    ];
    l.feature_names = [
        "ANC",
        "Ecualizador",
        "Modo juego",
        "Audio espacial",
        "Conexión dual",
        "Reducción de viento",
        "Avisos de voz",
        "Acción subir volumen",
        "Acción bajar volumen",
        "Dispositivos multipunto",
        "Códec de audio",
        "ID del auricular",
        "Color del auricular",
        "MAC del estuche",
        "Control LDAC",
        "LDAC para un emparejado",
        "Decodificación de alta calidad",
        "Comando de volumen",
        "Canal mono/stéreo",
        "Asignación de botones",
        "Acciones de IA",
        "Control de IA",
        "Desactivar uso compartido",
        "Compartir con un emparejado",
        "Compartir con un emparejado elegido",
        "Buscar auricular izquierdo",
        "Buscar auricular derecho",
        "Restablecimiento de fábrica",
        "Decodificador de capturas",
        "Análisis de capturas",
        "Vista previa de paquete",
    ];
    l.feature_descriptions = [
        "Profundidad de cancelación activa de ruido. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Preajuste del ecualizador. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Alternador de baja latencia del modo juego. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Alternador de audio espacial. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Alternador de conexión a dos dispositivos. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Alternador de reducción de ruido de viento. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Estilo de los avisos de voz. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Acción de mantener subir volumen. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Acción de mantener bajar volumen. Ajuste verificado; edítalo en la pantalla de Ajustes.",
        "Leer la lista de emparejados multipunto. El protocolo de emparejados aún requiere verificación en hardware.",
        "Leer el códec de audio Bluetooth activo. El formato de consulta aún requiere verificación en hardware.",
        "Leer el ID de hardware del auricular. El formato de consulta aún requiere verificación en hardware.",
        "Leer el color del auricular. El formato de consulta aún requiere verificación en hardware.",
        "Leer la MAC del estuche de carga. El formato de consulta aún requiere verificación en hardware.",
        "Alternar la codificación LDAC en la conexión actual. El formato de control aún requiere verificación en hardware.",
        "Alternar la codificación LDAC para un emparejado. El formato de control aún requiere verificación en hardware.",
        "Alternar la decodificación de audio de alta calidad. El formato de control aún requiere verificación en hardware.",
        "Enviar un byte de volumen absoluto. El rango de hardware no está verificado.",
        "Cambiar el canal de audio entre mono y stéreo. El formato de control aún requiere verificación en hardware.",
        "Reasignar las ocho acciones de mantener. El formato de control aún requiere verificación en hardware.",
        "Reasignar las cuatro acciones activadas por IA. El formato de control aún requiere verificación en hardware.",
        "Alternar el control de audio por IA. El formato de control aún requiere verificación en hardware.",
        "Desactivar el uso compartido de audio. El formato de control aún requiere verificación en hardware.",
        "Compartir audio con el único emparejado conectado. El formato de control aún requiere verificación en hardware.",
        "Compartir audio con un emparejado elegido. El formato de control aún requiere verificación en hardware.",
        "Reproducir un tono de localización en el auricular izquierdo desde el host. El protocolo de búsqueda aún requiere verificación en hardware.",
        "Reproducir un tono de localización en el auricular derecho desde el host. El protocolo de búsqueda aún requiere verificación en hardware.",
        "Quitar el emparejamiento y restaurar los valores de fábrica. Destructivo; el formato de control aún requiere verificación en hardware.",
        "Decodificar bytes de respuesta capturados sin conexión; sin acceso Bluetooth.",
        "Interpretar un flujo de captura completo sin conexión; sin acceso Bluetooth.",
        "Imprimir el paquete estático que enviaría un control; nunca se transmite.",
    ];
    l.feature_available = "Disponible";
    l.feature_pending = "Pendiente de verificación";
    l.feature_pending_reason =
        "El formato de cable aún requiere verificación en hardware con la app oficial.";
    l.feature_live_disabled =
        "La ejecución en vivo está desactivada hasta que exista evidencia verificada en el dispositivo.";
    l.feature_offline_workflow = "Solo análisis sin conexión; nada se envía al auricular.";
    l.feature_offline_hint = "Ejecutar: ugreen {}";
    l.feature_kind_setting = "Ajuste verificado";
    l.feature_kind_query = "Consulta de dispositivo";
    l.feature_kind_control = "Control de dispositivo";
    l.feature_kind_offline = "Análisis sin conexión";
    l.feature_category_label = "Categoría: {}";
    l.feature_current = "Actual: {}";
    l.feature_edit_in_settings = "Editar en Ajustes (s)";
    l.feature_run_query = "Enter: ejecutar consulta";
    l.feature_configure = "Enter: configurar";
    l.feature_back = "Esc: volver";
    l.feature_form_title = " Configurar {} ";
    l.feature_err_required = "Todos los campos deben completarse antes de revisar";
    l.feature_field_unset = "sin definir";
    l.feature_form_hint = "Arriba Abajo campo / Izq Der opción / Enter revisar / Esc volver";
    l.feature_review_title = " Revisar {} ";
    l.review_write_title = " Revisar escritura ";
    l.feature_review_target = "Destino: {}";
    l.feature_review_previous = "Actual: {}";
    l.feature_review_proposed = "Propuesto: {}";
    l.feature_review_values = "Valores: {}";
    l.feature_review_readback = "Lectura: {}";
    l.feature_readback_verified = "verificado por confirmación y lectura coincidente";
    l.feature_readback_noack =
        "verificado por lectura coincidente (el audio espacial no envía confirmación)";
    l.feature_readback_unconfirmed =
        "no confirmado de forma independiente; actualiza después de aplicar";
    l.feature_readback_query = "consulta de solo lectura";
    l.feature_confirm_apply = "Enter: aplicar    Esc: cancelar";
    l.feature_confirm_reset =
        "Destructivo: quita el emparejamiento del host y restaura los valores de fábrica.";
    l.features_footer = [
        [
            ("Arriba", "mover"),
            ("Abajo", "mover"),
            ("Tab", "categoría"),
            ("Enter", "abrir"),
            ("Esc", "volver"),
            ("", ""),
        ],
        [
            ("letras", "buscar"),
            ("Retroceso", "borrar"),
            ("Ctrl+U", "limpiar"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "cerrar"),
            ("?", "atajos"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "Ejecutando operación de funciones...";
    l.status_edit_in_settings =
        "{} es un ajuste verificado; edítalo en la pantalla de Ajustes (s).";
    l.codec_unverified = "consulta no verificada";
    l.arg_enabled = "Activado";
    l.arg_side = "Lado";
    l.arg_duration = "Duración (s)";
    l.arg_target = "Objetivo";
    l.arg_value = "Valor (0-255)";
    l.arg_mode = "Modo";
    l.arg_peer = "Emparejado";
    l.arg_actions = "Acciones";
    l.arg_address = "MAC del emparejado";
    l.arg_hint_toggle = "on o off";
    l.arg_hint_side = "izquierda o derecha";
    l.arg_hint_seconds = "1..30 segundos";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "seis octetos hexadecimales separados por dos puntos";
    l.arg_hint_peer = "1 o 2";
    l.arg_hint_actions = "{} valores de acción separados por comas";
    l.tui_no_tty =
        "La interfaz requiere terminales interactivas de entrada y salida; ejecuta ugreen --help para los comandos de la CLI.";
    l.tui_bad_channel =
        "El canal de la interfaz debe ser 1-30 y la espera mayor que cero y como máximo 60 segundos.";
    l.status_worker_busy = "Ya hay una operación en ejecución; espera o pulsa Esc para cancelar.";
    l.status_worker_sequence = "Secuencia de solicitudes agotada; reinicia la interfaz.";
    l.status_worker_stopped = "El trabajador de Bluetooth se detuvo; reinicia la interfaz.";
    l.status_worker_queue = "La cola del trabajador está llena; espera antes de reintentar.";
    l.status_reset_unavailable = "objetivo de restablecimiento no disponible";
    l.status_reset_done =
        "Restablecimiento confirmado; se quitaron el emparejamiento del host y el dispositivo guardado.";
    l.status_inflight_write =
        "Una escritura en curso puede haberse completado; actualiza después de reiniciar.";
    l
}
