//! Italian catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.device_ui = [
        "Device features",
        "Available",
        "Unavailable",
        "P: connected devices",
        "No matching features. Change category or search.",
        "Requires Studio Pro firmware 0.2.5.",
        "Inventario socket facoltativo",
        "Non disponibile; non blocca il controllo",
        "Controlli disponibili",
    ];
    l.cli_json_unsupported = "Questo comando non supporta --json.";
    l.status_applying_noack = "Applicazione di {}={} dopo il controllo preliminare e verifica della rilettura (audio spaziale senza conferma)...";
    l.status_verified_noack =
        "{}={} verificato tramite rilettura corrispondente; l’audio spaziale non invia conferma.";
    l.app_title = " UGREEN / controllo Studio Pro ";
    l.connected = "CONNESSO";
    l.disconnected = "DISCONNESSO";
    l.unknown = "SCONOSCIUTO";
    l.target = "Destinazione: ";
    l.target_none = "non selezionata";
    l.protocol = "Protocollo: ";
    l.protocol_on = "Studio Pro HP206 (scelto da te)";
    l.protocol_off = "non selezionato; m conferma Studio Pro HP206";
    l.battery = "Batteria: ";
    l.battery_pct = "{}%";
    l.unavailable = "non disponibile";
    l.codec_line = "Codec: non rilevato";
    l.firmware = "Firmware: ";
    l.snapshot = "Lettura: ";
    l.snapshot_stale = "OBSOLETA / non aggiornata";
    l.snapshot_ago = "letta {}s fa";
    l.snapshot_never = "non letta";
    l.rfcomm_line = "RFCOMM {} | timeout per operazione {}s | fw 0.2.5 verificato";
    l.labels = [
        "ANC",
        "Equalizzatore",
        "Modalità gioco",
        "Audio spaziale",
        "Connessione doppia",
        "Riduzione del vento",
        "Annunci",
        "Azione volume su",
        "Azione volume giù",
    ];
    l.proposed = "  -> {} (proposto)";
    l.settings_title = " Impostazioni / lettura del dispositivo, nessuna scrittura automatica ";
    l.settings_blocked = " Impostazioni / SCRITTURA BLOCCATA: r per aggiornare ";
    l.settings_stale = " Impostazioni / ultima lettura obsoleta ";
    l.status_title = " Stato ";
    l.status_working = " Lavorazione / Esc annulla le fasi successive ";
    l.log_title = " Registro / Su Giù Home Fine / l Esc chiudi ";
    l.shortcuts_title = " Scorciatoie / Su Giù scorre / ? Esc chiudi ";
    l.edit_title = " Modifica indirizzo di destinazione ";
    l.edit_lines = &[
        "Digita cifre esadecimali e due punti. Backspace cancella; Ctrl-U svuota.",
        "Invio salva localmente; Esc annulla. Nessuna connessione viene avviata.",
    ];
    l.model_title = " Conferma il protocollo specifico del modello ";
    l.model_lines = &[
        "Usa solo con UGREEN Studio Pro HP206.",
        "Verificato su firmware retail 0.2.5 (un'unità); altre",
        "varianti e firmware non sono verificati.",
        "HiTune Max5c usa identificativi di comando in conflitto e non è supportato.",
        "Questo sceglie un protocollo; non prova l'identità del dispositivo.",
        "y / Invio: scegli Studio Pro HP206    n / Esc: annulla",
    ];
    l.paired_title = " Appaiati / Su Giù Invio scegli / Esc chiudi ";
    l.paired_empty = "Nessun dispositivo appaiato in cache. Esc chiude.";
    l.quit_title = " Uscire con lavoro in corso? ";
    l.quit_lines = &[
        "Annullamento richiesto; le fasi successive saranno ignorate.",
        "Una scrittura in corso potrebbe ancora completarsi.",
        "Uscire non aspetta mai il worker.",
        "q / Invio: esci ora    Esc: resta e attendi il risultato",
    ];
    l.resize_to = "Ingrandisci ad almeno {}x{}";
    l.resize_quit = "Esc: annulla  q: esci";
    l.resize_inflight = "Una scrittura in corso potrebbe completarsi. q/Invio: esci; Esc: resta";
    l.resize_disabled = "Azioni disattivate a questa dimensione.";
    l.footer = [
        [
            ("a", "indirizzo"),
            ("p", "dispositivi"),
            ("m", "modello"),
            ("c", "connetti"),
            ("r", "aggiorna"),
            ("d", "scollega"),
        ],
        [
            ("<", "meno"),
            (">", "più"),
            ("Invio", "applica"),
            ("y", "conferma"),
            ("Esc", "indietro"),
            ("", ""),
        ],
        [
            ("l", "registro"),
            ("?", "scorciatoie"),
            ("L", "lingua"),
            ("q", "esci"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "Scollegato. Nessun accesso Bluetooth ancora. Modifica l'indirizzo (a) o leggi la cache (p).";
    l.status_loading = "Caricamento dispositivi appaiati in cache (nessuna scansione)...";
    l.status_connecting = "Connessione e lettura di stato e firmware...";
    l.status_refreshing = "Aggiornamento di stato e firmware...";
    l.status_disconnecting = "Disconnessione del controllo...";
    l.status_applying = "Applicazione di {}={} dopo preflight, verifica di conferma e lettura...";
    l.status_paired_empty =
        "Nessun dispositivo appaiato in cache. Appaia prima nelle impostazioni Bluetooth del sistema.";
    l.status_paired_pick =
        "Seleziona il dispositivo desiderato e premi Invio. Selezionare non connette.";
    l.status_read_ok =
        "Stato letto correttamente. Le impostazioni mostrate sono la lettura del dispositivo.";
    l.status_firmware_note = "Impostazioni lette; firmware non disponibile: {}";
    l.status_blocked_note =
        " È ancora richiesto un aggiornamento esplicito (r) per sbloccare le scritture.";
    l.status_verified =
        "{}={} verificato tramite conferma e lettura corrispondente del dispositivo.";
    l.status_disconnected = "Controllo disconnesso. Bluetooth non è stato modificato.";
    l.status_quit_confirm = "Annullamento richiesto. Una scrittura in corso potrebbe completarsi. Premi q o Invio per uscire senza attendere; Esc per restare.";
    l.status_cancel_busy = "Annullamento richiesto; in attesa della fase corrente. Una scrittura in corso potrebbe completarsi. Le fasi successive saranno ignorate.";
    l.status_address_saved = "Indirizzo salvato localmente. Premi c per connettere esplicitamente.";
    l.status_bad_address =
        "Inserisci sei ottetti esadecimali separati da due punti, es. AA:BB:CC:DD:EE:FF.";
    l.status_model_on =
        "Protocollo Studio Pro HP206 scelto da te, identità del dispositivo non verificata.";
    l.status_device_picked =
        "Indirizzo scelto. Conferma il protocollo Studio Pro per connettere (y o Invio).";
    l.status_device_bad = "Il dispositivo in cache ha un indirizzo non valido; modificalo a mano.";
    l.status_cancel_then_disconnect =
        "Annullamento richiesto; la disconnessione seguirà la fase corrente. Una scrittura in corso potrebbe completarsi.";
    l.status_must_disconnect = "Scollega (d) prima di cambiare destinazione o protocollo.";
    l.status_must_connect = "Scollegato; connetti (c) prima di aggiornare.";
    l.status_proposed = "Solo valore proposto. Invio applica; Esc scarta.";
    l.status_unavailable_setting =
        "Questa impostazione è assente nella lettura del dispositivo; la scrittura è disattivata.";
    l.status_already_matches =
        "Questo valore corrisponde già all'ultima lettura; nessuna scrittura necessaria.";
    l.status_pick_first =
        "Usa Sinistra/Destra per scegliere un valore proposto prima di applicare.";
    l.status_disabled =
        "Modifiche disattivate. Connetti e aggiorna esplicitamente dopo ogni operazione incerta.";
    l.status_lang = "Lingua: {}. Premi L per scorrere.";
    l.err_gone = "Scollegato; connetti prima.";
    l.err_status_query = "Query di stato non riuscita: {}";
    l.err_paired_list = "Elenco appaiati in cache non riuscito: {}";
    l.err_need_model = "Conferma il protocollo Studio Pro HP206 prima di connettere.";
    l.err_connected_already = "Scollega prima di connettere un'altra destinazione.";
    l.err_connect = "Connessione non riuscita: {}";
    l.err_uncertain_blocked =
        "Scritture bloccate dopo un'operazione incerta. Aggiorna esplicitamente prima.";
    l.err_preflight = "Stato di preflight non riuscito; nessuna impostazione inviata: {}";
    l.err_preflight_unreadable = "Il preflight non legge {}; nulla inviato.";
    l.err_ack = "L'impostazione potrebbe essere stata inviata; conferma non riuscita: {}. Aggiorna esplicitamente prima di un'altra scrittura.";
    l.err_readback = "Scrittura confermata ma lettura non riuscita: {}. Aggiorna esplicitamente prima di un'altra scrittura.";
    l.err_readback_noack = "Scrittura inviata ma lettura non riuscita: {}. Aggiorna esplicitamente prima di un'altra scrittura.";
    l.err_mismatch = "Scrittura confermata ma {} non corrispondeva a {}. Aggiorna esplicitamente prima di un'altra scrittura.";
    l.err_mismatch_noack = "Scrittura inviata ma {} non corrispondeva a {}. Aggiorna esplicitamente prima di un'altra scrittura.";
    l.err_cancel_uncertain = "Fasi successive annullate. Una scrittura in corso potrebbe essersi completata; aggiorna esplicitamente prima di un'altra scrittura.";
    l.err_cancel = "Annullato; nessuna fase successiva partirà.";
    l.cli_models = "studio-pro: protocollo basato sul codice di riferimento UGREEN Studio Pro HP206 e sulla sua cattura\nStato del test hardware: VERIFICATO su firmware retail 0.2.5 (Linux, canale 1; vedi docs/verification.md)\nHiTune Max5c: NON SUPPORTATO (identificativi di comando in conflitto)\nAltri modelli/firmware UGREEN: NON VERIFICATI";
    l.cli_banner = "Protocollo Studio Pro HP206 su {}, canale RFCOMM {}";
    l.cli_need_model =
        "passa --model studio-pro per scegliere esplicitamente questo protocollo specifico del modello";
    l.cli_need_address = "manca --address; usa discover per elencare i dispositivi appaiati";
    l.cli_connect_fail = "Connessione Bluetooth non riuscita: {}. Verifica appaiamento, alimentazione, adattatore e se un'altra app possiede il canale di controllo";
    l.cli_preflight_fail = "preflight non riuscito; nessuna impostazione inviata: {}";
    l.cli_verified = "Verificato {}={}";
    l.cli_apply_fail = "{} impostazioni su {} verificate prima dell'errore a {}={}: {}. Le impostazioni precedenti non sono ripristinate";
    l.cli_model_note =
        "model=studio-pro (scelto da te; non è una verifica dell'identità del dispositivo)";
    l.cli_unknown_value = "sconosciuto";
    l.cli_firmware_fail = "impostazioni lette, ma query firmware non riuscita: {}";
    l.cli_no_paired =
        "Nessun dispositivo appaiato restituito. Appaia prima le cuffie nelle impostazioni Bluetooth.";
    l.cli_paired_fail = "elenco dispositivi appaiati non riuscito: {}";
    l.cli_dry_run_writes =
        "Simulazione: nessuna connessione Bluetooth né scritture sul dispositivo";
    l.cli_bad_model =
        "solo --model studio-pro è implementato; gli identificativi Max5c sono incompatibili";
    l.cli_bad_option = "opzione sconosciuta '{}'";
    l.cli_bad_channel = "il canale deve essere 1–30";
    l.cli_bad_timeout = "il timeout deve essere 1–60 secondi";
    l.cli_need_value = "{} richiede un valore";
    l.cli_unknown_command = "comando sconosciuto '{}'; esegui ugreen --help";
    l.cli_bad_usage = "uso: {}";
    l.cli_bad_decode = "la cattura non è un flusso di risposta pulito e completo: {} frame, {} CRC rifiutati, {} byte scartati, {} frame sconosciuti, {} byte in sospeso";
    l.cli_no_tty = "La TUI richiede terminali interattivi su stdin e stdout; esegui ugreen --help per i comandi CLI.";
    l.cli_no_tui_feature =
        "La TUI è disattivata in questa build solo-CLI; ricompila senza --no-default-features";
    l.cli_tui_dry =
        "tui non si combina con --dry-run; usa --dry-run set CHIAVE VALORE per pacchetti offline";
    l.cli_bad_profile = "il profilo supera 16 KiB";
    l.cli_inflight_after_quit = "Una scrittura in corso potrebbe essersi completata. Aggiorna esplicitamente lo stato del dispositivo prima di altre modifiche.";
    l.set_bad_anc = "valore ANC non valido";
    l.set_bad_eq = "valore EQ non valido";
    l.set_bad_onoff = "atteso on o off";
    l.set_bad_prompts = "atteso voice o beeps";
    l.set_bad_button = "atteso none, next o previous";
    l.set_unknown = "impostazione sconosciuta '{}'\n{}";
    l.profile_big = "il profilo supera 16 KiB";
    l.profile_line = "la riga {} del profilo deve essere chiave=valore";
    l.profile_dup = "chiave duplicata '{}'";
    l.profile_model = "il modello del profilo deve essere studio-pro";
    l.profile_need_model = "il profilo richiede model=studio-pro";
    l.profile_empty = "il profilo non ha impostazioni";
    l.req_timeout = "nessuna risposta valida prima della scadenza";
    l.req_closed = "connessione Bluetooth chiusa";
    l.req_rejected = "le cuffie hanno rifiutato l'istruzione 0x{:02X}";
    l.info_short = "payload info dispositivo troppo corto: {} byte (servono almeno 8)";
    l.fw_short = "la risposta firmware non contiene byte di versione";
    l.set_ackfail = "l'impostazione potrebbe essere stata inviata ma la conferma non è riuscita: {}; interroga lo stato prima di riprovare";
    l.set_rbfail = "scrittura confermata ma lettura non riuscita: {}";
    l.set_rbfail_noack = "scrittura inviata ma lettura non riuscita: {}";
    l.set_mismatch =
        "scrittura confermata ma la lettura di {} non corrisponde a {}; interroga lo stato";
    l.set_mismatch_noack =
        "scrittura inviata ma la lettura di {} non corrisponde a {}; interroga lo stato";
    // Frammenti dell'aiuto generato; lo
    // schermo è renderizzato dal registro
    // dei comandi.
    l.help_subtitle = "CLI Bluetooth non ufficiale per UGREEN Studio Pro";
    l.help_usage = "UTILIZZO";
    l.help_offline = "COMANDI OFFLINE";
    l.help_bluetooth =
        "COMANDI BLUETOOTH (esegui prima l'accoppiamento nelle impostazioni di sistema)";
    l.help_capture_gated = "RICHIESTE CON GATE (attualmente rifiutate prima della connessione)";
    l.help_options = "OPZIONI (prima del COMANDO)";
    l.help_examples = "ESEMPI";
    l.help_footer = "Protocollo Studio Pro HP206 verificato su firmware retail 0.2.5 (un'unità);\nHiTune Max5c usa identificativi di comando in conflitto. Non sono previste azioni di\nfirmware, reset, scrittura raw né ricerca degli auricolari. I profili possono applicarsi parzialmente in caso di errore.";
    l.opt_address = "Indirizzo Bluetooth esplicito della destinazione";
    l.opt_model = "Obbligatorio per le richieste hardware; mai rilevato automaticamente";
    l.opt_channel = "Canale RFCOMM, 1–30 (predefinito 1)";
    l.opt_timeout = "Timeout per operazione in secondi, 1–60";
    l.opt_lang =
        "Lingua dell'interfaccia: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (predefinito en)";
    l.opt_autoconnect = "Connetti all'ultimo dispositivo all'avvio (predefinito attivo)";
    l.opt_no_autoconnect = "Disattiva la connessione automatica all'avvio";
    l.opt_json = "JSON con versione per i comandi di sola lettura supportati";
    l.opt_dry_run = "Stampa i pacchetti delle impostazioni senza accesso Bluetooth";
    l.opt_help = "Mostra l'aiuto";
    l.opt_version = "Mostra la versione";
    l.cmd_help = "Mostra questo aiuto";
    l.cmd_tui = "Apri l'interfaccia terminale interativa opzionale";
    l.cmd_models = "Mostra la compatibilità del protocollo e i limiti";
    l.cmd_commands = "Elenca le impostazioni supportate";
    l.cmd_decode = "Valida/decodifica offline byte di risposta catturati";
    l.cmd_analyze = "Interpreta offline i risultati statici dell'APK";
    l.cmd_candidate = "Stampa un candidato di pacchetto statico; non viene mai inviato";
    l.cmd_profile = "Gestisci i profili di impostazioni";
    l.cmd_profile_example = "Stampa un profilo di impostazioni di esempio";
    l.cmd_profile_export = "Leggi le impostazioni e stampa un profilo riutilizzabile";
    l.cmd_profile_apply = "Valida il profilo, applica e verifica ogni impostazione";
    l.cmd_discover = "Elenca i dispositivi accoppiati nella cache; nessuna scansione radio";
    l.cmd_capture = "Registra i byte TX/RX raw di una sessione come hex di test";
    l.cmd_status = "Leggi batteria, firmware e impostazioni";
    l.cmd_set = "Modifica un'impostazione, poi verifica con la lettura";
    l.cmd_query = "Richiesta con gate; non verificata su questo firmware";
    l.cmd_control = "Controllo con gate; --dry-run stampa i byte";
    l.cmd_find = "Tono di ricerca con gate";
    l.navigation = [
        "Home",
        "Audio",
        "Dispositivi",
        "Impostazioni",
        "Diagnostica",
        "Ricerca",
    ];
    l.headset_connections = ["Connessioni delle cuffie", "Questi dispositivi sono collegati alle cuffie, distinti dai dispositivi associati a questo PC.", "La richiesta HP206 (0x0D) richiede acquisizioni dell’app ufficiale per questo firmware. Nessuna richiesta inviata; numero e identità sconosciuti.", "Disconnessione, riconnessione e cambio dei dispositivi remoti non supportati. Le azioni Bluetooth host riguardano solo questo PC.", "r: verifica · d: doppia connessione · Esc: indietro"];
    l.headset_connections[4] = "r: aggiorna stato doppio · d: doppia connessione · Esc: indietro";
    l.page_labels = [
        "Dispositivi",
        "Audio",
        "Impostazioni cuffie",
        "Multipoint",
        "Funzioni sperimentali",
        "Diagnostica",
    ];
    l.page_hint = "↑/↓ Invio: apri · r: aggiorna · Esc: indietro";
    l.host_commands = [
        "Osserva il trasporto audio del sistema",
        "Elenca le uscite audio locali",
        "Mostra le prove delle funzioni",
        "Leggi la diagnostica locale",
        "Connetti il collegamento Bluetooth locale",
        "Disconnetti il collegamento Bluetooth locale",
    ];
    l.cmd_audio_devices = "Elenca le uscite audio di Windows";
    l.features_title = " Funzionalità ";
    l.features_search = "Cerca: ";
    l.features_all = "Tutto";
    l.feature_categories = [
        "Audio",
        "Connessione",
        "Ambiente",
        "Risposte",
        "Informazioni del dispositivo",
        "Multipunto",
        "Codec",
        "Audio avanzato",
        "Controlli",
        "Condivisione audio",
        "Ricerca",
        "Gestione del dispositivo",
        "Diagnostica",
    ];
    l.feature_names = [
        "ANC",
        "Equalizzatore",
        "Modalità gioco",
        "Audio spaziale",
        "Connessione doppia",
        "Riduzione del vento",
        "Avvisi vocali",
        "Azione volume su",
        "Azione volume giù",
        "Dispositivi multipunto",
        "Codec audio",
        "ID del headset",
        "Colore del headset",
        "MAC della custodia",
        "Controllo LDAC",
        "LDAC per un dispositivo",
        "Decodifica di alta qualità",
        "Comando volume",
        "Canale mono/stereo",
        "Mappatura dei tasti",
        "Azioni IA",
        "Controllo IA",
        "Disattiva condivisione",
        "Condividi con un dispositivo",
        "Condividi con un dispositivo scelto",
        "Trova l'auricolare sinistro",
        "Trova l'auricolare destro",
        "Ripristino di fabbrica",
        "Decodificatore di catture",
        "Analisi delle catture",
        "Anteprima del pacchetto",
        "Disconnetti e disattiva modalità doppia",
    ];
    l.feature_descriptions = [
        "Profondità di riduzione attiva del rumore. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Preimpostazione dell'equalizzatore. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Interruttore a bassa latenza della modalità gioco. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Interruttore audio spaziale. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Interruttore di connessione a due dispositivi. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Interruttore di riduzione del rumore del vento. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Stile degli avvisi vocali. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Azione volume su tenuta premuta. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Azione volume giù tenuta premuta. Impostazione verificata; modificala nello schermo Impostazioni.",
        "Leggere l'elenco dei dispositivi multipunto connessi. Il protocollo di accoppiamento richiede ancora la verifica hardware.",
        "Leggere il codec audio Bluetooth attivo. Il formato di query richiede ancora la verifica hardware.",
        "Leggere l'ID hardware del headset. Il formato di query richiede ancora la verifica hardware.",
        "Leggere il colore del headset. Il formato di query richiede ancora la verifica hardware.",
        "Leggere la MAC della custodia di ricarica. Il formato di query richiede ancora la verifica hardware.",
        "Commutare la codifica LDAC sulla connessione attuale. Il formato di controllo richiede ancora la verifica hardware.",
        "Commutare la codifica LDAC per un dispositivo accoppiato. Il formato di controllo richiede ancora la verifica hardware.",
        "Commutare la decodifica audio di alta qualità. Il formato di controllo richiede ancora la verifica hardware.",
        "Inviare un byte di volume assoluto. L'intervallo hardware non è verificato.",
        "Cambiare il canale audio tra mono e stereo. Il formato di controllo richiede ancora la verifica hardware.",
        "Riassociare le otto azioni di pressione prolungata. Il formato di controllo richiede ancora la verifica hardware.",
        "Riassociare le quattro azioni attivate dall'IA. Il formato di controllo richiede ancora la verifica hardware.",
        "Commutare il controllo audio tramite IA. Il formato di controllo richiede ancora la verifica hardware.",
        "Disattivare la condivisione audio. Il formato di controllo richiede ancora la verifica hardware.",
        "Condividere l'audio con l'unico dispositivo connesso. Il formato di controllo richiede ancora la verifica hardware.",
        "Condividere l'audio con un dispositivo scelto. Il formato di controllo richiede ancora la verifica hardware.",
        "Riprodurre un tono di localizzazione sull'auricolare sinistro tramite l'host. Il protocollo di ricerca richiede ancora la verifica hardware.",
        "Riprodurre un tono di localizzazione sull'auricolare destro tramite l'host. Il protocollo di ricerca richiede ancora la verifica hardware.",
        "Rimuovere l'accoppiamento e ripristinare i valori di fabbrica. Distruttivo; il formato di controllo richiede ancora la verifica hardware.",
        "Decodificare byte di risposta catturati offline; nessun accesso Bluetooth.",
        "Interpretare un flusso di cattura completo offline; nessun accesso Bluetooth.",
        "Stampare il pacchetto statico che un controllo invierebbe; non viene mai trasmesso.",
        "Disconnette il dispositivo scelto e disattiva la modalità doppia. Sperimentale; confermare indice e MAC con una nuova lettura.",
    ];
    l.feature_available = "Disponibile";
    l.feature_pending = "Verifica in attesa";
    l.feature_pending_reason =
        "Il formato via cavo richiede ancora la verifica hardware con l'app ufficiale.";
    l.feature_live_disabled =
        "L'esecuzione dal vivo è disattivata fino a prova verificata sul dispositivo.";
    l.feature_offline_workflow = "Solo analisi offline; nulla viene inviato al headset.";
    l.feature_offline_hint = "Esegui: ugreen {}";
    l.feature_kind_setting = "Impostazione verificata";
    l.feature_kind_query = "Query del dispositivo";
    l.feature_kind_control = "Controllo del dispositivo";
    l.feature_kind_offline = "Analisi offline";
    l.feature_category_label = "Categoria: {}";
    l.feature_current = "Attuale: {}";
    l.feature_edit_in_settings = "Modifica in Impostazioni (s)";
    l.feature_run_query = "Invio: esegui query";
    l.feature_configure = "Invio: configura";
    l.feature_back = "Esc: indietro";
    l.feature_form_title = " Configura {} ";
    l.feature_err_required = "Compila ogni campo prima della revisione";
    l.feature_field_unset = "non impostato";
    l.feature_form_hint = "Su Giù campo / Sinistra Destra scelta / Invio verifica / Esc indietro";
    l.feature_review_title = " Verifica {} ";
    l.review_write_title = " Verifica scrittura ";
    l.feature_review_target = "Destinazione: {}";
    l.feature_review_previous = "Attuale: {}";
    l.feature_review_proposed = "Proposto: {}";
    l.feature_review_values = "Valori: {}";
    l.feature_review_readback = "Lettura: {}";
    l.feature_readback_verified = "verificato da conferma e lettura corrispondente";
    l.feature_readback_noack =
        "verificato da lettura corrispondente (l'audio spaziale non invia conferma)";
    l.feature_readback_unconfirmed =
        "non confermato in modo indipendente; aggiorna dopo l'applicazione";
    l.feature_readback_query = "query in sola lettura";
    l.feature_confirm_apply = "Invio: applica    Esc: annulla";
    l.feature_confirm_reset =
        "Distruttivo: rimuove l'accoppiamento dell'host e ripristina i valori di fabbrica.";
    l.features_footer = [
        [
            ("Su", "sposta"),
            ("Giù", "sposta"),
            ("Tab", "categoria"),
            ("Invio", "apri"),
            ("Esc", "indietro"),
            ("", ""),
        ],
        [
            ("lettere", "cerca"),
            ("Backspace", "cancella"),
            ("Ctrl+U", "svuota"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "chiudi"),
            ("?", "scorciatoie"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "Esecuzione dell'operazione di funzionalità...";
    l.status_edit_in_settings =
        "{} è un'impostazione verificata; modificala nello schermo Impostazioni (F4).";
    l.codec_unverified = "query non verificata";
    l.arg_enabled = "Attivato";
    l.arg_side = "Lato";
    l.arg_duration = "Durata (s)";
    l.arg_target = "Destinazione";
    l.arg_value = "Valore (0-255)";
    l.arg_mode = "Modalità";
    l.arg_peer = "Accoppiato";
    l.arg_actions = "Azioni";
    l.arg_address = "MAC dell'accoppiato";
    l.arg_hint_toggle = "on o off";
    l.arg_hint_side = "sinistra o destra";
    l.arg_hint_seconds = "1..30 secondi";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "sei ottetti esadecimali separati da due punti";
    l.arg_hint_peer = "1 o 2";
    l.arg_hint_actions = "{} valori di azione separati da virgole";
    l.tui_no_tty =
        "L'interfaccia richiede terminali interattivi stdin e stdout; esegui ugreen --help per i comandi della CLI.";
    l.tui_bad_channel =
        "Il canale dell'interfaccia deve essere 1-30 e il timeout maggiore di zero e al massimo 60 secondi.";
    l.status_worker_busy = "Un'operazione è già in esecuzione; attendi o premi Esc per annullare.";
    l.status_worker_sequence = "Sequenza di richieste esaurita; riavvia l'interfaccia.";
    l.status_worker_stopped = "Il worker Bluetooth si è arrestato; riavvia l'interfaccia.";
    l.status_worker_queue = "La coda del worker è piena; attendi prima di riprovare.";
    l.status_reset_unavailable = "destinazione del ripristino non disponibile";
    l.status_reset_done =
        "Ripristino confermato; accoppiamento host e dispositivo salvato rimossi.";
    l.status_inflight_write =
        "Una scrittura in corso potrebbe essere stata completata; aggiorna dopo il riavvio.";
    l.page_labels[4] = l.navigation[5];
    l
}
