//! Help rendering. The full `--help` screen, per-command
//! help and the settings listing are all generated from
//! the command registry and the localized catalogs; no
//! help text is hand-maintained in translation source.
use crate::{
    cli::registry::{self, Category, CommandSpec},
    i18n::{txt, Lang, L},
};

/// Column where command and option descriptions start.
const DESCRIPTION_COLUMN: usize = 30;

/// Function that retrieves a localized description.
type LocalizedDescription = fn(&'static L) -> &'static str;

/// Help entry containing syntax and its localized description.
type HelpOption = (&'static str, LocalizedDescription);

/// Global CLI options in display order.
const OPTIONS: &[HelpOption] = &[
    ("--address XX:XX:XX:XX:XX:XX", |t| t.opt_address),
    ("--model studio-pro", |t| t.opt_model),
    ("--channel 1", |t| t.opt_channel),
    ("--timeout 3", |t| t.opt_timeout),
    ("--lang en", |t| t.opt_lang),
    ("--autoconnect", |t| t.opt_autoconnect),
    ("--no-autoconnect", |t| t.opt_no_autoconnect),
    ("--json", |t| t.opt_json),
    ("--dry-run", |t| t.opt_dry_run),
    ("--help, -h", |t| t.opt_help),
    ("--version, -V", |t| t.opt_version),
];

/// Example command lines. Command syntax is
/// language-independent, like all protocol text.
const EXAMPLES: &[&str] = &[
    "ugreen discover",
    "ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status",
    "ugreen --dry-run set anc ultra",
    "ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass",
    "ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > my-profile.conf",
    "ugreen --dry-run profile apply my-profile.conf",
];

/// The full `ugreen --help` screen in a language.
pub fn full(lang: Lang) -> String {
    let t = txt(lang);
    let mut out = String::new();
    out.push_str(&format!(
        "ugreen {} — {}\n\n",
        env!("CARGO_PKG_VERSION"),
        t.help_subtitle
    ));
    out.push_str(&format!("{}\n  ugreen [OPTIONS] COMMAND\n\n", t.help_usage));
    for (category, header) in [
        (Category::Offline, t.help_offline),
        (Category::Bluetooth, t.help_bluetooth),
        (Category::CaptureGated, t.help_capture_gated),
    ] {
        out.push_str(&format!("{header}\n"));
        render_category(&mut out, t, category);
        out.push('\n');
    }
    out.push_str(&format!("{}\n", t.help_options));
    for (syntax, about) in OPTIONS {
        out.push_str(&line(syntax, about(t)));
    }
    out.push('\n');
    out.push_str(&format!("{}\n", t.help_examples));
    for example in EXAMPLES {
        out.push_str(&format!("  {example}\n"));
    }
    out.push('\n');
    out.push_str(t.help_footer);
    out.push('\n');
    out
}

/// The `ugreen help COMMAND` / `COMMAND --help` screen.
pub fn command(lang: Lang, path: &[&'static CommandSpec]) -> String {
    let t = txt(lang);
    let spec = *path.last().expect("non-empty path");
    let mut out = String::new();
    out.push_str(&format!(
        "ugreen {} — {}\n\n",
        display_name(path),
        (spec.about)(t)
    ));
    out.push_str(&format!(
        "{}\n  {}\n",
        t.help_usage,
        registry::usage_line(path)
    ));
    if spec.supports_json {
        out.push_str(&format!("\n--json: {}\n", t.opt_json));
    }
    out
}

/// One command row: two-space indent, the name and
/// argument syntax padded to the description column.
fn line(left: &str, about: &str) -> String {
    format!("  {left:<DESCRIPTION_COLUMN$}{about}\n")
}

fn display_name(path: &[&'static CommandSpec]) -> String {
    path.iter()
        .map(|spec| spec.name)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every help-listed command as its display path,
/// expanding nested subcommands to their own rows.
/// A hidden command with subcommands (e.g. `profile`,
/// which only runs through its subcommands) still
/// lists them under its name; a hidden leaf
/// (e.g. `version`) is omitted entirely.
fn listed_commands() -> Vec<Vec<&'static CommandSpec>> {
    fn visit(
        spec: &'static CommandSpec,
        path: &mut Vec<&'static CommandSpec>,
        entries: &mut Vec<Vec<&'static CommandSpec>>,
    ) {
        path.push(spec);

        if spec.subcommands.is_empty() {
            if !spec.hidden {
                entries.push(path.clone());
            }
        } else {
            for &sub in spec.subcommands {
                visit(sub, path, entries);
            }
        }

        path.pop();
    }

    let mut entries = Vec::new();
    let mut path = Vec::new();

    for &spec in registry::COMMANDS {
        visit(spec, &mut path, &mut entries);
    }

    entries
}

fn render_category(out: &mut String, t: &'static L, category: Category) {
    let entries: Vec<Vec<&CommandSpec>> = listed_commands()
        .into_iter()
        .filter(|path| path.last().is_some_and(|spec| spec.category == category))
        .collect();
    let mut index = 0;
    while index < entries.len() {
        let path = &entries[index];
        let spec = *path.last().expect("non-empty path");
        match spec.help_group {
            // Consecutive grouped commands render as one
            // comma-joined line, e.g. the capture-gated
            // queries.
            Some(group) => {
                let mut names = vec![display_name(path)];
                let mut next = index + 1;
                while next < entries.len() {
                    let sibling = *entries[next].last().expect("non-empty path");
                    if sibling.help_group != Some(group) {
                        break;
                    }
                    names.push(display_name(&entries[next]));
                    next += 1;
                }
                out.push_str(&format!("  {}\n", names.join(", ")));
                index = next;
            }
            None => {
                let left = if spec.usage.is_empty() {
                    display_name(path)
                } else {
                    format!("{} {}", display_name(path), spec.usage)
                };
                out.push_str(&line(&left, (spec.about)(t)));
                index += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_help_keeps_the_historical_layout() {
        let help = full(Lang::En);
        assert!(help.starts_with(&format!(
            "ugreen {} — unofficial UGREEN Studio Pro Bluetooth CLI",
            env!("CARGO_PKG_VERSION")
        )));
        assert!(help.contains("USAGE\n  ugreen [OPTIONS] COMMAND"));
        assert!(help.contains("OFFLINE COMMANDS"));
        assert!(help.contains("BLUETOOTH COMMANDS (pair in OS settings first)"));
        assert!(help.contains("CAPTURE-GATED QUERIES (currently refuse before connecting)"));
        assert!(help.contains("OPTIONS (before COMMAND)"));
        assert!(help.contains("EXAMPLES"));
        // The capture-gated queries share one line.
        assert!(help.contains("  peers, codec, id, color, box-mac\n"));
        // Descriptions align at the description column.
        assert!(help.contains(&format!("  {:<30}{}", "help [COMMAND]", "Show this help")));
        assert!(help.contains(&format!(
            "  {:<30}{}",
            "set KEY VALUE", "Change one setting, then verify readback"
        )));
        assert!(help.contains(&format!(
            "  {:<30}{}",
            "--address XX:XX:XX:XX:XX:XX", "Explicit target Bluetooth address"
        )));
    }

    #[test]
    fn generated_commands_listing_matches_the_settings_registry() {
        assert_eq!(
            crate::settings::commands_help(),
            "anc: off|ultra|general|gentle|adaptive|ambient\n\
             eq: classic|jazz|electronic|pop|classical|rock|bass|treble\n\
             game, spatial, dual, wind: on|off\n\
             prompts: voice|beeps\n\
             volume-up-action, volume-down-action: none|next|previous"
        );
    }

    #[test]
    fn every_locale_renders_the_full_screen() {
        for lang in Lang::ALL {
            let help = full(lang);
            assert!(help.contains("ugreen "), "{lang:?}");
            assert!(help.contains("capture"), "{lang:?}");
            assert!(help.contains("discover"), "{lang:?}");
            assert!(!help.contains("{}"), "{lang:?}");
        }
    }

    #[test]
    fn spanish_uses_its_own_usage_header() {
        assert!(full(Lang::Es).contains("USO"));
        assert!(!full(Lang::Es).contains("USAGE"));
    }

    #[test]
    fn command_help_shows_nested_usage() {
        let status = registry::lookup("status").expect("registered");
        let screen = command(Lang::En, &[status]);
        assert!(screen.contains("ugreen status — Read battery, firmware and settings"));
        assert!(screen.contains("ugreen [OPTIONS] status"));
        let profile = registry::lookup("profile").expect("registered");
        let apply = profile.subcommands[2];
        let screen = command(Lang::En, &[profile, apply]);
        assert!(screen
            .contains("ugreen profile apply — Validate profile, apply and verify each setting"));
        assert!(screen.contains("ugreen [OPTIONS] profile apply FILE"));
    }
}
