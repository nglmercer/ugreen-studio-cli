//! The declarative command registry: the single authority
//! for command metadata. Parsing, help generation, usage
//! lines and future completion/introspection all derive
//! from `COMMANDS`. Behavior lives in `handlers`; this
//! module owns definitions only.
use crate::{cli::context::Invocation, i18n::L};

/// A command handler: pure execution, independent of
/// parsing and translation. Receives the parsed invocation.
pub type Handler = fn(&Invocation) -> Result<(), String>;

/// Help section a command is listed under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    /// No Bluetooth or host hardware access at launch.
    Offline,
    /// Connects to the device or reads host-side state.
    Bluetooth,
    /// Refuses before connecting until an official-app
    /// capture verifies the operation.
    CaptureGated,
}

/// What a command may touch once running. Registry
/// metadata for introspection; handlers still enforce
/// every capability, model and dry-run gate themselves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessPolicy {
    Offline,
    HostRead,
    DeviceRead,
    DeviceWrite,
    CaptureGated,
}

/// One command definition. Everything user-facing about a
/// command lives here so adding a command needs exactly
/// one registration and one handler.
pub struct CommandSpec {
    /// Command name as typed on the command line.
    pub name: &'static str,
    /// Alternative names accepted by the parser.
    pub aliases: &'static [&'static str],
    /// Argument syntax shown in help and usage lines.
    pub usage: &'static str,
    /// Localized one-line description.
    pub about: fn(&'static L) -> &'static str,
    pub category: Category,
    pub access: AccessPolicy,
    /// Inclusive bounds on the argument count.
    pub min_args: usize,
    pub max_args: usize,
    pub handler: Handler,
    /// Nested subcommands, e.g. `profile example|export|apply`.
    pub subcommands: &'static [&'static CommandSpec],
    /// Consecutive commands sharing a group key render on
    /// one comma-joined help line.
    pub help_group: Option<&'static str>,
    /// Hide from the help listing (e.g. flag-only `version`).
    pub hidden: bool,
}

const HELP: CommandSpec = CommandSpec {
    name: "help",
    aliases: &[],
    usage: "[COMMAND]",
    about: |t| t.cmd_help,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: 1,
    handler: crate::cli::handlers::app::help,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const TUI: CommandSpec = CommandSpec {
    name: "tui",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_tui,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::app::tui,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const MODELS: CommandSpec = CommandSpec {
    name: "models",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_models,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::app::models,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const COMMANDS_LIST: CommandSpec = CommandSpec {
    name: "commands",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_commands,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::app::commands,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const DECODE: CommandSpec = CommandSpec {
    name: "decode",
    aliases: &[],
    usage: "HEX",
    about: |t| t.cmd_decode,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 1,
    max_args: 1,
    handler: crate::cli::handlers::analysis::decode,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const ANALYZE: CommandSpec = CommandSpec {
    name: "analyze",
    aliases: &[],
    usage: "HEX",
    about: |t| t.cmd_analyze,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 1,
    max_args: 1,
    handler: crate::cli::handlers::analysis::analyze,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const CANDIDATE: CommandSpec = CommandSpec {
    name: "candidate",
    aliases: &[],
    usage: "OP [ARGS]",
    about: |t| t.cmd_candidate,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: usize::MAX,
    handler: crate::cli::handlers::analysis::candidate,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const PROFILE: CommandSpec = CommandSpec {
    name: "profile",
    aliases: &[],
    usage: "example|export|apply FILE",
    about: |t| t.cmd_profile,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 1,
    max_args: 2,
    handler: crate::cli::handlers::settings::profile,
    subcommands: &[&PROFILE_EXAMPLE, &PROFILE_EXPORT, &PROFILE_APPLY],
    help_group: None,
    hidden: true,
};

const PROFILE_EXAMPLE: CommandSpec = CommandSpec {
    name: "example",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_profile_example,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::settings::profile_example,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const PROFILE_EXPORT: CommandSpec = CommandSpec {
    name: "export",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_profile_export,
    category: Category::Bluetooth,
    access: AccessPolicy::DeviceRead,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::settings::profile_export,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const PROFILE_APPLY: CommandSpec = CommandSpec {
    name: "apply",
    aliases: &[],
    usage: "FILE",
    about: |t| t.cmd_profile_apply,
    category: Category::Bluetooth,
    access: AccessPolicy::DeviceWrite,
    min_args: 1,
    max_args: 1,
    handler: crate::cli::handlers::settings::profile_apply,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const DISCOVER: CommandSpec = CommandSpec {
    name: "discover",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_discover,
    category: Category::Bluetooth,
    access: AccessPolicy::HostRead,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::discover,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const CAPTURE: CommandSpec = CommandSpec {
    name: "capture",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_capture,
    category: Category::Bluetooth,
    access: AccessPolicy::DeviceRead,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::capture,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const STATUS: CommandSpec = CommandSpec {
    name: "status",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_status,
    category: Category::Bluetooth,
    access: AccessPolicy::DeviceRead,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::status,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const SET: CommandSpec = CommandSpec {
    name: "set",
    aliases: &[],
    usage: "KEY VALUE",
    about: |t| t.cmd_set,
    category: Category::Bluetooth,
    access: AccessPolicy::DeviceWrite,
    min_args: 2,
    max_args: 2,
    handler: crate::cli::handlers::settings::set,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

/// The five capture-gated queries share one help line.
const QUERY_GROUP: Option<&str> = Some("capture-queries");

const PEERS: CommandSpec = CommandSpec {
    name: "peers",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_query,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::peers,
    subcommands: &[],
    help_group: QUERY_GROUP,
    hidden: false,
};

const CODEC: CommandSpec = CommandSpec {
    name: "codec",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_query,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::codec,
    subcommands: &[],
    help_group: QUERY_GROUP,
    hidden: false,
};

const ID: CommandSpec = CommandSpec {
    name: "id",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_query,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::id,
    subcommands: &[],
    help_group: QUERY_GROUP,
    hidden: false,
};

const COLOR: CommandSpec = CommandSpec {
    name: "color",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_query,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::color,
    subcommands: &[],
    help_group: QUERY_GROUP,
    hidden: false,
};

const BOX_MAC: CommandSpec = CommandSpec {
    name: "box-mac",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_query,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::device::box_mac,
    subcommands: &[],
    help_group: QUERY_GROUP,
    hidden: false,
};

const CONTROL: CommandSpec = CommandSpec {
    name: "control",
    aliases: &[],
    usage: "OP [ARGS]",
    about: |t| t.cmd_control,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 0,
    max_args: usize::MAX,
    handler: crate::cli::handlers::analysis::control,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const FIND: CommandSpec = CommandSpec {
    name: "find",
    aliases: &[],
    usage: "SIDE SECONDS TARGET",
    about: |t| t.cmd_find,
    category: Category::CaptureGated,
    access: AccessPolicy::CaptureGated,
    min_args: 3,
    max_args: 3,
    handler: crate::cli::handlers::analysis::find,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const AUDIO_DEVICES: CommandSpec = CommandSpec {
    name: "audio-devices",
    aliases: &[],
    usage: "",
    about: |t| t.cmd_audio_devices,
    category: Category::CaptureGated,
    access: AccessPolicy::HostRead,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::app::audio_devices,
    subcommands: &[],
    help_group: None,
    hidden: false,
};

const VERSION: CommandSpec = CommandSpec {
    name: "version",
    aliases: &[],
    usage: "",
    about: |t| t.opt_version,
    category: Category::Offline,
    access: AccessPolicy::Offline,
    min_args: 0,
    max_args: 0,
    handler: crate::cli::handlers::app::version,
    subcommands: &[],
    help_group: None,
    hidden: true,
};

/// Every registered command, in help-listing order.
/// `profile` sits after `set` so its subcommands
/// land at the end of the Bluetooth section, and
/// its example subcommand lands at the end of the
/// offline section, matching the historical layout.
pub const COMMANDS: &[&CommandSpec] = &[
    &HELP,
    &TUI,
    &MODELS,
    &COMMANDS_LIST,
    &DECODE,
    &ANALYZE,
    &CANDIDATE,
    &DISCOVER,
    &CAPTURE,
    &STATUS,
    &SET,
    &PROFILE,
    &PEERS,
    &CODEC,
    &ID,
    &COLOR,
    &BOX_MAC,
    &CONTROL,
    &FIND,
    &AUDIO_DEVICES,
    &VERSION,
];

/// Find a top-level command by name or alias.
pub fn lookup(name: &str) -> Option<&'static CommandSpec> {
    COMMANDS
        .iter()
        .copied()
        .find(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// Resolve a command path: `"status"` or `"profile apply"`.
/// Returns the full path so help can show nested usage.
pub fn lookup_path(path: &str) -> Option<Vec<&'static CommandSpec>> {
    let (name, rest) = path.split_once(' ').unwrap_or((path, ""));
    let parent = lookup(name)?;
    if rest.is_empty() {
        return Some(vec![parent]);
    }
    let sub = parent
        .subcommands
        .iter()
        .copied()
        .find(|spec| spec.name == rest)?;
    Some(vec![parent, sub])
}

/// Execute a parsed invocation through its registered handler.
pub fn dispatch(invocation: &Invocation) -> Result<(), String> {
    (invocation.target().handler)(invocation)
}

/// `ugreen [OPTIONS] profile apply FILE`-style usage string.
pub fn usage_line(path: &[&'static CommandSpec]) -> String {
    let mut out = String::from("ugreen [OPTIONS]");
    for spec in path {
        out.push(' ');
        out.push_str(spec.name);
    }
    if let Some(last) = path.last() {
        if !last.usage.is_empty() {
            out.push(' ');
            out.push_str(last.usage);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_has_a_handler_and_metadata() {
        for spec in COMMANDS {
            assert!(!spec.name.is_empty(), "empty name");
            assert!(
                lookup(spec.name).is_some(),
                "{} not found by lookup",
                spec.name
            );
            // The handler pointer must be set: a null fn
            // pointer would panic on dispatch.
            let _ = spec.handler;
            if spec.subcommands.is_empty() {
                assert!(spec.min_args <= spec.max_args);
            }
        }
    }

    #[test]
    fn subcommands_are_registered_under_their_parent() {
        let profile = lookup("profile").expect("profile registered");
        assert_eq!(profile.subcommands.len(), 3);
        for sub in profile.subcommands {
            assert!(!sub.name.is_empty());
            assert!(sub.min_args <= sub.max_args);
        }
    }

    #[test]
    fn usage_lines_include_arguments() {
        let set = lookup("set").expect("set registered");
        assert_eq!(usage_line(&[set]), "ugreen [OPTIONS] set KEY VALUE");
        let profile = lookup("profile").expect("profile registered");
        assert_eq!(
            usage_line(&[profile]),
            "ugreen [OPTIONS] profile example|export|apply FILE"
        );
        let apply = profile.subcommands[2];
        assert_eq!(
            usage_line(&[profile, apply]),
            "ugreen [OPTIONS] profile apply FILE"
        );
    }

    #[test]
    fn lookup_path_resolves_nested_commands() {
        let path = lookup_path("profile apply").expect("path resolves");
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].name, "profile");
        assert_eq!(path[1].name, "apply");
        assert!(lookup_path("profile missing").is_none());
        assert!(lookup_path("missing").is_none());
    }

    #[test]
    fn capture_gated_queries_share_one_help_line() {
        let grouped: Vec<_> = COMMANDS
            .iter()
            .filter(|spec| spec.help_group == QUERY_GROUP)
            .collect();
        assert_eq!(grouped.len(), 5);
        for spec in grouped {
            assert_eq!(spec.category, Category::CaptureGated);
        }
    }
}
