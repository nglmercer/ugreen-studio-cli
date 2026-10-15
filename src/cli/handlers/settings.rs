//! Setting writes and profile management. Every
//! write path shares one preflight-and-verify
//! policy; dry runs never connect.
use crate::cli::context::{connect, fill, read_profile, Invocation};
use crate::i18n::txt;
use crate::settings::{self, Setting};

/// Apply settings with the shared safety policy:
/// dry run prints packets offline; live runs do a
/// preflight info read, then write and verify each
/// setting by acknowledgement and readback.
fn apply(options: &crate::cli::context::Options, settings_list: &[Setting]) -> Result<(), String> {
    let lang = options.lang.unwrap_or_else(crate::i18n::Lang::detect);
    let t = txt(lang);
    for setting in settings_list {
        crate::application::validate_setting(setting).map_err(|error| error.to_string())?;
    }
    if options.dry_run {
        for setting in settings_list {
            println!(
                "{}={}  {}",
                setting.key,
                setting.value,
                crate::protocol::hex(&setting.packet())
            );
        }
        println!("{}", t.cli_dry_run_writes);
        return Ok(());
    }
    let mut client = connect(options)?;
    let firmware = client
        .firmware_in(lang)
        .map_err(|e| fill(t.cli_preflight_fail, &[&e.to_string()]))?;
    for setting in settings_list {
        crate::application::authorize_setting(
            Some(crate::models::Model::StudioPro),
            Some(&firmware),
            setting,
        )
        .map_err(|error| error.to_string())?;
    }
    // Initial query must be understood before the first setting write.
    let before = client
        .info_in(lang)
        .map_err(|e| fill(t.cli_preflight_fail, &[&e.to_string()]))?;
    for setting in settings_list {
        crate::application::validate_preflight(&before, setting)
            .map_err(|error| fill(t.cli_preflight_fail, &[&error.to_string()]))?;
    }
    for (index, setting) in settings_list.iter().enumerate() {
        client
            .set_for_model_in(lang, Some(crate::models::Model::StudioPro), setting)
            .map_err(|e| {
                fill(
                    t.cli_apply_fail,
                    &[
                        &index,
                        &settings_list.len(),
                        &setting.key,
                        &setting.value,
                        &e.to_string(),
                    ],
                )
            })?;
        println!("{}", fill(t.cli_verified, &[&setting.key, &setting.value]));
    }
    Ok(())
}

pub fn set(invocation: &Invocation) -> Result<(), String> {
    let settings_list = [Setting::parse_in(
        invocation.lang(),
        &invocation.args[0],
        &invocation.args[1],
    )?];
    apply(&invocation.options, &settings_list)
}

pub fn profile(invocation: &Invocation) -> Result<(), String> {
    // Reached only when a subcommand cannot be
    // resolved; the parser routes valid subcommands
    // to their own handlers.
    Err(usage(invocation))
}

pub fn profile_example(invocation: &Invocation) -> Result<(), String> {
    let _ = invocation;
    print!("# UGREEN Studio Pro profile v1\nmodel=studio-pro\nanc=ultra\neq=classic\ngame=off\nspatial=off\n");
    Ok(())
}

pub fn profile_export(invocation: &Invocation) -> Result<(), String> {
    if invocation.options.dry_run {
        return Err(
            "profile export requires reading the device; use profile example offline".into(),
        );
    }
    let mut client = connect(&invocation.options)?;
    print!(
        "{}",
        client
            .info_in(invocation.lang())
            .map_err(|e| e.to_string())?
            .profile()
    );
    Ok(())
}

pub fn profile_apply(invocation: &Invocation) -> Result<(), String> {
    let settings_list =
        settings::parse_profile_in(invocation.lang(), &read_profile(&invocation.args[0])?)?;
    apply(&invocation.options, &settings_list)
}

fn usage(invocation: &Invocation) -> String {
    fill(
        invocation.txt().cli_bad_usage,
        &[&crate::cli::registry::usage_line(&[
            crate::cli::registry::lookup("profile").expect("registered"),
        ])],
    )
}
