use std::{
    env, fs,
    io::{IsTerminal, Read},
    process::ExitCode,
    time::Duration,
};
use ugreen_cli::{
    client::Client,
    i18n::{Lang, L},
    protocol,
    settings::{self, Setting},
    transport,
};

#[derive(Debug)]
struct Options {
    address: Option<String>,
    model: Option<String>,
    channel: Option<u8>,
    timeout: Duration,
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
        timeout: Duration::from_secs(3),
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
                o.timeout = Duration::from_secs(n);
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
fn connect(o: &Options) -> Result<Client<transport::Connection>, String> {
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
    transport::Connection::connect(address, o.channel.unwrap_or(1), o.timeout)
        .map(|io| Client::new(io, o.timeout))
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
                // The cached target, protocol choice, channel and
                // language fill defaults; explicit flags always win.
                let cache = ugreen_cli::cache::load();
                let autoconnect = o.autoconnect.unwrap_or(cache.autoconnect);
                if let Some(autoconnect) = o.autoconnect {
                    ugreen_cli::cache::save(&ugreen_cli::cache::Cache {
                        autoconnect,
                        ..cache.clone()
                    });
                }
                ugreen_cli::tui::run(ugreen_cli::tui::Config {
                    address: o.address.or(cache.address),
                    model_confirmed: o.model.as_deref() == Some("studio-pro")
                        || cache.model_confirmed,
                    channel: o.channel.or(cache.channel).unwrap_or(1),
                    timeout: o.timeout,
                    lang: o.lang.or(cache.lang).unwrap_or_else(Lang::detect),
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
                return Err(fill(
                    t.cli_bad_decode,
                    &[
                        &frames.len(),
                        &decoder.rejected_checksums,
                        &decoder.discarded_bytes,
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
            let devices =
                transport::list_paired().map_err(|e| fill(t.cli_paired_fail, &[&e.to_string()]))?;
            if devices.is_empty() {
                println!("{}", t.cli_no_paired);
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
