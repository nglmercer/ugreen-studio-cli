use std::{
    env, fs,
    io::{IsTerminal, Read},
    process::ExitCode,
    time::Duration,
};
use ugreen_cli::{
    client::Client,
    protocol,
    settings::{self, Setting},
    transport,
};

const HELP:&str = "ugreen 0.1.0 — unofficial UGREEN Studio Pro Bluetooth CLI\n\nUSAGE\n  ugreen [OPTIONS] COMMAND\n\nOFFLINE COMMANDS\n  help                          Show this help\n  tui                           Open the optional interactive terminal UI\n  models                        Show protocol compatibility and limits\n  commands                      List supported settings\n  decode HEX                    Validate/decode captured response bytes offline\n  profile example               Print an example settings profile\n\nBLUETOOTH COMMANDS (pair in OS settings first)\n  discover                      List cached paired devices; no radio scan\n  status                        Read battery, firmware and settings\n  set KEY VALUE                 Change one setting, then verify readback\n  profile export                Read settings and print a reusable profile\n  profile apply FILE            Validate profile, apply and verify each setting\n\nOPTIONS (before COMMAND)\n  --address XX:XX:XX:XX:XX:XX    Explicit target Bluetooth address\n  --model studio-pro            Required for hardware requests; never auto-detected\n  --channel 1                   RFCOMM channel, 1–30 (default 1)\n  --timeout 3                   Per-operation timeout in seconds, 1–60\n  --dry-run                     Print setting packets without Bluetooth access\n  --help, -h                    Show help\n  --version, -V                 Show version\n\nEXAMPLES\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > my-profile.conf\n  ugreen --dry-run profile apply my-profile.conf\n\nHardware compatibility is unverified here. Studio Pro HP206 protocol only;\nHiTune Max5c uses conflicting command IDs. No firmware, reset, raw-write or\nfind-headphones actions are provided. Profiles may apply partially on failure.\n";

#[derive(Debug)]
struct Options {
    address: Option<String>,
    model: Option<String>,
    channel: u8,
    timeout: Duration,
    dry_run: bool,
    command: Vec<String>,
}
fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut o = Options {
        address: None,
        model: None,
        channel: 1,
        timeout: Duration::from_secs(3),
        dry_run: false,
        command: Vec::new(),
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--address" => o.address = Some(args.next().ok_or("--address requires a value")?),
            "--model" => o.model = Some(args.next().ok_or("--model requires a value")?),
            "--channel" => {
                o.channel = args
                    .next()
                    .ok_or("--channel requires a value")?
                    .parse()
                    .map_err(|_| "invalid channel")?;
                if !(1..=30).contains(&o.channel) {
                    return Err("channel must be 1–30".into());
                }
            }
            "--timeout" => {
                let n: u64 = args
                    .next()
                    .ok_or("--timeout requires a value")?
                    .parse()
                    .map_err(|_| "invalid timeout")?;
                if !(1..=60).contains(&n) {
                    return Err("timeout must be 1–60 seconds".into());
                }
                o.timeout = Duration::from_secs(n);
            }
            "--dry-run" => o.dry_run = true,
            "--help" | "-h" => {
                o.command = vec!["help".into()];
                return Ok(o);
            }
            "--version" | "-V" => {
                o.command = vec!["version".into()];
                return Ok(o);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option '{arg}'")),
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
        return Err("only --model studio-pro is implemented; Max5c IDs are incompatible".into());
    }
    Ok(o)
}
fn connect(o: &Options) -> Result<Client<transport::Connection>, String> {
    if o.model.as_deref() != Some("studio-pro") {
        return Err(
            "pass --model studio-pro to explicitly select this model-specific protocol".into(),
        );
    }
    let address = o
        .address
        .as_deref()
        .ok_or("--address is required; use discover to list paired devices")?;
    eprintln!(
        "Using experimental Studio Pro HP206 protocol at {address}, RFCOMM channel {}",
        o.channel
    );
    transport::Connection::connect(address,o.channel,o.timeout).map(|io|Client::new(io,o.timeout)).map_err(|e|format!("Bluetooth connection failed: {e}. Check pairing, power, adapter support, and whether another app owns the control channel"))
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
fn apply(o: &Options, settings: &[Setting]) -> Result<(), String> {
    if o.dry_run {
        for s in settings {
            println!("{}={}  {}", s.key, s.value, protocol::hex(&s.packet()));
        }
        println!("Dry run: no Bluetooth connection or device writes");
        return Ok(());
    }
    let mut c = connect(o)?;
    // Initial query must be understood before the first setting write.
    c.info()
        .map_err(|e| format!("preflight status failed; no settings sent: {e}"))?;
    for (i, s) in settings.iter().enumerate() {
        c.set(s).map_err(|e|format!("{} of {} settings verified before failure at {}={}: {e}. Earlier settings are not rolled back",i,settings.len(),s.key,s.value))?;
        println!("Verified {}={}", s.key, s.value);
    }
    Ok(())
}
fn run(o: Options) -> Result<(), String> {
    let args = &o.command;
    match args[0].as_str() {
        "tui" => {
            exact(args, 1, "ugreen [OPTIONS] tui")?;
            if o.dry_run {
                return Err("tui cannot be combined with --dry-run; use --dry-run set KEY VALUE for offline packets".into());
            }
            #[cfg(feature = "tui")]
            {
                ugreen_cli::tui::run(ugreen_cli::tui::Config {
                    address: o.address,
                    model_confirmed: o.model.as_deref() == Some("studio-pro"),
                    channel: o.channel,
                    timeout: o.timeout,
                })
                .map_err(|e| e.to_string())?;
            }
            #[cfg(not(feature = "tui"))]
            return Err("TUI feature is disabled in this CLI-only build; rebuild without --no-default-features".into());
        }
        "help" => {
            exact(args, 1, "ugreen help")?;
            print!("{HELP}");
        }
        "version" => {
            exact(args, 1, "ugreen --version")?;
            println!("ugreen {}", env!("CARGO_PKG_VERSION"));
        }
        "models" => {
            exact(args, 1, "ugreen models")?;
            println!("studio-pro: protocol based on UGREEN Studio Pro HP206 reference code and capture\nHardware test status: NOT VERIFIED on this build\nHiTune Max5c: NOT SUPPORTED (conflicting command IDs)\nOther UGREEN models/retail variants: NOT VERIFIED");
        }
        "commands" => {
            exact(args, 1, "ugreen commands")?;
            println!("{}", settings::HELP);
        }
        "decode" => {
            exact(args, 2, "ugreen decode 'DD EE FF ...'")?;
            let bytes = protocol::parse_hex(&args[1])?;
            let mut decoder = protocol::Decoder::default();
            let frames = decoder.feed(&bytes);
            for f in &frames {
                println!(
                    "instruction=0x{:02X} success={} crc=valid payload={}",
                    f.instruction,
                    f.succeeded,
                    protocol::hex(&f.payload)
                );
            }
            if frames.is_empty()
                || decoder.pending_bytes() != 0
                || decoder.rejected_checksums != 0
                || decoder.discarded_bytes != 0
            {
                return Err(format!("capture is not a clean complete response stream: {} frames, {} rejected CRCs, {} discarded bytes, {} pending bytes",frames.len(),decoder.rejected_checksums,decoder.discarded_bytes,decoder.pending_bytes()));
            }
        }
        "discover" => {
            exact(args, 1, "ugreen discover")?;
            if o.dry_run {
                return Err("discover cannot be combined with --dry-run".into());
            }
            let devices = transport::list_paired()
                .map_err(|e| format!("paired-device listing failed: {e}"))?;
            if devices.is_empty() {
                println!("No paired devices returned. Pair the headphones in OS Bluetooth settings first.");
            }
            for d in devices {
                println!("{}  {}", d.address, terminal_text(&d.name));
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
            println!("model=studio-pro (user-selected; not device identity verification)");
            println!(
                "battery={}",
                info.battery()
                    .map(|v| format!("{v}%"))
                    .unwrap_or("unknown".into())
            );
            for key in settings::KEYS {
                println!("{key}={}", info.value(key).unwrap_or("unknown".into()));
            }
            let fw = c
                .firmware()
                .map_err(|e| format!("settings read, but firmware query failed: {e}"))?;
            println!("firmware={fw}");
            println!("raw_device_info={}", protocol::hex(&info.raw));
        }
        "set" => {
            exact(args, 3, "ugreen [OPTIONS] set KEY VALUE")?;
            apply(&o, &[Setting::parse(&args[1], &args[2])?])?;
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
                    let settings = settings::parse_profile(&read_profile(&args[2])?)?;
                    apply(&o, &settings)?;
                }
                _ => return Err("usage: ugreen profile example|export|apply FILE".into()),
            }
        }
        _ => return Err(format!("unknown command '{}'; run ugreen --help", args[0])),
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
    fn device_names_cannot_control_terminal() {
        assert_eq!(terminal_text("Studio\x1b[31m\n"), "Studio\\u{1b}[31m\\n");
        assert_eq!(terminal_text("Auriculares José"), "Auriculares José");
    }
    #[test]
    fn dry_run_needs_no_device() {
        assert!(run(parse(args("--dry-run set anc ultra")).unwrap()).is_ok());
    }
}
