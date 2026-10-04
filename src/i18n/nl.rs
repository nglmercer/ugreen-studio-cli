//! Dutch catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / Studio Pro-bediening ";
    l.connected = "VERBONDEN";
    l.disconnected = "VERBROKEN";
    l.target = "Doel: ";
    l.target_none = "niet gekozen";
    l.protocol = "Protocol: ";
    l.protocol_on = "Studio Pro HP206 (door jou gekozen)";
    l.protocol_off = "niet gekozen; m bevestigt Studio Pro HP206";
    l.battery = "Batterij: ";
    l.battery_pct = "{}%";
    l.unavailable = "onbeschikbaar";
    l.codec_line = "Codec: onbeschikbaar (niet vrijgegeven)";
    l.firmware = "Firmware: ";
    l.snapshot = "Uitlezing: ";
    l.snapshot_stale = "VEROUDERD / niet actueel";
    l.snapshot_ago = "gelezen {}s geleden";
    l.snapshot_never = "niet gelezen";
    l.rfcomm_line = "RFCOMM {} | tijdlimiet per bewerking {}s | retail-fw 0.2.5 gecontroleerd";
    l.labels = [
        "ANC",
        "Equalizer",
        "Spelmodus",
        "Spatiaal geluid",
        "Dubbele verbinding",
        "Windreductie",
        "Aankondigingen",
        "Actie volume omhoog",
        "Actie volume omlaag",
    ];
    l.proposed = "  -> {} (voorgesteld)";
    l.settings_title = " Instellingen / apparaatuitlezing, geen automatische schrijfbewerkingen ";
    l.settings_blocked = " Instellingen / SCHRIJVEN GEBLOKKEERD: r om te verversen ";
    l.settings_stale = " Instellingen / laatste uitlezing verouderd ";
    l.status_title = " Status ";
    l.status_working = " Bezig / Esc breekt latere stappen af ";
    l.log_title = " Sessielogboek / Omhoog Omlaag Home Eind / l Esc sluiten ";
    l.shortcuts_title = " Sneltoetsen / Omhoog Omlaag scrollen / ? Esc sluiten ";
    l.edit_title = " Doeladres bewerken ";
    l.edit_lines = &[
        "Typ hexadecimale cijfers en dubbele punten. Backspace verwijdert; Ctrl-U leegt.",
        "Enter slaat lokaal op; Esc annuleert. Er wordt geen verbinding gestart.",
    ];
    l.model_title = " Model-specifiek protocol bevestigen ";
    l.model_lines = &[
        "Alleen te gebruiken met UGREEN Studio Pro HP206.",
        "Gecontroleerd op retail-firmware 0.2.5 (één apparaat); andere",
        "varianten en firmware zijn niet gecontroleerd.",
        "HiTune Max5c gebruikt conflicterende opdracht-IDs en wordt niet ondersteund.",
        "Dit kiest een protocol; het bewijst geen apparaatidentiteit.",
        "y / Enter: kies Studio Pro HP206    n / Esc: annuleren",
    ];
    l.paired_title = " Gekoppeld / Omhoog Omlaag Enter kiezen / Esc sluiten ";
    l.paired_empty = "Geen gekoppelde apparaten in de cache. Esc sluit.";
    l.quit_title = " Afsluiten tijdens lopend werk? ";
    l.quit_lines = &[
        "Annulering gevraagd; latere stappen worden overgeslagen.",
        "Een lopende schrijfbewerking kan nog worden voltooid.",
        "Afsluiten wacht nooit op de worker.",
        "q / Enter: nu afsluiten    Esc: blijven en het resultaat afwachten",
    ];
    l.resize_to = "Vergroot naar minstens {}x{}";
    l.resize_quit = "Esc: annuleren  q: afsluiten";
    l.resize_inflight =
        "Lopende schrijfbewerking kan worden voltooid. q/Enter: afsluiten; Esc: blijven";
    l.resize_disabled = "Acties uitgeschakeld bij deze grootte.";
    l.footer = [
        [
            ("a", "adres"),
            ("p", "apparaten"),
            ("m", "model"),
            ("c", "verbinden"),
            ("r", "verversen"),
            ("d", "verbreken"),
        ],
        [
            ("<", "minder"),
            (">", "meer"),
            ("Enter", "toepassen"),
            ("y", "bevestigen"),
            ("Esc", "terug"),
            ("", ""),
        ],
        [
            ("l", "logboek"),
            ("?", "sneltoetsen"),
            ("L", "taal"),
            ("q", "afsluiten"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "Niet verbonden. Nog geen Bluetooth-toegang. Bewerk het adres (a) of lees de gekoppelde cache (p).";
    l.status_loading = "Gekoppelde apparaten uit cache laden (geen scan)...";
    l.status_connecting = "Verbinding maken en status en firmware lezen...";
    l.status_refreshing = "Status en firmware verversen...";
    l.status_disconnecting = "Verbinding verbreken...";
    l.status_applying = "{}={} toepassen na preflight, bevestiging en uitlezing controleren...";
    l.status_paired_empty =
        "Geen gekoppelde apparaten in de cache. Koppel eerst in de Bluetooth-instellingen van het systeem.";
    l.status_paired_pick = "Kies het gewenste apparaat en druk op Enter. Kiezen verbindt niet.";
    l.status_read_ok = "Status succesvol gelezen. Getoonde instellingen zijn de apparaatuitlezing.";
    l.status_firmware_note = "Instellingen gelezen; firmware onbeschikbaar: {}";
    l.status_blocked_note =
        " Een expliciete verversing (r) is nog vereist om schrijven te ontgrendelen.";
    l.status_verified = "{}={} geverifieerd via bevestiging en bijbehorende apparaatuitlezing.";
    l.status_disconnected = "Niet verbonden. Geen automatische heroverbinding.";
    l.status_quit_confirm = "Annulering gevraagd. Een lopende schrijfbewerking kan worden voltooid. Druk q of Enter om zonder te wachten af te sluiten; Esc om te blijven.";
    l.status_cancel_busy = "Annulering gevraagd; wacht op de huidige stap. Een lopende schrijfbewerking kan worden voltooid. Latere stappen worden overgeslagen.";
    l.status_address_saved = "Adres lokaal opgeslagen. Druk c om expliciet te verbinden.";
    l.status_bad_address =
        "Voer zes hexadecimale octetten gescheiden door dubbele punten in, bijv. AA:BB:CC:DD:EE:FF.";
    l.status_model_on =
        "Studio Pro HP206-protocol door jou gekozen, geen apparaatidentiteit geverifieerd.";
    l.status_device_picked =
        "Adres gekozen. Bevestig het Studio Pro HP206-protocol om te verbinden (y of Enter).";
    l.status_device_bad = "Apparaat in de cache heeft een ongeldig adres; bewerk het handmatig.";
    l.status_cancel_then_disconnect =
        "Annulering gevraagd; verbreking volgt de huidige stap. Een lopende schrijfbewerking kan worden voltooid.";
    l.status_must_disconnect = "Verbreek (d) voordat je doel of protocol wijzigt.";
    l.status_must_connect = "Niet verbonden; verbind (c) voordat je ververst.";
    l.status_proposed = "Alleen een voorgestelde waarde. Enter past toe; Esc verwierp.";
    l.status_unavailable_setting =
        "Deze instelling ontbreekt in de apparaatuitlezing; schrijven is uitgeschakeld.";
    l.status_already_matches =
        "Deze waarde komt al overeen met de laatste uitlezing; geen schrijven nodig.";
    l.status_pick_first =
        "Gebruik Links/Rechts om een voorgestelde waarde te kiezen voordat je toepast.";
    l.status_disabled =
        "Wijzigingen uitgeschakeld. Verbind en ververs expliciet na elke onzekere bewerking.";
    l.status_lang = "Taal: {}. Druk L om door te lopen.";
    l.err_gone = "Niet verbonden; verbind eerst.";
    l.err_status_query = "Statusopdracht mislukt: {}";
    l.err_paired_list = "Cache-lijst van koppelingen mislukt: {}";
    l.err_need_model = "Bevestig het Studio Pro HP206-protocol voordat je verbindt.";
    l.err_connected_already = "Verbreken voordat je een ander doel verbindt.";
    l.err_connect = "Verbinding mislukt: {}";
    l.err_uncertain_blocked =
        "Schrijven geblokkeerd na een onzekere bewerking. Ververs eerst expliciet.";
    l.err_preflight = "Preflight-status mislukt; geen instelling verzonden: {}";
    l.err_preflight_unreadable = "Preflight leest {} niet; niets verzonden.";
    l.err_ack = "Instelling is mogelijk verzonden; bevestiging mislukt: {}. Ververs expliciet voor een volgende schrijfbewerking.";
    l.err_readback = "Schrijven bevestigd maar uitlezing mislukt: {}. Ververs expliciet voor een volgende schrijfbewerking.";
    l.err_readback_noack = "Schrijven verzonden maar uitlezing mislukt: {}. Ververs expliciet voor een volgende schrijfbewerking.";
    l.err_mismatch = "Schrijven bevestigd maar {} kwam niet overeen met {}. Ververs expliciet voor een volgende schrijfbewerking.";
    l.err_mismatch_noack = "Schrijven verzonden maar {} kwam niet overeen met {}. Ververs expliciet voor een volgende schrijfbewerking.";
    l.err_cancel_uncertain = "Latere stappen geannuleerd. Een lopende schrijfbewerking is mogelijk voltooid; ververs expliciet voor een volgende schrijfbewerking.";
    l.err_cancel = "Geannuleerd; geen volgende stappen starten.";
    l.cli_help = "ugreen 0.1.0 — niet-officiële Bluetooth-CLI voor UGREEN Studio Pro\n\nGEBRUIK\n  ugreen [OPTIES] OPDRACHT\n\nOFFLINE OPDRACHTEN\n  help                          Deze hulp tonen\n  tui                           Optionele terminal-interface openen\n  models                        Protocolcompatibiliteit en limieten tonen\n  commands                      Ondersteunde instellingen opnoemen\n  decode HEX                    Gecapte responsebytes offline valideren/decoderen\n  profile example               Een voorbeeldprofiel tonen\n\nBLUETOOTH-OPDRACHTEN (koppel eerst in de systeeminstellingen)\n  discover                      Gekoppelde apparaten uit cache; geen radioscan\n  status                        Batterij, firmware en instellingen lezen\n  set SLEUTEL WAARDE            Een instelling wijzigen en uitlezing verifiëren\n  profile export                Instellingen lezen en een herbruikbaar profiel afdrukken\n  profile apply BESTAND         Profiel valideren, toepassen en elke instelling verifiëren\n\nOPTIES (vóór de OPDRACHT)\n  --address XX:XX:XX:XX:XX:XX    Expliciet Bluetooth-doeladres\n  --model studio-pro            Vereist voor hardwareverzoeken; nooit automatisch gedetecteerd\n  --channel 1                   RFCOMM-kanaal, 1–30 (standaard 1)\n  --timeout 3                   Tijdlimiet per bewerking in seconden, 1–60\n  --lang nl                     Interface-taal: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (standaard en)\n  --autoconnect                 TUI verbindt met het onthouden doel bij start\n  --no-autoconnect              Automatische verbinding bij start uitschakelen\n  --dry-run                     Instellingspakketten afdrukken zonder Bluetooth-toegang\n  --help, -h                    Hulp tonen\n  --version, -V                 Versie tonen\n\nVOORBEELDEN\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > mijn-profiel.conf\n  ugreen --dry-run profile apply mijn-profiel.conf\n\nStudio Pro HP206-protocol gecontroleerd op retail-firmware 0.2.5 (één apparaat);\nHiTune Max5c gebruikt conflicterende opdracht-IDs. Geen firmware-, reset-, ruwe\nschrijf- of zoekacties. Profielen kunnen deels worden toegepast bij een fout.\n";
    l.cli_models = "studio-pro: protocol gebaseerd op de UGREEN Studio Pro HP206-referentiecode en -opname\nHardware-teststatus: GECONTROLEERD op retail-firmware 0.2.5 (Linux, kanaal 1; zie docs/verification.md)\nHiTune Max5c: NIET ONDERSTEUND (conflicterende opdracht-IDs)\nAndere UGREEN-modellen/firmware: NIET GECONTROLEERD";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Studio Pro HP206-protocol op {}, RFCOMM-kanaal {}";
    l.cli_need_model =
        "geef --model studio-pro om dit model-specifieke protocol expliciet te kiezen";
    l.cli_need_address =
        "--address ontbreekt; gebruik discover om gekoppelde apparaten op te noemen";
    l.cli_connect_fail = "Bluetooth-verbinding mislukt: {}. Controleer koppeling, stroom, adapter en of een andere app het controlekanaal bezit";
    l.cli_preflight_fail = "preflight-status mislukt; geen instellingen verzonden: {}";
    l.cli_verified = "Geverifieerd {}={}";
    l.cli_apply_fail = "{} van {} instellingen geverifieerd vóór de fout bij {}={}: {}. Eerdere instellingen worden niet teruggedraaid";
    l.cli_model_note =
        "model=studio-pro (door jou gekozen; geen verificatie van apparaatidentiteit)";
    l.cli_unknown_value = "onbekend";
    l.cli_firmware_fail = "instellingen gelezen, maar firmware-opdracht mislukt: {}";
    l.cli_no_paired =
        "Geen gekoppelde apparaten geretourneerd. Koppel de koptelefoon eerst in de Bluetooth-instellingen.";
    l.cli_paired_fail = "lijst van gekoppelde apparaten mislukt: {}";
    l.cli_dry_run_writes = "Droogloop: geen Bluetooth-verbinding en geen schrijfbewerkingen";
    l.cli_bad_model = "alleen --model studio-pro is geïmplementeerd; Max5c-IDs zijn incompatibel";
    l.cli_bad_option = "onbekende optie '{}'";
    l.cli_bad_channel = "kanaal moet 1–30 zijn";
    l.cli_bad_timeout = "tijdlimiet moet 1–60 seconden zijn";
    l.cli_need_value = "{} vereist een waarde";
    l.cli_unknown_command = "onbekende opdracht '{}'; voer ugreen --help uit";
    l.cli_bad_usage = "gebruik: {}";
    l.cli_bad_decode = "de opname is geen schone, volledige responsstroom: {} frames, {} verworpen CRC's, {} weggegooide bytes, {} onbekende frames, {} openstaande bytes";
    l.cli_no_tty = "De TUI vereist interactieve stdin- en stdout-terminals; voer ugreen --help uit voor CLI-opdrachten.";
    l.cli_no_tui_feature =
        "De TUI is uitgeschakeld in deze alleen-CLI-build; herbouw zonder --no-default-features";
    l.cli_tui_dry =
        "tui kan niet met --dry-run worden gecombineerd; gebruik --dry-run set SLEUTEL WAARDE voor offline pakketten";
    l.cli_bad_profile = "profiel overschrijdt 16 KiB";
    l.cli_inflight_after_quit = "Een lopende schrijfbewerking is mogelijk voltooid. Ververs expliciet de apparaatstatus voordat je verdere wijzigingen aanbrengt.";
    l.set_bad_anc = "ongeldige ANC-waarde";
    l.set_bad_eq = "ongeldige EQ-waarde";
    l.set_bad_onoff = "on of off verwacht";
    l.set_bad_prompts = "voice of beeps verwacht";
    l.set_bad_button = "none, next of previous verwacht";
    l.set_unknown = "onbekende instelling '{}'\n{}";
    l.profile_big = "profiel overschrijdt 16 KiB";
    l.profile_line = "regel {} van het profiel moet sleutel=waarde zijn";
    l.profile_dup = "dubbele profielsleutel '{}'";
    l.profile_model = "het model van het profiel moet studio-pro zijn";
    l.profile_need_model = "profiel vereist model=studio-pro";
    l.profile_empty = "profiel bevat geen instellingen";
    l.req_timeout = "geen geldige respons vóór de deadline";
    l.req_closed = "Bluetooth-verbinding gesloten";
    l.req_rejected = "koptelefoon weigerde instructie 0x{:02X}";
    l.info_short = "apparaatinfo-payload te kort: {} bytes (minimaal 8 nodig)";
    l.fw_short = "firmware-respons bevat geen versiebytes";
    l.set_ackfail = "instelling is mogelijk verzonden maar bevestiging mislukt: {}; vraag de status vóór een nieuwe poging";
    l.set_rbfail = "schrijven bevestigd maar uitlezing mislukt: {}";
    l.set_rbfail_noack = "schrijven verzonden maar uitlezing mislukt: {}";
    l.set_mismatch =
        "schrijven bevestigd maar de uitlezing van {} komt niet overeen met {}; vraag de status";
    l.set_mismatch_noack =
        "schrijven verzonden maar de uitlezing van {} komt niet overeen met {}; vraag de status";
    l
}
