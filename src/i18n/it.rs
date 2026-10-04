//! Italian catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
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
    l.cli_help = "ugreen 0.1.0 — CLI Bluetooth non ufficiale per UGREEN Studio Pro\n\nUSO\n  ugreen [OPZIONI] COMANDO\n\nCOMANDI OFFLINE\n  help                          Mostra questo aiuto\n  tui                           Apri l'interfaccia terminale opzionale\n  models                        Mostra compatibilità e limiti del protocollo\n  commands                      Elenca le impostazioni supportate\n  decode HEX                    Valida/decodifica byte di risposta catturati offline\n  profile example               Mostra un profilo di impostazioni di esempio\n\nCOMANDI BLUETOOTH (appaia prima nelle impostazioni di sistema)\n  discover                      Elenca dispositivi appaiati in cache; nessuna scansione radio\n  capture                       Registra i byte TX/RX grezzi di una sessione come esadecimale\n  status                        Leggi batteria, firmware e impostazioni\n  set CHIAVE VALORE             Cambia un'impostazione e verifica la lettura\n  profile export                Leggi le impostazioni e stampa un profilo riutilizzabile\n  profile apply FILE            Valida il profilo, applica e verifica ogni impostazione\n\nOPZIONI (prima del COMANDO)\n  --address XX:XX:XX:XX:XX:XX    Indirizzo Bluetooth di destinazione esplicito\n  --model studio-pro            Richiesto per richieste hardware; mai rilevato automaticamente\n  --channel 1                   Canale RFCOMM, 1–30 (predefinito 1)\n  --timeout 3                   Timeout per operazione in secondi, 1–60\n  --lang it                     Lingua dell'interfaccia: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (predefinito en)\n  --autoconnect                 La TUI connette la destinazione memorizzata all'avvio\n  --no-autoconnect              Disattiva la connessione automatica all'avvio\n  --dry-run                     Mostra i pacchetti di impostazioni senza accesso Bluetooth\n  --help, -h                    Mostra l'aiuto\n  --version, -V                 Mostra la versione\n\nESEMPI\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > mio-profilo.conf\n  ugreen --dry-run profile apply mio-profilo.conf\n\nProtocollo Studio Pro HP206 verificato su firmware retail 0.2.5 (un'unità);\nHiTune Max5c usa identificativi di comando in conflitto. Nessuna azione di\nfirmware, ripristino, scrittura bruta o ricerca. I profili possono applicarsi\nparzialmente in caso di errore.\n";
    l.cli_models = "studio-pro: protocollo basato sul codice di riferimento UGREEN Studio Pro HP206 e sulla sua cattura\nStato del test hardware: VERIFICATO su firmware retail 0.2.5 (Linux, canale 1; vedi docs/verification.md)\nHiTune Max5c: NON SUPPORTATO (identificativi di comando in conflitto)\nAltri modelli/firmware UGREEN: NON VERIFICATI";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
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
    l
}
