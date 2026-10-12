use std::{
    env, fs,
    io::{IsTerminal, Read},
    process::ExitCode,
    time::Duration,
};
use ugreen_cli::{
    bluetooth::{self, BluetoothAddress},
    client::Client,
    i18n::{Lang, L},
    protocol,
    settings::{self, Setting},
};

#[derive(Debug)]
struct Options {
    address: Option<String>,
    model: Option<String>,
    channel: Option<u8>,
    timeout: Option<Duration>,
    dry_run: bool,
    lang: Option<Lang>,
    autoconnect: Option<bool>,
    command: Vec<String>,
}
fn lang_of(o: &Options) -> Lang {
    o.lang.unwrap_or_else(Lang::detect)
}
fn txt_of(o: &Options) -> &'static L {
    ugreen_cli::i18n::txt(lang_of(o))
}
fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = template.to_owned();
    for arg in args {
        if let Some(pos) = out.find("{}") {
            out.replace_range(pos..pos + 2, &arg.to_string());
        } else {
            break;
        }
    }
    out
}
fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut o = Options {
        address: None,
        model: None,
        channel: None,
        timeout: None,
        dry_run: false,
        lang: None,
        autoconnect: None,
        command: Vec::new(),
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--address" => {
                o.address = Some(args.next().ok_or(
                    ugreen_cli::i18n::txt(lang_of(&o)).cli_need_value.replacen(
                        "{}",
                        "--address",
                        1,
                    ),
                )?)
            }
            "--model" => {
                o.model = Some(
                    args.next().ok_or(
                        ugreen_cli::i18n::txt(lang_of(&o))
                            .cli_need_value
                            .replacen("{}", "--model", 1),
                    )?,
                )
            }
            "--channel" => {
                let raw = args.next().ok_or(
                    ugreen_cli::i18n::txt(lang_of(&o)).cli_need_value.replacen(
                        "{}",
                        "--channel",
                        1,
                    ),
                )?;
                o.channel = Some(raw.parse().map_err(|_| {
                    ugreen_cli::i18n::txt(lang_of(&o))
                        .cli_bad_channel
                        .to_string()
                })?);
                if !(1..=30).contains(&o.channel.unwrap()) {
                    return Err(ugreen_cli::i18n::txt(lang_of(&o)).cli_bad_channel.into());
                }
            }
            "--timeout" => {
                let raw = args.next().ok_or(
                    ugreen_cli::i18n::txt(lang_of(&o)).cli_need_value.replacen(
                        "{}",
                        "--timeout",
                        1,
                    ),
                )?;
                let n: u64 = raw.parse().map_err(|_| {
                    ugreen_cli::i18n::txt(lang_of(&o))
                        .cli_bad_timeout
                        .to_string()
                })?;
                if !(1..=60).contains(&n) {
                    return Err(ugreen_cli::i18n::txt(lang_of(&o)).cli_bad_timeout.into());
                }
                o.timeout = Some(Duration::from_secs(n));
            }
            "--lang" => {
                let raw = args.next().ok_or(
                    ugreen_cli::i18n::txt(lang_of(&o))
                        .cli_need_value
                        .replacen("{}", "--lang", 1),
                )?;
                o.lang = Some(Lang::from_code(&raw)?);
            }
            "--autoconnect" => o.autoconnect = Some(true),
            "--no-autoconnect" => o.autoconnect = Some(false),
            "--dry-run" => o.dry_run = true,
            "--help" | "-h" => {
                o.command = vec!["help".into()];
                return Ok(o);
            }
            "--version" | "-V" => {
                o.command = vec!["version".into()];
                return Ok(o);
            }
            _ if arg.starts_with('-') => {
                return Err(ugreen_cli::i18n::txt(lang_of(&o))
                    .cli_bad_option
                    .replacen("{}", &arg, 1))
            }
            _ => {
                o.command.push(arg);
                o.command.extend(args);
                break;
            }
        }
    }
    if o.command.is_empty() {
        let interactive = cfg!(feature = "tui")
            && std::io::stdin().is_terminal()
            && std::io::stdout().is_terminal();
        o.command
            .push(if interactive { "tui" } else { "help" }.into());
    }
    if o.model.as_deref().is_some_and(|m| m != "studio-pro") {
        return Err(txt_of(&o).cli_bad_model.into());
    }
    Ok(o)
}
/// Every live command connects through here, already wrapped in a
/// recording tee: most callers ignore the log, `capture` exports it.
fn connect(o: &Options) -> Result<Client<protocol::Monitored<bluetooth::Connection>>, String> {
    let t = txt_of(o);
    if o.model.as_deref() != Some("studio-pro") {
        return Err(t.cli_need_model.into());
    }
    let address = o.address.as_deref().ok_or(t.cli_need_address)?;
    eprintln!(
        "{}",
        fill(
            t.cli_banner,
            &[&address, &o.channel.unwrap_or(1).to_string()]
        )
    );
    let timeout = o.timeout.unwrap_or(Duration::from_secs(3));
    BluetoothAddress::parse(address)
        .and_then(|address| {
            bluetooth::Connection::connect(&address, o.channel.unwrap_or(1), timeout)
        })
        .map(|io| {
            Client::new(
                protocol::Monitored::new(io, protocol::shared_monitor()),
                timeout,
            )
        })
        .map_err(|e| fill(t.cli_connect_fail, &[&e.to_string()]))
}
fn exact(args: &[String], count: usize, usage: &str) -> Result<(), String> {
    if args.len() == count {
        Ok(())
    } else {
        Err(format!("usage: {usage}"))
    }
}
fn read_profile(path: &str) -> Result<String, String> {
    let f = fs::File::open(path).map_err(|e| format!("cannot open profile: {e}"))?;
    let mut text = String::new();
    f.take(16_385)
        .read_to_string(&mut text)
        .map_err(|e| format!("cannot read UTF-8 profile: {e}"))?;
    if text.len() > 16_384 {
        return Err("profile exceeds 16 KiB".into());
    }
    Ok(text)
}
fn apply(o: &Options, settings_list: &[Setting]) -> Result<(), String> {
    let t = txt_of(o);
    if o.dry_run {
        for s in settings_list {
            println!("{}={}  {}", s.key, s.value, protocol::hex(&s.packet()));
        }
        println!("{}", t.cli_dry_run_writes);
        return Ok(());
    }
    let mut c = connect(o)?;
    // Initial query must be understood before the first setting write.
    c.info()
        .map_err(|e| fill(t.cli_preflight_fail, &[&e.to_string()]))?;
    for (i, s) in settings_list.iter().enumerate() {
        c.set(s).map_err(|e| {
            fill(
                t.cli_apply_fail,
                &[&i, &settings_list.len(), &s.key, &s.value, &e.to_string()],
            )
        })?;
        println!("{}", fill(t.cli_verified, &[&s.key, &s.value]));
    }
    Ok(())
}
fn run(o: Options) -> Result<(), String> {
    let t = txt_of(&o);
    let args = &o.command;
    match args[0].as_str() {
        "tui" => {
            exact(args, 1, "ugreen [OPTIONS] tui")?;
            if o.dry_run {
                return Err(t.cli_tui_dry.into());
            }
            #[cfg(feature = "tui")]
            {
                // The remembered target, protocol choice, channel,
                // timeout and language fill defaults; explicit
                // flags always win.
                let registry = ugreen_cli::device::registry::DeviceRegistry::load();
                let autoconnect = o.autoconnect.unwrap_or(registry.autoconnect);
                if let Some(autoconnect) = o.autoconnect {
                    let mut registry = registry.clone();
                    registry.autoconnect = autoconnect;
                    registry.save();
                }
                let address = o.address.or_else(|| {
                    registry
                        .selected
                        .as_ref()
                        .map(|address| address.to_string())
                });
                // The model is confirmed per address, never guessed.
                let model_confirmed = o.model.as_deref() == Some("studio-pro")
                    || address
                        .as_deref()
                        .and_then(|raw| BluetoothAddress::parse(raw).ok())
                        .is_some_and(|address| {
                            registry.model_of(&address)
                                == Some(ugreen_cli::models::Model::StudioPro)
                        });
                ugreen_cli::tui::run(ugreen_cli::tui::Config {
                    address,
                    model_confirmed,
                    channel: o.channel.or(registry.channel).unwrap_or(1),
                    timeout: o
                        .timeout
                        .or(registry.timeout.map(Duration::from_secs))
                        .unwrap_or(Duration::from_secs(3)),
                    lang: o.lang.or(registry.lang).unwrap_or_else(Lang::detect),
                    skip_autoload: false,
                    autoconnect,
                })
                .map_err(|e| e.to_string())?;
            }
            #[cfg(not(feature = "tui"))]
            return Err(t.cli_no_tui_feature.into());
        }
        "help" => {
            exact(args, 1, "ugreen help")?;
            print!("{}", t.cli_help);
        }
        "version" => {
            exact(args, 1, "ugreen --version")?;
            println!("ugreen {}", env!("CARGO_PKG_VERSION"));
        }
        "models" => {
            exact(args, 1, "ugreen models")?;
            println!("{}", t.cli_models);
        }
        "commands" => {
            exact(args, 1, "ugreen commands")?;
            println!("{}", t.cli_commands);
        }
        "peers" | "codec" | "id" | "color" | "box-mac" => {
            exact(args, 1, "ugreen [OPTIONS] peers|codec|id|color|box-mac")?;
            let (bit, feature) = match args[0].as_str() {
                "peers" => (
                    ugreen_cli::models::Capabilities::MULTIPOINT_PEERS,
                    "multipoint peer list",
                ),
                "codec" => (
                    ugreen_cli::models::Capabilities::CODEC_STATUS,
                    "codec status",
                ),
                "id" => (
                    ugreen_cli::models::Capabilities::HEADPHONE_ID,
                    "headphone ID",
                ),
                "color" => (
                    ugreen_cli::models::Capabilities::HEADPHONE_COLOR,
                    "headphone color",
                ),
                _ => (ugreen_cli::models::Capabilities::BOX_MAC, "box MAC"),
            };
            if o.dry_run {
                return Err("use candidate for offline static query packets".into());
            }
            if !ugreen_cli::models::Model::StudioPro
                .capabilities()
                .contains(bit)
            {
                return Err(format!(
                    "{feature} is not verified for Studio Pro; official-app capture required"
                ));
            }
            let mut client = connect(&o)?;
            match args[0].as_str() {
                "peers" => {
                    let peers = client.peers().map_err(|e| e.to_string())?;
                    for peer in peers.slots.into_iter().flatten() {
                        println!(
                            "peer={} address={} name={}",
                            peer.index,
                            peer.address,
                            terminal_text(&peer.name)
                        );
                    }
                }
                "codec" => println!("codecs={:?}", client.codecs().map_err(|e| e.to_string())?),
                "id" => println!("id={}", client.headphone_id().map_err(|e| e.to_string())?),
                "color" => println!(
                    "color={}",
                    client.headphone_color().map_err(|e| e.to_string())?
                ),
                _ => println!("box_mac={}", client.box_mac().map_err(|e| e.to_string())?),
            }
        }
        "audio-devices" => {
            exact(args, 1, "ugreen audio-devices")?;
            if o.dry_run {
                return Err(
                    "audio-devices reads host output devices; --dry-run is unsupported".into(),
                );
            }
            for (id, name) in
                ugreen_cli::finder::windows_output_devices().map_err(|e| e.to_string())?
            {
                println!("{id} {}", terminal_text(&name));
            }
        }
        "find" => {
            exact(
                args,
                4,
                "ugreen [OPTIONS] find left|right SECONDS AUDIO_TARGET",
            )?;
            let side = match args[1].as_str() {
                "left" => protocol::static_analysis::Side::Left,
                "right" => protocol::static_analysis::Side::Right,
                _ => return Err("finder side must be left or right".into()),
            };
            let seconds: u8 = args[2]
                .parse()
                .map_err(|_| "finder duration must be 1..30 seconds")?;
            if !(1..=30).contains(&seconds) {
                return Err("finder duration must be 1..30 seconds".into());
            }
            if o.dry_run {
                return Err("use candidate find left|right on|off for offline packets".into());
            }
            ugreen_cli::finder::validate(seconds, &args[3]).map_err(|e| e.to_string())?;
            protocol::static_analysis::Command::Find {
                enabled: true,
                side,
            }
            .check_control()
            .map_err(|e| e.to_string())?;
            let interrupts = ugreen_cli::interrupt::Guard::install().map_err(|e| e.to_string())?;
            let mut client = connect(&o)?;
            client
                .info()
                .map_err(|e| format!("finder preflight failed: {e}"))?;
            ugreen_cli::finder::ring_cancellable(&mut client, side, seconds, &args[3], || {
                interrupts.interrupted()
            })
            .map_err(|e| e.to_string())?;
            println!("host finder playback completed; stop sent; device state unconfirmed");
        }
        "control" => {
            let command = protocol::static_analysis::Command::parse(&args[1..])?;
            if command.control_capability().is_none() {
                return Err("query is not a control; use its query command".into());
            }
            if o.dry_run {
                println!("# static APK candidate; unverified; no Bluetooth access");
                println!(
                    "{}",
                    protocol::hex(&command.packet().map_err(|e| e.to_string())?)
                );
            } else {
                command.check_control().map_err(|e| e.to_string())?;
                let mut client = connect(&o)?;
                client
                    .info()
                    .map_err(|e| format!("control preflight failed: {e}"))?;
                let outcome = client.control(&command).map_err(|e| e.to_string())?;
                if command == protocol::static_analysis::Command::FactoryReset {
                    drop(client);
                    let address = BluetoothAddress::parse(
                        o.address.as_deref().expect("connect validated address"),
                    )
                    .map_err(|e| e.to_string())?;
                    ugreen_cli::reset::cleanup(&address).map_err(|e| e.to_string())?;
                    println!("reset acknowledged; host pairing and registry entry removed");
                    return Ok(());
                }
                match outcome {
                    ugreen_cli::client::ControlOutcome::Readback(_) => {
                        println!("control verified by device-info readback")
                    }
                    ugreen_cli::client::ControlOutcome::Acknowledged(_) => println!(
                        "control acknowledged; resulting device state not independently confirmed"
                    ),
                    ugreen_cli::client::ControlOutcome::SentUnconfirmed => {
                        println!("control sent; finder state and audio playback unconfirmed")
                    }
                }
            }
        }
        "candidate" => {
            let command = protocol::static_analysis::Command::parse(&args[1..])?;
            println!("# static APK candidate; unverified; no Bluetooth access");
            println!(
                "{}",
                protocol::hex(&command.packet().map_err(|e| e.to_string())?)
            );
        }
        "analyze" => {
            exact(args, 2, "ugreen analyze 'DD EE FF ...'")?;
            let bytes = protocol::parse_hex(&args[1])?;
            let mut decoder = protocol::Decoder::default();
            let frames = decoder.feed(&bytes);
            if frames.is_empty()
                || decoder.pending_bytes() != 0
                || decoder.stats.response_crc_failures != 0
                || decoder.stats.discarded_bytes != 0
                || decoder.stats.unknown_frames != 0
            {
                return Err("analysis requires complete CRC-valid incoming frames".into());
            }
            println!("static APK interpretations; unverified on hardware");
            let mut peers = protocol::static_analysis::PeerList::default();
            for frame in frames {
                match frame {
                    protocol::IncomingFrame::Response(frame)
                        if frame.instruction == protocol::INFO =>
                    {
                        if !frame.succeeded {
                            return Err("device rejected request".into());
                        }
                        println!(
                            "{:?}",
                            protocol::static_analysis::DeviceInfo::parse(&frame.payload)?
                        );
                    }
                    protocol::IncomingFrame::Response(frame) => {
                        let response = protocol::static_analysis::interpret(&frame)?;
                        if let protocol::static_analysis::Response::Peers(update) = &response {
                            peers.apply(update.clone())?;
                        }
                        println!("{response:?}");
                    }
                    protocol::IncomingFrame::Notification(frame) => {
                        println!(
                            "{:?}",
                            protocol::static_analysis::interpret_notification(&frame)
                        );
                    }
                    protocol::IncomingFrame::Unknown(_) => unreachable!(),
                }
            }
            if peers.observed {
                println!(
                    "static_peer_list complete={} slots={:?}",
                    peers.complete, peers.slots
                );
            }
        }
        "decode" => {
            exact(args, 2, "ugreen decode 'DD EE FF ...'")?;
            let bytes = protocol::parse_hex(&args[1])?;
            let mut decoder = protocol::Decoder::default();
            let frames = decoder.feed(&bytes);
            for frame in &frames {
                match frame {
                    protocol::IncomingFrame::Response(frame) => println!(
                        "instruction=0x{:02X} success={} crc=valid payload={}",
                        frame.instruction,
                        frame.succeeded,
                        protocol::hex(&frame.payload)
                    ),
                    protocol::IncomingFrame::Notification(frame) => println!(
                        "notification kind=0x{:02X} payload={} event=unknown",
                        frame.kind,
                        protocol::hex(&frame.payload)
                    ),
                    protocol::IncomingFrame::Unknown(frame) => println!(
                        "unknown instruction=0x{:02X} payload={} rx-meaning=unverified",
                        frame.instruction,
                        protocol::hex(&frame.payload)
                    ),
                }
            }
            let stats = decoder.stats;
            if frames.is_empty()
                || decoder.pending_bytes() != 0
                || stats.response_crc_failures != 0
                || stats.discarded_bytes != 0
                || stats.unknown_frames != 0
            {
                return Err(fill(
                    t.cli_bad_decode,
                    &[
                        &frames.len(),
                        &stats.response_crc_failures,
                        &stats.discarded_bytes,
                        &stats.unknown_frames,
                        &decoder.pending_bytes(),
                    ],
                ));
            }
        }
        "discover" => {
            exact(args, 1, "ugreen discover")?;
            if o.dry_run {
                return Err(t.cli_tui_dry.into());
            }
            let mut devices = bluetooth::list_devices()
                .map_err(|e| fill(t.cli_paired_fail, &[&e.to_string()]))?;
            if devices.is_empty() {
                println!("{}", t.cli_no_paired);
            }
            ugreen_cli::device::DeviceRegistry::load().annotate(&mut devices);
            for d in devices {
                match d.model {
                    Some(model) => println!(
                        "{}  {}  model={}",
                        d.address,
                        terminal_text(&d.name),
                        model.as_str()
                    ),
                    None => println!("{}  {}", d.address, terminal_text(&d.name)),
                }
            }
        }
        "status" => {
            exact(args, 1, "ugreen [OPTIONS] status")?;
            if o.dry_run {
                return Err(
                    "status cannot be combined with --dry-run; use decode for offline data".into(),
                );
            }
            let mut c = connect(&o)?;
            let info = c.info().map_err(|e| format!("status failed: {e}"))?;
            println!("{}", t.cli_model_note);
            println!(
                "battery={}",
                info.battery()
                    .map(|v| format!("{v}%"))
                    .unwrap_or(t.cli_unknown_value.into())
            );
            for key in settings::KEYS {
                println!(
                    "{key}={}",
                    info.value(key).unwrap_or(t.cli_unknown_value.into())
                );
            }
            let fw = c
                .firmware()
                .map_err(|e| fill(t.cli_firmware_fail, &[&e.to_string()]))?;
            println!("firmware={fw}");
            println!("raw_device_info={}", protocol::hex(&info.raw));
        }
        "set" => {
            exact(args, 3, "ugreen [OPTIONS] set KEY VALUE")?;
            apply(&o, &[Setting::parse_in(lang_of(&o), &args[1], &args[2])?])?;
        }
        "capture" => {
            exact(args, 1, "ugreen [OPTIONS] capture")?;
            if o.dry_run {
                return Err(
                    "capture cannot be combined with --dry-run; it records a live session".into(),
                );
            }
            let mut c = connect(&o)?;
            // Record the whole session even when a query fails: a
            // failed capture is exactly when the bytes matter most.
            // stdout stays pure fixture hex; context goes to stderr.
            let info = c.info();
            let firmware = c.firmware();
            let (_transport, log) = c.into_inner().into_inner_and_log();
            let export = log
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .export();
            print!("{export}");
            let info = info.map_err(|e| fill(t.cli_preflight_fail, &[&e.to_string()]))?;
            eprintln!(
                "battery={}",
                info.battery()
                    .map(|v| format!("{v}%"))
                    .unwrap_or(t.cli_unknown_value.into())
            );
            match firmware {
                Ok(firmware) => eprintln!("firmware={firmware}"),
                Err(e) => eprintln!("{}", fill(t.cli_firmware_fail, &[&e.to_string()])),
            }
        }
        "profile" => {
            match args.get(1).map(String::as_str) {
                Some("example") => {
                    exact(args, 2, "ugreen profile example")?;
                    print!("# UGREEN Studio Pro profile v1\nmodel=studio-pro\nanc=ultra\neq=classic\ngame=off\nspatial=off\n");
                }
                Some("export") => {
                    exact(args, 2, "ugreen [OPTIONS] profile export")?;
                    if o.dry_run {
                        return Err("profile export requires reading the device; use profile example offline".into());
                    }
                    let mut c = connect(&o)?;
                    print!("{}", c.info().map_err(|e| e.to_string())?.profile());
                }
                Some("apply") => {
                    exact(args, 3, "ugreen [OPTIONS] profile apply FILE")?;
                    let settings_list =
                        settings::parse_profile_in(lang_of(&o), &read_profile(&args[2])?)?;
                    apply(&o, &settings_list)?;
                }
                _ => {
                    return Err(fill(
                        t.cli_bad_usage,
                        &[&"ugreen profile example|export|apply FILE"],
                    ))
                }
            }
        }
        _ => return Err(fill(t.cli_unknown_command, &[&args[0]])),
    }
    Ok(())
}
fn terminal_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| {
            if ch.is_control() {
                ch.escape_default().collect::<Vec<_>>()
            } else {
                vec![ch]
            }
        })
        .collect()
}
fn main() -> ExitCode {
    match parse(env::args().skip(1).collect()).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }
    #[test]
    fn strict_options() {
        for input in [
            "--channel 0 status",
            "--channel 31 status",
            "--timeout 0 status",
            "--model max5c status",
            "--wat status",
            "--address",
            "--lang xx status",
            "--lang status",
        ] {
            assert!(parse(args(input)).is_err(), "{input}");
        }
    }
    #[test]
    fn hardware_requires_explicit_model_and_address() {
        let o = parse(args("status")).unwrap();
        assert!(connect(&o).is_err());
        let o = parse(args("--model studio-pro status")).unwrap();
        assert!(connect(&o).is_err());
    }
    #[test]
    fn capture_refuses_dry_run_and_missing_target() {
        let o = parse(args("--dry-run capture")).unwrap();
        assert!(run(o).is_err());
        let o = parse(args("capture")).unwrap();
        assert!(run(o).is_err());
        let o = parse(args("--model studio-pro capture")).unwrap();
        assert!(connect(&o).is_err());
    }
    #[test]
    fn autoconnect_flags_parse() {
        assert_eq!(
            parse(args("--autoconnect tui")).unwrap().autoconnect,
            Some(true)
        );
        assert_eq!(
            parse(args("--no-autoconnect tui")).unwrap().autoconnect,
            Some(false)
        );
        assert_eq!(parse(args("tui")).unwrap().autoconnect, None);
    }
    #[test]
    fn device_names_cannot_control_terminal() {
        assert_eq!(terminal_text("Studio\x1b[31m\n"), "Studio\\u{1b}[31m\\n");
        assert_eq!(terminal_text("Auriculares José"), "Auriculares José");
    }
    #[test]
    fn dry_run_needs_no_device() {
        assert!(run(parse(args("--dry-run set anc ultra")).unwrap()).is_ok());
    }
    #[test]
    fn lang_flag_and_locale_detection() {
        let o = parse(args("--lang es status")).unwrap();
        assert_eq!(lang_of(&o), Lang::Es);
        assert!(txt_of(&o).cli_help.contains("USO"));
        let o = parse(args("--lang en status")).unwrap();
        assert_eq!(lang_of(&o), Lang::En);
        assert!(txt_of(&o).cli_help.contains("USAGE"));
        assert_eq!(Lang::from_code("en").unwrap(), Lang::En);
    }
}
