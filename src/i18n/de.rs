//! German catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / Studio Pro-Steuerung ";
    l.connected = "VERBUNDEN";
    l.disconnected = "GETRENNT";
    l.target = "Ziel: ";
    l.target_none = "nicht gewählt";
    l.protocol = "Protokoll: ";
    l.protocol_on = "Studio Pro HP206 (vom Benutzer gewählt)";
    l.protocol_off = "nicht gewählt; m bestätigt Studio Pro HP206";
    l.battery = "Akku: ";
    l.battery_pct = "{}%";
    l.unavailable = "nicht verfügbar";
    l.codec_line = "Codec: nicht verfügbar (nicht freigegeben)";
    l.firmware = "Firmware: ";
    l.snapshot = "Auslesen: ";
    l.snapshot_stale = "VERALTET / nicht aktuell";
    l.snapshot_ago = "vor {}s ausgelesen";
    l.snapshot_never = "nicht ausgelesen";
    l.rfcomm_line = "RFCOMM {} | Timeout pro Operation {}s | Retail-fw 0.2.5 geprüft";
    l.labels = [
        "ANC",
        "Equalizer",
        "Spielmodus",
        "Spatialaudio",
        "Doppelverbindung",
        "Windreduzierung",
        "Ansagen",
        "Lautstärke-hoch-Aktion",
        "Lautstärke-runter-Aktion",
    ];
    l.proposed = "  -> {} (vorgeschlagen)";
    l.settings_title = " Einstellungen / Geräteauslesung, keine automatischen Schreibvorgänge ";
    l.settings_blocked = " Einstellungen / SCHREIBEN GESPERRT: r zum Aktualisieren ";
    l.settings_stale = " Einstellungen / letzte Auslesung veraltet ";
    l.status_title = " Status ";
    l.status_working = " Arbeitend / Esc bricht spätere Schritte ab ";
    l.log_title = " Sitzungsprotokoll / Hoch Runter Pos1 Ende / l Esc schließen ";
    l.shortcuts_title = " Tasten / Hoch Runter scrollen / ? Esc schließen ";
    l.edit_title = " Zieladresse bearbeiten ";
    l.edit_lines = &[
        "Hexadezimalziffern und Doppelpunkte eingeben. Rücktaste löscht; Ctrl-U leert.",
        "Enter speichert lokal; Esc bricht ab. Keine Verbindung wird gestartet.",
    ];
    l.model_title = " Modellspezifisches Protokoll bestätigen ";
    l.model_lines = &[
        "Nur mit UGREEN Studio Pro HP206 verwenden.",
        "Geprüft auf Retail-Firmware 0.2.5 (ein Gerät); andere",
        "Varianten und Firmware sind nicht geprüft.",
        "HiTune Max5c verwendet konfliktreiche Befehls-IDs und ist nicht unterstützt.",
        "Dies wählt ein Protokoll; es beweist keine Geräteidentität.",
        "y / Enter: Studio Pro HP206 wählen    n / Esc: abbrechen",
    ];
    l.paired_title = " Paarungen / Hoch Runter Enter wählen / Esc schließen ";
    l.paired_empty = "Keine gepaarten Geräte im Cache. Esc schließt.";
    l.quit_title = " Beenden während Arbeit läuft? ";
    l.quit_lines = &[
        "Abbruch angefordert; spätere Schritte werden übersprungen.",
        "Ein laufender Schreibvorgang kann noch abgeschlossen werden.",
        "Beenden wartet nie auf den Worker.",
        "q / Enter: jetzt beenden    Esc: bleiben und Ergebnis abwarten",
    ];
    l.resize_to = "Auf mindestens {}x{} vergrößern";
    l.resize_quit = "Esc: abbrechen  q: beenden";
    l.resize_inflight =
        "Laufender Schreibvorgang kann abgeschlossen werden. q/Enter: beenden; Esc: bleiben";
    l.resize_disabled = "Aktionen bei dieser Größe deaktiviert.";
    l.footer = [
        [
            ("a", "adresse"),
            ("p", "geräte"),
            ("m", "modell"),
            ("c", "verbinden"),
            ("r", "aktualisieren"),
            ("d", "trennen"),
        ],
        [
            ("<", "weniger"),
            (">", "mehr"),
            ("Enter", "anwenden"),
            ("y", "bestätigen"),
            ("Esc", "zurück"),
            ("", ""),
        ],
        [
            ("l", "protokoll"),
            ("?", "tasten"),
            ("L", "sprache"),
            ("q", "beenden"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "Getrennt. Noch kein Bluetooth-Zugriff. Adresse bearbeiten (a) oder Paarungs-Cache lesen (p).";
    l.status_loading = "Gepaarte Geräte aus dem Cache laden (kein Scan)...";
    l.status_connecting = "Verbinde und lese Status und Firmware...";
    l.status_refreshing = "Aktualisiere Status und Firmware...";
    l.status_disconnecting = "Trenne...";
    l.status_applying = "Wende {}={} nach Preflight an, prüfe Bestätigung und Auslesung...";
    l.status_paired_empty =
        "Keine gepaarten Geräte im Cache. Zuerst in den Bluetooth-Einstellungen paaren.";
    l.status_paired_pick = "Gewünschtes Gerät wählen und Enter drücken. Wählen verbindet nicht.";
    l.status_read_ok = "Status erfolgreich gelesen. Gezeigte Einstellungen sind Geräteauslesung.";
    l.status_firmware_note = "Einstellungen gelesen; Firmware nicht verfügbar: {}";
    l.status_blocked_note = " Explizites Aktualisieren (r) ist zum Entsperren nötig.";
    l.status_verified = "{}={} geprüft durch Bestätigung und passende Geräteauslesung.";
    l.status_disconnected = "Getrennt. Keine automatische Wiederverbindung.";
    l.status_quit_confirm = "Abbruch angefordert. Ein laufender Schreibvorgang kann abgeschlossen werden. q oder Enter drücken, um ohne Warten zu beenden; Esc zum Bleiben.";
    l.status_cancel_busy = "Abbruch angefordert; warte auf aktuellen Schritt. Ein laufender Schreibvorgang kann abgeschlossen werden. Spätere Schritte werden übersprungen.";
    l.status_address_saved = "Adresse lokal gespeichert. c drücken, um explizit zu verbinden.";
    l.status_bad_address =
        "Sechs mit Doppelpunkten getrennte Hexadezimal-Oktetten eingeben, z. B. AA:BB:CC:DD:EE:FF.";
    l.status_model_on =
        "Studio Pro HP206-Protokoll von dir gewählt, keine Geräteidentität geprüft.";
    l.status_device_picked =
        "Adresse gewählt. Bestätige das Studio Pro HP206-Protokoll zum Verbinden (y oder Enter).";
    l.status_device_bad = "Gerät im Cache hat eine ungültige Adresse; manuell bearbeiten.";
    l.status_cancel_then_disconnect =
        "Abbruch angefordert; Trennung folgt dem aktuellen Schritt. Ein laufender Schreibvorgang kann abgeschlossen werden.";
    l.status_must_disconnect = "Trenne (d), bevor Ziel oder Protokoll geändert werden.";
    l.status_must_connect = "Getrennt; verbinde (c), bevor du aktualisierst.";
    l.status_proposed = "Nur Vorschlag. Enter wendet an; Esc verwirft.";
    l.status_unavailable_setting =
        "Diese Einstellung ist in der Geräteauslesung nicht verfügbar; Schreiben deaktiviert.";
    l.status_already_matches =
        "Der Wert stimmt schon mit der letzten Auslesung überein; kein Schreibvorgang nötig.";
    l.status_pick_first = "Wähle mit Links/Rechts einen Vorschlag, bevor du anwendest.";
    l.status_disabled =
        "Änderungen deaktiviert. Verbinde und aktualisiere explizit nach jeder unsicheren Operation.";
    l.status_lang = "Sprache: {}. L drücken zum Wechseln.";
    l.err_gone = "Getrennt; zuerst verbinden.";
    l.err_status_query = "Statusabfrage fehlgeschlagen: {}";
    l.err_paired_list = "Cache-Liste der Paarungen fehlgeschlagen: {}";
    l.err_need_model = "Bestätige das Studio Pro HP206-Protokoll vor dem Verbinden.";
    l.err_connected_already = "Trenne, bevor ein anderes Ziel verbunden wird.";
    l.err_connect = "Verbindung fehlgeschlagen: {}";
    l.err_uncertain_blocked =
        "Schreiben nach unsicherer Operation gesperrt. Zuerst explizit aktualisieren.";
    l.err_preflight = "Preflight-Status fehlgeschlagen; keine Einstellung gesendet: {}";
    l.err_preflight_unreadable = "Preflight kann {} nicht lesen; nichts gesendet.";
    l.err_ack = "Einstellung evtl. gesendet; Bestätigung fehlgeschlagen: {}. Vor dem nächsten Schreibvorgang explizit aktualisieren.";
    l.err_readback = "Schreibvorgang bestätigt, aber Auslesung fehlgeschlagen: {}. Vor dem nächsten Schreibvorgang explizit aktualisieren.";
    l.err_readback_noack = "Schreibvorgang gesendet, aber Auslesung fehlgeschlagen: {}. Vor dem nächsten Schreibvorgang explizit aktualisieren.";
    l.err_mismatch = "Schreibvorgang bestätigt, aber {} stimmte nicht mit {} überein. Vor dem nächsten Schreibvorgang explizit aktualisieren.";
    l.err_mismatch_noack = "Schreibvorgang gesendet, aber {} stimmte nicht mit {} überein. Vor dem nächsten Schreibvorgang explizit aktualisieren.";
    l.err_cancel_uncertain = "Spätere Schritte abgebrochen. Ein laufender Schreibvorgang wurde evtl. abgeschlossen; vor dem nächsten Schreibvorgang explizit aktualisieren.";
    l.err_cancel = "Abgebrochen; keine weiteren Schritte starten.";
    l.cli_help = "ugreen 0.1.0 — inoffizielle Bluetooth-CLI für UGREEN Studio Pro\n\nVERWENDUNG\n  ugreen [OPTIONEN] BEFEHL\n\nOFFLINE-BEFEHLE\n  help                          Diese Hilfe zeigen\n  tui                           Optionale Terminal-Oberfläche öffnen\n  models                        Protokoll-Kompatibilität und Grenzen zeigen\n  commands                      Unterstützte Einstellungen auflisten\n  decode HEX                    Aufgezeichnete Antwort-Bytes offline prüfen\n  profile example               Beispielprofil anzeigen\n\nBLUETOOTH-BEFEHLE (zuerst in den Systemeinstellungen paaren)\n  discover                      Gepaarte Geräte aus dem Cache; kein Radioscan\n  status                        Akku, Firmware und Einstellungen lesen\n  set SCHLÜSSEL WERT            Eine Einstellung ändern und Auslesung prüfen\n  profile export                Einstellungen lesen und wiederverwendbares Profil ausgeben\n  profile apply DATEI           Profil validieren, anwenden und jede Einstellung prüfen\n\nOPTIONEN (vor dem BEFEHL)\n  --address XX:XX:XX:XX:XX:XX    Explizite Bluetooth-Zieladresse\n  --model studio-pro            Für Hardware-Anfragen nötig; nie automatisch erkannt\n  --channel 1                   RFCOMM-Kanal, 1–30 (Standard 1)\n  --timeout 3                   Timeout pro Operation in Sekunden, 1–60\n  --lang de                     Oberflächensprache: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (Standard en)\n  --autoconnect                 TUI verbindet beim Start zum gespeicherten Ziel\n  --no-autoconnect              Automatische Verbindung beim Start deaktivieren\n  --dry-run                     Einstellungspakete ohne Bluetooth-Zugriff ausgeben\n  --help, -h                    Hilfe zeigen\n  --version, -V                 Version zeigen\n\nBEISPIELE\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > mein-profil.conf\n  ugreen --dry-run profile apply mein-profil.conf\n\nStudio Pro HP206-Protokoll geprüft auf Retail-Firmware 0.2.5 (ein Gerät);\nHiTune Max5c verwendet konfliktreiche Befehls-IDs. Keine Firmware-, Reset-\n, Rohschreib- oder Suchaktionen. Profile können teilweise angewendet werden.\n";
    l.cli_models = "studio-pro: Protokoll basierend auf dem UGREEN Studio Pro HP206-Referenzcode und der Aufzeichnung\nHardware-Teststatus: VERIFIZIERT auf Retail-Firmware 0.2.5 (Linux, Kanal 1; siehe docs/verification.md)\nHiTune Max5c: NICHT UNTERSTÜTZT (konfliktreiche Befehls-IDs)\nAndere UGREEN-Modelle/Firmware: NICHT GEPRÜFT";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Studio Pro HP206-Protokoll bei {}, RFCOMM-Kanal {}";
    l.cli_need_model =
        "--model studio-pro übergeben, um dieses modellspezifische Protokoll explizit zu wählen";
    l.cli_need_address = "--address fehlt; discover listet gepaarte Geräte auf";
    l.cli_connect_fail = "Bluetooth-Verbindung fehlgeschlagen: {}. Prüfe Paarung, Strom, Adapter und ob eine andere App den Kontrollkanal besitzt";
    l.cli_preflight_fail = "Preflight-Status fehlgeschlagen; keine Einstellungen gesendet: {}";
    l.cli_verified = "Geprüft {}={}";
    l.cli_apply_fail = "{} von {} Einstellungen geprüft vor dem Fehler bei {}={}: {}. Frühere Einstellungen werden nicht zurückgerollt";
    l.cli_model_note = "model=studio-pro (vom Benutzer gewählt; keine Geräteidentitätsprüfung)";
    l.cli_unknown_value = "unbekannt";
    l.cli_firmware_fail = "Einstellungen gelesen, aber Firmware-Abfrage fehlgeschlagen: {}";
    l.cli_no_paired =
        "Keine gepaarten Geräte zurückgegeben. Paare die Kopfhörer zuerst in den Bluetooth-Einstellungen.";
    l.cli_paired_fail = "Auflistung der Paarungen fehlgeschlagen: {}";
    l.cli_dry_run_writes = "Simulationslauf: keine Bluetooth-Verbindung und keine Schreibvorgänge";
    l.cli_bad_model = "nur --model studio-pro ist implementiert; Max5c-IDs sind inkompatibel";
    l.cli_bad_option = "unbekannte Option '{}'";
    l.cli_bad_channel = "Kanal muss 1–30 sein";
    l.cli_bad_timeout = "Timeout muss 1–60 Sekunden sein";
    l.cli_need_value = "{} benötigt einen Wert";
    l.cli_unknown_command = "unbekannter Befehl '{}'; führe ugreen --help aus";
    l.cli_bad_usage = "verwendung: {}";
    l.cli_bad_decode = "die Aufzeichnung ist kein sauberer, vollständiger Antwortstrom: {} Rahmen, {} abgelehnte CRCs, {} verworfene Bytes, {} unbekannte Rahmen, {} offene Bytes";
    l.cli_no_tty = "Die TUI erfordert interaktive stdin- und stdout-Terminals; führe ugreen --help für CLI-Befehle aus.";
    l.cli_no_tui_feature =
        "Die TUI ist in dieser Nur-CLI-Kompilation deaktiviert; kompiliere ohne --no-default-features neu";
    l.cli_tui_dry =
        "tui lässt sich nicht mit --dry-run kombinieren; nutze --dry-run set SCHLÜSSEL WERT für Offline-Pakete";
    l.cli_bad_profile = "Profil überschreitet 16 KiB";
    l.cli_inflight_after_quit = "Ein laufender Schreibvorgang wurde evtl. abgeschlossen. Aktualisiere den Gerätestatus explizit, bevor du weitere Änderungen machst.";
    l.set_bad_anc = "ungültiger ANC-Wert";
    l.set_bad_eq = "ungültiger EQ-Wert";
    l.set_bad_onoff = "on oder off erwartet";
    l.set_bad_prompts = "voice oder beeps erwartet";
    l.set_bad_button = "none, next oder previous erwartet";
    l.set_unknown = "unbekannte Einstellung '{}'\n{}";
    l.profile_big = "Profil überschreitet 16 KiB";
    l.profile_line = "Profil-Zeile {} muss schlüssel=wert sein";
    l.profile_dup = "doppelter Profilschlüssel '{}'";
    l.profile_model = "Profil-Modell muss studio-pro sein";
    l.profile_need_model = "Profil erfordert model=studio-pro";
    l.profile_empty = "Profil hat keine Einstellungen";
    l.req_timeout = "keine passende gültige Antwort vor Ablauf der Frist";
    l.req_closed = "Bluetooth-Verbindung geschlossen";
    l.req_rejected = "Kopfhörer haben Anweisung 0x{:02X} abgelehnt";
    l.info_short = "Geräteinfo-Nutzlast zu kurz: {} Bytes (mindestens 8 nötig)";
    l.fw_short = "Firmware-Antwort enthält keine Versionsbytes";
    l.set_ackfail = "Einstellung evtl. gesendet, aber Bestätigung fehlgeschlagen: {}; vor erneutem Versuch den Status abfragen";
    l.set_rbfail = "Schreibvorgang bestätigt, aber Auslesung fehlgeschlagen: {}";
    l.set_rbfail_noack = "Schreibvorgang gesendet, aber Auslesung fehlgeschlagen: {}";
    l.set_mismatch = "Schreibvorgang bestätigt, aber die Auslesung von {} stimmt nicht mit {} überein; Status abfragen";
    l.set_mismatch_noack = "Schreibvorgang gesendet, aber die Auslesung von {} stimmt nicht mit {} überein; Status abfragen";
    l
}
