//! Dutch catalog. Starts from English; untranslated strings
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
        "Optioneel socketoverzicht",
        "Niet beschikbaar; blokkeert de bediening niet",
        "Beschikbare bediening",
    ];
    l.cli_json_unsupported = "Deze opdracht ondersteunt --json niet.";
    l.status_applying_noack = "{}={} toepassen na voorcontrole en teruglezing controleren (ruimtelijke audio zonder bevestiging)...";
    l.status_verified_noack = "{}={} geverifieerd door overeenkomende teruglezing; ruimtelijke audio verzendt geen bevestiging.";
    l.app_title = " UGREEN / Studio Pro-bediening ";
    l.connected = "VERBONDEN";
    l.disconnected = "VERBROKEN";
    l.unknown = "ONBEKEND";
    l.target = "Doel: ";
    l.target_none = "niet gekozen";
    l.protocol = "Protocol: ";
    l.protocol_on = "Studio Pro HP206 (door jou gekozen)";
    l.protocol_off = "niet gekozen; m bevestigt Studio Pro HP206";
    l.battery = "Batterij: ";
    l.battery_pct = "{}%";
    l.unavailable = "onbeschikbaar";
    l.codec_line = "Codec: niet gedetecteerd";
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
    l.status_disconnecting = "Controlverbinding verbreken...";
    l.status_applying = "{}={} toepassen na preflight, bevestiging en uitlezing controleren...";
    l.status_paired_empty =
        "Geen gekoppelde apparaten in de cache. Koppel eerst in de Bluetooth-instellingen van het systeem.";
    l.status_paired_pick = "Kies het gewenste apparaat en druk op Enter. Kiezen verbindt niet.";
    l.status_read_ok = "Status succesvol gelezen. Getoonde instellingen zijn de apparaatuitlezing.";
    l.status_firmware_note = "Instellingen gelezen; firmware onbeschikbaar: {}";
    l.status_blocked_note =
        " Een expliciete verversing (r) is nog vereist om schrijven te ontgrendelen.";
    l.status_verified = "{}={} geverifieerd via bevestiging en bijbehorende apparaatuitlezing.";
    l.status_disconnected = "Controlverbinding verbroken. Bluetooth is niet gewijzigd.";
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
    l.cli_models = "studio-pro: protocol gebaseerd op de UGREEN Studio Pro HP206-referentiecode en -opname\nHardware-teststatus: GECONTROLEERD op retail-firmware 0.2.5 (Linux, kanaal 1; zie docs/verification.md)\nHiTune Max5c: NIET ONDERSTEUND (conflicterende opdracht-IDs)\nAndere UGREEN-modellen/firmware: NIET GECONTROLEERD";
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
    // Fragmenten van gegenereerde hulp;
    // het scherm zelf wordt uit het
    // opdrachtregister gegenereerd.
    l.help_subtitle = "onofficiële Bluetooth-CLI voor UGREEN Studio Pro";
    l.help_usage = "GEBRUIK";
    l.help_offline = "OFFLINE-OPDRACHTEN";
    l.help_bluetooth = "BLUETOOTH-OPDRACHTEN (koppel eerst in de systeeminstellingen)";
    l.help_capture_gated =
        "CAPTURE-BEVEILIGDE OPVRAGINGEN (worden nu geweigerd vóór het verbinden)";
    l.help_options = "OPTIES (voor de OPDRACHT)";
    l.help_examples = "VOORBEELDEN";
    l.help_footer = "Studio Pro HP206-protocol geverifieerd op retail-firmware 0.2.5 (één apparaat);\nHiTune Max5c gebruikt conflicterende opdracht-IDs. Er zijn geen firmware-, reset-\nrauwe-schrijf- of koptelefoonzoekacties. Profielen kunnen bij fouten deels worden toegepast.";
    l.opt_address = "Expliciet Bluetooth-adres van het doel";
    l.opt_model = "Vereist voor hardwareverzoeken; wordt nooit automatisch gedetecteerd";
    l.opt_channel = "RFCOMM-kanaal, 1–30 (standaard 1)";
    l.opt_timeout = "Time-out per operatie in seconden, 1–60";
    l.opt_lang = "Interface-taal: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (standaard en)";
    l.opt_autoconnect = "Verbind met het laatst gebruikte apparaat bij opstarten (standaard aan)";
    l.opt_no_autoconnect = "Automatische verbinding bij opstarten uitschakelen";
    l.opt_json = "JSON met versie voor ondersteunde leesopdrachten";
    l.opt_dry_run = "Instelingspakketten afdrukken zonder Bluetooth-toegang";
    l.opt_help = "Hulp tonen";
    l.opt_version = "Versie tonen";
    l.cmd_help = "Deze hulp tonen";
    l.cmd_tui = "De optionale interactieve terminalinterface openen";
    l.cmd_models = "Protocolcompatibiliteit en limieten tonen";
    l.cmd_commands = "Ondersteunde instellingen oplijsten";
    l.cmd_decode = "Gecaptureerde antwoordbytes offline valideren/decoderen";
    l.cmd_analyze = "Statische APK-bevindingen offline interpreteren";
    l.cmd_candidate = "Een statisch pakketkandidaat afdrukken; wordt nooit verzonden";
    l.cmd_profile = "Instelingsprofielen beheren";
    l.cmd_profile_example = "Een voorbeeldinstelingsprofiel afdrukken";
    l.cmd_profile_export = "Instellingen lezen en een herbruikbaar profiel afdrukken";
    l.cmd_profile_apply = "Profiel valideren, toepassen en elke instelling verifiëren";
    l.cmd_discover = "Gecachte gekoppelde apparaten oplijsten; geen radioscan";
    l.cmd_capture = "Ruwe TX/RX-bytes van een sessie opnemen als fixture-hex";
    l.cmd_status = "Batterij, firmware en instellingen lezen";
    l.cmd_set = "Een instelling wijzigen en vervolgens de readback verifiëren";
    l.cmd_query = "Capture-beveiligde opvraging; niet geverifieerd op deze firmware";
    l.cmd_control = "Capture-beveiligde besturing; --dry-run drukt bytes af";
    l.cmd_find = "Capture-beveiligde zoontoon";
    l.navigation = [
        "Start",
        "Audio",
        "Apparaten",
        "Instellingen",
        "Diagnostiek",
        "Onderzoek",
    ];
    l.headset_connections = ["Headsetverbindingen", "Dit zijn headsetapparaten, los van apparaten gekoppeld aan deze pc.", "De HP206-query (0x0D) vereist opnamen van de officiële app voor deze firmware. Geen query verzonden; aantal en identiteiten zijn onbekend.", "Externe apparaten loskoppelen, verbinden en wisselen wordt niet ondersteund. Hostacties betreffen alleen deze pc.", "r: controleren · d: dubbele verbinding · Esc: terug"];
    l.headset_connections[4] = "r: dubbele status vernieuwen · d: dubbele verbinding · Esc: terug";
    l.page_labels = [
        "Apparaten",
        "Audio",
        "Headsetinstellingen",
        "Multipoint",
        "Experimentele functies",
        "Diagnostiek",
    ];
    l.page_hint = "↑/↓ Enter: openen · r: vernieuwen · Esc: terug";
    l.host_commands = [
        "Systeemaudiotransport bekijken",
        "Audio-uitgangen van de host tonen",
        "Functiebewijs tonen",
        "Hostdiagnose lezen",
        "Bluetooth-verbinding van host openen",
        "Bluetooth-verbinding van host verbreken",
    ];
    l.cmd_audio_devices = "Windows-uitvoerapparaten oplijsten";
    l.features_title = " Functies ";
    l.features_search = "Zoeken: ";
    l.features_all = "Alles";
    l.feature_categories = [
        "Audio",
        "Verbinding",
        "Omgeving",
        "Terugkoppeling",
        "Apparaatinformatie",
        "Multipunt",
        "Codecs",
        "Geavanceerd audio",
        "Besturing",
        "Audio delen",
        "Zoeken",
        "Apparaatbeheer",
        "Diagnose",
    ];
    l.feature_names = [
        "ANC",
        "Equalizer",
        "Spelmodus",
        "Ruimtelijk geluid",
        "Dubbele verbinding",
        "Windreductie",
        "Spraakberichten",
        "Actie volume omhoog",
        "Actie volume omlaag",
        "Multipunt-apparaten",
        "Audio-codec",
        "Koptelefoon-ID",
        "Koptelefoonkleur",
        "Case-MAC",
        "LDAC-besturing",
        "LDAC voor een apparaat",
        "Hoogwaardige decodering",
        "Volumebevel",
        "Mono/stereo-kanaal",
        "Toetsenindeling",
        "AI-acties",
        "AI-besturing",
        "Audio delen uit",
        "Delen met een apparaat",
        "Delen met een gekozen apparaat",
        "Linker oortje vinden",
        "Rechter oortje vinden",
        "Fabrieksinstelling",
        "Capture-decoder",
        "Capture-analyse",
        "Pakketvoorbeeld",
        "Apparaat verbreken en dubbele modus uit",
    ];
    l.feature_descriptions = [
        "Diepte van actieve geluidsonderdrukking. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Equalizer-voorinstelling. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Lage latentie-schakelaar van spelmodus. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Schakelaar voor ruimtelijk geluid. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Schakelaar voor verbinding met twee apparaten. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Schakelaar voor windgeluidreductie. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Stijl van de spraakberichten. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Actie bij ingedrukt houden volume omhoog. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "Actie bij ingedrukt houden volume omlaag. Geverifieerde instelling; bewerk in het Instellingen-scherm.",
        "De lijst van verbonden multipunt-apparaten lezen. Het peer-protocol vereist nog hardwareverificatie.",
        "De actieve Bluetooth-audio-codec lezen. Het queryformaat vereist nog hardwareverificatie.",
        "De hardware-ID van de koptelefoon lezen. Het queryformaat vereist nog hardwareverificatie.",
        "De kleur van de koptelefoon lezen. Het queryformaat vereist nog hardwareverificatie.",
        "Het MAC-adres van de oplaadhoes lezen. Het queryformaat vereist nog hardwareverificatie.",
        "LDAC-codering op de huidige verbinding omschakelen. Het besturingsformaat vereist nog hardwareverificatie.",
        "LDAC-codering voor een gekoppeld apparaat omschakelen. Het besturingsformaat vereist nog hardwareverificatie.",
        "Hoogwaardige audio-decodering omschakelen. Het besturingsformaat vereist nog hardwareverificatie.",
        "Een absoluut volume-byte verzenden. Het hardwarebereik is niet geverifieerd.",
        "Het audio-kanaal wisselen tussen mono en stereo. Het besturingsformaat vereist nog hardwareverificatie.",
        "De acht langdrukacties opnieuw toewijzen. Het besturingsformaat vereist nog hardwareverificatie.",
        "De vier door AI geactiveerde acties opnieuw toewijzen. Het besturingsformaat vereist nog hardwareverificatie.",
        "AI-audiobesturing omschakelen. Het besturingsformaat vereist nog hardwareverificatie.",
        "Audio delen uitschakelen. Het besturingsformaat vereist nog hardwareverificatie.",
        "Audio delen met het enige verbonden apparaat. Het besturingsformaat vereist nog hardwareverificatie.",
        "Audio delen met een gekozen apparaat. Het besturingsformaat vereist nog hardwareverificatie.",
        "Een locatingtoon op het linker oortje afspelen via de host. Het finder-protocol vereist nog hardwareverificatie.",
        "Een locatingtoon op het rechter oortje afspelen via de host. Het finder-protocol vereist nog hardwareverificatie.",
        "De koppeling verwijderen en fabrieksinstellingen herstellen. Destructief; het besturingsformaat vereist nog hardwareverificatie.",
        "Opgevangen antwoord-bytes offline decoderen; geen Bluetooth-toegang.",
        "Een complete capture-stream offline interpreteren; geen Bluetooth-toegang.",
        "Het statische pakket afdrukken dat een besturing zou verzenden; het wordt nooit verzonden.",
        "Verbreekt het gekozen apparaat en schakelt dubbele modus uit. Experimenteel; bevestig positie en MAC met een nieuwe opvraag.",
    ];
    l.feature_available = "Beschikbaar";
    l.feature_pending = "Verificatie in behandeling";
    l.feature_pending_reason =
        "Het bedraad formaat vereist nog hardwareverificatie met de officiële app.";
    l.feature_live_disabled =
        "Live-uitvoering is uitgeschakeld tot er geverifieerde apparaatbewijzen zijn.";
    l.feature_offline_workflow =
        "Alleen offline analyse; er wordt niets naar de koptelefoon verzonden.";
    l.feature_offline_hint = "Uitvoeren: ugreen {}";
    l.feature_kind_setting = "Geverifieerde instelling";
    l.feature_kind_query = "Apparaatquery";
    l.feature_kind_control = "Apparaatbesturing";
    l.feature_kind_offline = "Offline-analyse";
    l.feature_category_label = "Categorie: {}";
    l.feature_current = "Huidig: {}";
    l.feature_edit_in_settings = "Bewerken in Instellingen (s)";
    l.feature_run_query = "Enter: query uitvoeren";
    l.feature_configure = "Enter: configureren";
    l.feature_back = "Esc: terug";
    l.feature_form_title = " {} configureren ";
    l.feature_err_required = "Vul elk veld in voordat u controleert";
    l.feature_field_unset = "niet ingesteld";
    l.feature_form_hint = "Omhoog Omlaag veld / Links Rechts keuze / Enter controleren / Esc terug";
    l.feature_review_title = " {} controleren ";
    l.review_write_title = " Schrijfactie controleren ";
    l.feature_review_target = "Bestemming: {}";
    l.feature_review_previous = "Huidig: {}";
    l.feature_review_proposed = "Voorgesteld: {}";
    l.feature_review_values = "Waarden: {}";
    l.feature_review_readback = "Teruglezing: {}";
    l.feature_readback_verified = "geverifieerd door bevestiging en passende teruglezing";
    l.feature_readback_noack =
        "geverifieerd door passende teruglezing (ruimtelijk geluid stuurt geen bevestiging)";
    l.feature_readback_unconfirmed = "niet onafhankelijk bevestigd; ververs na toepassing";
    l.feature_readback_query = "query zonder schrijven";
    l.feature_confirm_apply = "Enter: toepassen    Esc: annuleren";
    l.feature_confirm_reset =
        "Destructief: verwijdert de host-koppeling en herstelt fabrieksinstellingen.";
    l.features_footer = [
        [
            ("Omhoog", "verplaatsen"),
            ("Omlaag", "verplaatsen"),
            ("Tab", "categorie"),
            ("Enter", "openen"),
            ("Esc", "terug"),
            ("", ""),
        ],
        [
            ("letters", "zoeken"),
            ("Backspace", "wissen"),
            ("Ctrl+U", "legen"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "sluiten"),
            ("?", "sneltoetsen"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "Functiebewerking wordt uitgevoerd...";
    l.status_edit_in_settings =
        "{} is een geverifieerde instelling; bewerk in het Instellingen-scherm (F4).";
    l.codec_unverified = "query niet geverifieerd";
    l.arg_enabled = "Ingeschakeld";
    l.arg_side = "Kant";
    l.arg_duration = "Duur (s)";
    l.arg_target = "Bestemming";
    l.arg_value = "Waarde (0-255)";
    l.arg_mode = "Modus";
    l.arg_peer = "Gekoppeld";
    l.arg_actions = "Acties";
    l.arg_address = "MAC van het gekoppelde apparaat";
    l.arg_hint_toggle = "on of off";
    l.arg_hint_side = "links of rechts";
    l.arg_hint_seconds = "1..30 seconden";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "zes hexadecimale octetten met dubbele punten";
    l.arg_hint_peer = "1 of 2";
    l.arg_hint_actions = "{} kommagescheiden actiewaarden";
    l.tui_no_tty =
        "De interface vereist interactieve stdin- en stdout-terminals; voer ugreen --help uit voor de CLI-opdrachten.";
    l.tui_bad_channel =
        "Het kanaal van de interface moet 1-30 zijn en de time-out groter dan nul en ten hoogste 60 seconden.";
    l.status_worker_busy = "Er is al een bewerking bezig; wacht of druk Esc om te annuleren.";
    l.status_worker_sequence = "Aanvraagreeks uitgeput; herstart de interface.";
    l.status_worker_stopped = "Bluetooth-worker gestopt; herstart de interface.";
    l.status_worker_queue = "Worker-wachtrij vol; wacht voordat je het opnieuw probeert.";
    l.status_reset_unavailable = "terugzetdoel niet beschikbaar";
    l.status_reset_done =
        "Terugzetten bevestigd; host-koppeling en opgeslagen apparaat verwijderd.";
    l.status_inflight_write =
        "Een lopende schrijfbewerking is mogelijk voltooid; ververs na herstart.";
    l.page_labels[4] = l.navigation[5];
    l
}
