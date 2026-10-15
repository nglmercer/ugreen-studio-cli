//! Device identity, independent host/control states, and observation age.
use super::*;

pub(super) fn render(frame: &mut Frame<'_>, regions: &Regions, app: &App) {
    let t = app.t();
    let vendor_codec = app
        .codecs
        .map(|pairs| {
            use crate::protocol::static_analysis::Codec;
            let names = pairs.map(|(_, codec)| match codec {
                Codec::Sbc => "SBC".into(),
                Codec::Aac => "AAC".into(),
                Codec::Ldac => "LDAC".into(),
                Codec::Unknown(v) => format!("unknown({v})"),
            });
            format!("Codecs (last query): {}/{}", names[0], names[1])
        })
        .unwrap_or_else(|| t.codec_line.into());
    let codec_line = match &app.audio {
        Some(observations) if !observations.is_empty() => format!(
            "{} | {}",
            observations
                .iter()
                .map(|status| format!(
                    "{}: {}/{} {} ({}s)",
                    t.host_commands[0],
                    status.profile.label(),
                    clean(
                        status
                            .codec
                            .as_ref()
                            .map(|codec| codec.label())
                            .unwrap_or(t.unknown)
                    ),
                    status.activity.label(),
                    status.observed_at.elapsed().unwrap_or_default().as_secs()
                ))
                .collect::<Vec<_>>()
                .join("; "),
            vendor_codec
        ),
        Some(_) => format!(
            "{}: {} | {}",
            t.host_commands[0], t.unavailable, vendor_codec
        ),
        None => vendor_codec,
    };
    // The OS Bluetooth link and the RFCOMM control session are shown
    // as two separate lines: an open session implies the link, but the
    // link can be up without a session (never the other way around).
    let bluetooth = match app.host_connection {
        HostConnectionState::Connected => t.connected,
        HostConnectionState::Disconnected => t.disconnected,
        HostConnectionState::Unknown => t.unknown,
    };
    let control = match app.control_state {
        crate::bluetooth::ControlSessionState::Connected => t.connected,
        crate::bluetooth::ControlSessionState::Disconnected => t.disconnected,
        crate::bluetooth::ControlSessionState::Connecting => t.status_connecting,
        crate::bluetooth::ControlSessionState::Error => t.unavailable,
    };
    let protocol: String = if app.model_confirmed {
        t.protocol_on.into()
    } else {
        t.protocol_off.into()
    };
    let battery = app
        .info
        .as_ref()
        .and_then(|info| info.battery())
        .map_or(t.unavailable.into(), |v| fill(t.battery_pct, &[&v]));
    let snapshot = if app.stale {
        t.snapshot_stale.into()
    } else if let Some(at) = app.observed_at {
        fill(t.snapshot_ago, &[&at.elapsed().as_secs()])
    } else {
        t.snapshot_never.into()
    };
    let state_style = |connected: bool| {
        Style::default()
            .fg(if connected {
                Color::Green
            } else {
                Color::Yellow
            })
            .add_modifier(Modifier::BOLD)
    };
    let header = vec![
        Line::from(Span::styled(
            format!(
                " {}{}  {}",
                t.bluetooth,
                bluetooth,
                app.host_observed_at
                    .map(|at| fill(t.snapshot_ago, &[&at.elapsed().as_secs()]))
                    .unwrap_or_else(|| t.snapshot_never.into())
            ),
            state_style(app.host_connection.is_connected()),
        )),
        Line::from(Span::styled(
            format!(" {}{}", t.control, control),
            state_style(app.control_state.is_connected()),
        )),
        Line::raw(format!(
            " {}{}",
            t.target,
            if app.address.is_empty() {
                t.target_none.into()
            } else {
                clean(&app.address)
            }
        )),
        Line::raw(format!(" {}{}", t.protocol, protocol)),
        Line::raw(format!(" {}{}  {}", t.battery, battery, codec_line)),
        Line::raw(format!(
            " {}{}  {}{}",
            t.firmware,
            clean(app.firmware.as_deref().unwrap_or(t.unavailable)),
            t.snapshot,
            snapshot
        )),
    ];
    frame.render_widget(
        Paragraph::new(header).block(panel(t.app_title)),
        regions.header,
    );
}
