//! Offline analysis and capture-gated operations:
//! decode, analyze, candidate, control and find.
//! Static-APK candidates never touch Bluetooth.
use crate::cli::context::{connect, fill, Invocation};
use crate::protocol;

pub fn decode(invocation: &Invocation) -> Result<(), String> {
    let bytes = protocol::parse_hex(&invocation.args[0])?;
    let mut decoder = protocol::Decoder::default();
    let frames = decoder.feed(&bytes);
    for frame in &frames {
        match frame {
            protocol::IncomingFrame::Response(frame) => {
                let mut line = format!(
                    "instruction=0x{:02X} success={} crc=valid payload={}",
                    frame.instruction,
                    frame.succeeded,
                    protocol::hex(&frame.payload)
                );
                if let Some((feature, verified)) =
                    protocol::static_analysis::describe_instruction(frame.instruction)
                {
                    if verified {
                        line.push_str(&format!(" feature={feature}"));
                    } else {
                        line.push_str(&format!(" feature={feature} (unverified)"));
                    }
                }
                println!("{line}");
            }
            protocol::IncomingFrame::Notification(frame) => {
                // The event code is frame byte 5 (payload[0]);
                // the envelope byte (`kind`) is 0x02 in every
                // capture so far. Event labels are static APK
                // evidence, never capture-verified.
                let event = frame.payload[0];
                let feature = match protocol::static_analysis::describe_notification_event(event) {
                    Some(name) => format!(" feature={name} (unverified)"),
                    None => String::new(),
                };
                println!(
                    "notification envelope=0x{:02X} event=0x{:02X} payload={}{feature}",
                    frame.kind,
                    event,
                    protocol::hex(&frame.payload)
                );
            }
            protocol::IncomingFrame::Unknown(frame) => {
                let mut line = format!(
                    "unknown instruction=0x{:02X} payload={} rx-meaning=unverified",
                    frame.instruction,
                    protocol::hex(&frame.payload)
                );
                if let Some((feature, _)) =
                    protocol::static_analysis::describe_instruction(frame.instruction)
                {
                    line.push_str(&format!(" feature={feature} (unverified)"));
                }
                println!("{line}");
            }
        }
    }
    let stats = decoder.stats;
    let t = invocation.txt();
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
    Ok(())
}

pub fn analyze(invocation: &Invocation) -> Result<(), String> {
    let bytes = protocol::parse_hex(&invocation.args[0])?;
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
            protocol::IncomingFrame::Response(frame) if frame.instruction == protocol::INFO => {
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
    Ok(())
}

pub fn candidate(invocation: &Invocation) -> Result<(), String> {
    let command = protocol::static_analysis::Command::parse(&invocation.args)?;
    println!("# static APK candidate; unverified; no Bluetooth access");
    println!(
        "{}",
        protocol::hex(&command.packet().map_err(|e| e.to_string())?)
    );
    Ok(())
}

pub fn control(invocation: &Invocation) -> Result<(), String> {
    let command = protocol::static_analysis::Command::parse(&invocation.args)?;
    if command.control_capability().is_none() {
        return Err("query is not a control; use its query command".into());
    }
    if invocation.options.dry_run {
        println!("# static APK candidate; unverified; no Bluetooth access");
        println!(
            "{}",
            protocol::hex(&command.packet().map_err(|e| e.to_string())?)
        );
    } else {
        command.check_control().map_err(|e| e.to_string())?;
        let mut client = connect(&invocation.options)?;
        client
            .info()
            .map_err(|e| format!("control preflight failed: {e}"))?;
        let outcome = client.control(&command).map_err(|e| e.to_string())?;
        if command == protocol::static_analysis::Command::FactoryReset {
            drop(client);
            let address = crate::bluetooth::BluetoothAddress::parse(
                invocation
                    .options
                    .address
                    .as_deref()
                    .expect("connect validated address"),
            )
            .map_err(|e| e.to_string())?;
            crate::reset::cleanup(&address).map_err(|e| e.to_string())?;
            println!("reset acknowledged; host pairing and registry entry removed");
            return Ok(());
        }
        match outcome {
            crate::client::ControlOutcome::Readback(_) => {
                println!("control verified by device-info readback")
            }
            crate::client::ControlOutcome::Acknowledged(_) => {
                println!("control acknowledged; resulting device state not independently confirmed")
            }
            crate::client::ControlOutcome::SentUnconfirmed => {
                println!("control sent; finder state and audio playback unconfirmed")
            }
        }
    }
    Ok(())
}

pub fn find(invocation: &Invocation) -> Result<(), String> {
    let side = match invocation.args[0].as_str() {
        "left" => protocol::static_analysis::Side::Left,
        "right" => protocol::static_analysis::Side::Right,
        _ => return Err("finder side must be left or right".into()),
    };
    let seconds: u8 = invocation.args[1]
        .parse()
        .map_err(|_| "finder duration must be 1..30 seconds")?;
    if !(1..=30).contains(&seconds) {
        return Err("finder duration must be 1..30 seconds".into());
    }
    if invocation.options.dry_run {
        return Err("use candidate find left|right on|off for offline packets".into());
    }
    crate::finder::validate(seconds, &invocation.args[2]).map_err(|e| e.to_string())?;
    protocol::static_analysis::Command::Find {
        enabled: true,
        side,
    }
    .check_control()
    .map_err(|e| e.to_string())?;
    let interrupts = crate::interrupt::Guard::install().map_err(|e| e.to_string())?;
    let mut client = connect(&invocation.options)?;
    client
        .info()
        .map_err(|e| format!("finder preflight failed: {e}"))?;
    crate::finder::ring_cancellable(&mut client, side, seconds, &invocation.args[2], || {
        interrupts.interrupted()
    })
    .map_err(|e| e.to_string())?;
    println!("host finder playback completed; stop sent; device state unconfirmed");
    Ok(())
}
