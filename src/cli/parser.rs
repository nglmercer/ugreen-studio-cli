//! Registry-backed argument parser. Global options are
//! validated here; the first positional argument selects
//! a registered command, and its subcommand and argument
//! count are validated against the registry.
use crate::{
    cli::{
        context::{Invocation, Options},
        registry,
    },
    i18n::{txt, Lang},
};
use std::{io::IsTerminal, time::Duration};

/// Parse raw arguments into an invocation, or a
/// localized error. `--help` and `--version` short-circuit.
pub fn parse(args: Vec<String>) -> Result<Invocation, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--address" => {
                options.address = Some(need_value(&mut args, &options, "--address")?);
            }
            "--model" => {
                options.model = Some(need_value(&mut args, &options, "--model")?);
            }
            "--channel" => {
                let raw = need_value(&mut args, &options, "--channel")?;
                let channel: u8 = raw
                    .parse()
                    .map_err(|_| txt(lang_of(&options)).cli_bad_channel.to_string())?;
                if !(1..=30).contains(&channel) {
                    return Err(txt(lang_of(&options)).cli_bad_channel.into());
                }
                options.channel = Some(channel);
            }
            "--timeout" => {
                let raw = need_value(&mut args, &options, "--timeout")?;
                let seconds: u64 = raw
                    .parse()
                    .map_err(|_| txt(lang_of(&options)).cli_bad_timeout.to_string())?;
                if !(1..=60).contains(&seconds) {
                    return Err(txt(lang_of(&options)).cli_bad_timeout.into());
                }
                options.timeout = Some(Duration::from_secs(seconds));
            }
            "--lang" => {
                let raw = need_value(&mut args, &options, "--lang")?;
                options.lang = Some(Lang::from_code(&raw)?);
            }
            "--autoconnect" => options.autoconnect = Some(true),
            "--no-autoconnect" => options.autoconnect = Some(false),
            "--dry-run" => options.dry_run = true,
            "--help" | "-h" => {
                return command_invocation(
                    &options,
                    registry::lookup("help").expect("registered"),
                    Vec::new(),
                );
            }
            "--version" | "-V" => {
                return command_invocation(
                    &options,
                    registry::lookup("version").expect("registered"),
                    Vec::new(),
                );
            }
            _ if arg.starts_with('-') => {
                return Err(crate::cli::context::fill(
                    txt(lang_of(&options)).cli_bad_option,
                    &[&arg],
                ));
            }
            _ => {
                let command = registry::lookup(&arg).ok_or_else(|| {
                    crate::cli::context::fill(txt(lang_of(&options)).cli_unknown_command, &[&arg])
                })?;
                let rest: Vec<String> = args.collect();
                return resolve(&options, command, rest);
            }
        }
    }
    // No command given: the interactive TUI when both
    // streams are terminals, otherwise the help screen.
    let interactive =
        cfg!(feature = "tui") && std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let name = if interactive { "tui" } else { "help" };
    let command = registry::lookup(name).expect("registered");
    command_invocation(&options, command, Vec::new())
}

/// Language for parser errors: explicit flag, else environment.
fn lang_of(options: &Options) -> Lang {
    options.lang.unwrap_or_else(Lang::detect)
}

fn need_value(
    args: &mut impl Iterator<Item = String>,
    options: &Options,
    flag: &str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| crate::cli::context::fill(txt(lang_of(options)).cli_need_value, &[&flag]))
}

/// Validate a command (and its subcommand) and build the
/// invocation. The model gate runs before any execution.
fn resolve(
    options: &Options,
    command: &'static registry::CommandSpec,
    rest: Vec<String>,
) -> Result<Invocation, String> {
    check_model(options)?;
    // A `--help` or `-h` anywhere after the command
    // requests that command's own help screen.
    if let Some(index) = rest.iter().position(|arg| arg == "--help" || arg == "-h") {
        // Only nested commands contribute their
        // arguments to the help path; a variadic
        // command like `set anc --help` still
        // shows the `set` screen.
        let mut path = vec![command.name];
        if !command.subcommands.is_empty() {
            path.extend(rest[..index].iter().map(String::as_str));
        }
        return help_invocation(options, &path.join(" "));
    }
    if command.subcommands.is_empty() {
        validate_args(options, command, &rest, &[command])
    } else {
        // The first argument selects the subcommand.
        let Some(name) = rest.first() else {
            return Err(usage_error(options, &[command]));
        };
        let Some(sub) = command
            .subcommands
            .iter()
            .copied()
            .find(|spec| spec.name == name.as_str())
        else {
            return Err(usage_error(options, &[command]));
        };
        let args = rest[1..].to_vec();
        validate_args(options, sub, &args, &[command, sub])
    }
}

fn validate_args(
    options: &Options,
    spec: &'static registry::CommandSpec,
    args: &[String],
    path: &[&'static registry::CommandSpec],
) -> Result<Invocation, String> {
    if args.len() < spec.min_args || args.len() > spec.max_args {
        return Err(usage_error(options, path));
    }
    command_invocation(options, spec, args.to_vec())
}

fn command_invocation(
    options: &Options,
    command: &'static registry::CommandSpec,
    args: Vec<String>,
) -> Result<Invocation, String> {
    check_model(options)?;
    Ok(Invocation {
        options: options.clone(),
        command,
        subcommand: None,
        args,
    })
}

/// `command --help` rewrites to `help command...` so one
/// handler renders every help screen.
fn help_invocation(options: &Options, path: &str) -> Result<Invocation, String> {
    let help = registry::lookup("help").expect("registered");
    let args = if path.is_empty() {
        Vec::new()
    } else {
        vec![path.to_string()]
    };
    command_invocation(options, help, args)
}

fn usage_error(options: &Options, path: &[&'static registry::CommandSpec]) -> String {
    crate::cli::context::fill(
        txt(lang_of(options)).cli_bad_usage,
        &[&registry::usage_line(path)],
    )
}

/// Hardware commands require the explicit model selection;
/// it is never auto-detected or guessed.
fn check_model(options: &Options) -> Result<(), String> {
    if options
        .model
        .as_deref()
        .is_some_and(|model| model != "studio-pro")
    {
        return Err(txt(lang_of(options)).cli_bad_model.into());
    }
    Ok(())
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
        let invocation = parse(args("status")).unwrap();
        assert!(crate::cli::context::connect(&invocation.options).is_err());
        let invocation = parse(args("--model studio-pro status")).unwrap();
        assert!(crate::cli::context::connect(&invocation.options).is_err());
    }

    #[test]
    fn capture_refuses_dry_run_and_missing_target() {
        // --dry-run capture is rejected by the handler
        // before any connection attempt.
        let invocation = parse(args("--dry-run capture")).unwrap();
        assert!(registry::dispatch(&invocation).is_err());
        // Without the model, connecting fails.
        let invocation = parse(args("capture")).unwrap();
        assert!(registry::dispatch(&invocation).is_err());
        let invocation = parse(args("--model studio-pro capture")).unwrap();
        assert!(crate::cli::context::connect(&invocation.options).is_err());
    }

    #[test]
    fn autoconnect_flags_parse() {
        assert_eq!(
            parse(args("--autoconnect tui"))
                .unwrap()
                .options
                .autoconnect,
            Some(true)
        );
        assert_eq!(
            parse(args("--no-autoconnect tui"))
                .unwrap()
                .options
                .autoconnect,
            Some(false)
        );
        assert_eq!(parse(args("tui")).unwrap().options.autoconnect, None);
    }

    #[test]
    fn dry_run_needs_no_device() {
        let invocation = parse(args("--dry-run set anc ultra")).unwrap();
        assert!(registry::dispatch(&invocation).is_ok());
    }

    #[test]
    fn arg_counts_come_from_the_registry() {
        assert!(parse(args("set anc")).is_err());
        assert!(parse(args("set anc ultra extra")).is_err());
        assert!(parse(args("find left 1")).is_err());
        assert!(parse(args("decode")).is_err());
        assert!(parse(args("commands surprise")).is_err());
        assert!(parse(args("profile apply")).is_err());
        assert!(parse(args("profile missing")).is_err());
    }

    #[test]
    fn command_help_flag_rewrites_to_help() {
        let invocation = parse(args("status --help")).unwrap();
        assert_eq!(invocation.command.name, "help");
        assert_eq!(invocation.args, ["status"]);
        let invocation = parse(args("profile export --help")).unwrap();
        assert_eq!(invocation.command.name, "help");
        assert_eq!(invocation.args, ["profile export"]);
    }

    #[test]
    fn lang_flag_and_locale_detection() {
        let invocation = parse(args("--lang es status")).unwrap();
        assert_eq!(invocation.lang(), Lang::Es);
        assert!(invocation.txt().cmd_help.contains("Mostrar"));
        let invocation = parse(args("--lang en status")).unwrap();
        assert_eq!(invocation.lang(), Lang::En);
        assert!(invocation.txt().cmd_help.contains("Show"));
        assert_eq!(Lang::from_code("en").unwrap(), Lang::En);
    }

    #[test]
    fn unknown_commands_are_rejected_offline() {
        assert!(parse(args("wat")).is_err());
        assert!(parse(args("--dry-run wat")).is_err());
    }

    #[test]
    fn default_command_depends_on_the_terminal() {
        // Piped streams fall back to the help screen.
        let invocation = parse(Vec::new()).unwrap();
        assert_eq!(invocation.command.name, "help");
    }
}
