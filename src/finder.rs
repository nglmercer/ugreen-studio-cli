//! Host-generated finder sound. No official-app audio assets are copied.
//! Linux playback explicitly targets a PipeWire sink and disables fallback.
use crate::{
    client::Client,
    protocol::static_analysis::{Command, Side},
};
use std::io::{self, Read, Write};

/// Start the selected finder, play a bounded host tone, then send stop even
/// if playback fails. Success proves host playback/stop send, not audibility
/// or device finder-state confirmation. Both wire operations remain gated.
pub fn ring<T: Read + Write>(
    client: &mut Client<T>,
    side: Side,
    seconds: u8,
    target: &str,
) -> io::Result<()> {
    ring_cancellable(client, side, seconds, target, || false)
}
/// Finder operation with caller-owned cancellation. Cancellation is checked
/// before start and while playing; once start is attempted, stop is attempted
/// even if cancellation or an I/O error occurs.
pub fn ring_cancellable<T: Read + Write>(
    client: &mut Client<T>,
    side: Side,
    seconds: u8,
    target: &str,
    cancelled: impl Fn() -> bool,
) -> io::Result<()> {
    validate(seconds, target)?;
    Command::Find {
        enabled: true,
        side,
    }
    .check_control()?;
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (client, cancelled);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "finder playback requires Linux PipeWire or Windows waveform audio",
        ))
    }
    #[cfg(target_os = "windows")]
    {
        if cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "finder cancelled before start",
            ));
        }
        run(
            |enabled| client.control(&Command::Find { enabled, side }).map(|_| ()),
            || windows::play(side, seconds, target, &cancelled),
        )
    }
    #[cfg(target_os = "linux")]
    {
        let tone = linux::ToneFile::create(side, seconds)?;
        if cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "finder cancelled before start",
            ));
        }
        run(
            |enabled| client.control(&Command::Find { enabled, side }).map(|_| ()),
            || linux::play(&tone, seconds, target, &cancelled),
        )
    }
}
/// Validate finder duration and explicit playback target before any I/O.
pub fn validate(seconds: u8, target: &str) -> io::Result<()> {
    if !(1..=30).contains(&seconds)
        || target.is_empty()
        || target.len() > 256
        || (cfg!(target_os = "linux") && target == "0")
        || target.bytes().any(|b| b.is_ascii_control())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "finder needs 1..30 seconds and an explicit nonzero PipeWire sink name or serial",
        ));
    }
    #[cfg(target_os = "windows")]
    if target
        .parse::<u32>()
        .ok()
        .filter(|v| *v != u32::MAX)
        .is_none()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows finder target must be an explicit waveform device index",
        ));
    }
    Ok(())
}
#[cfg(any(target_os = "linux", target_os = "windows", test))]
fn run(
    mut control: impl FnMut(bool) -> io::Result<()>,
    play: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    let result = control(true).and_then(|_| play());
    let stop = control(false);
    match (result, stop) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(io::Error::new(
            error.kind(),
            format!("finder failed: {error}; stop sent, device state unconfirmed"),
        )),
        (result, Err(error)) => Err(io::Error::new(
            error.kind(),
            format!(
                "finder stop failed: {error}; prior result: {result:?}; device state unconfirmed"
            ),
        )),
    }
}
/// Original stereo PCM WAV: 900 Hz pulses with short fades, 12% peak amplitude.
/// Only the selected channel contains samples. Duration is bounded to 30 s.
pub fn tone_wav(side: Side, seconds: u8) -> io::Result<Vec<u8>> {
    if !(1..=30).contains(&seconds) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tone duration must be 1..30 seconds",
        ));
    }
    const RATE: u32 = 48_000;
    let frames = RATE * u32::from(seconds);
    let data_len = frames * 4;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 4).to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..frames {
        let pulse = i % 38_400;
        let envelope = if pulse < 19_200 {
            (pulse.min(19_199 - pulse) as f64 / 480.0).min(1.0)
        } else {
            0.0
        };
        let sample = ((f64::from(i) * 900.0 * std::f64::consts::TAU / f64::from(RATE)).sin()
            * 3932.0
            * envelope) as i16;
        let channels = match side {
            Side::Left => [sample, 0],
            Side::Right => [0, sample],
        };
        for sample in channels {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
    }
    Ok(wav)
}
/// Cached Windows waveform output device IDs and names; never plays audio.
pub fn windows_output_devices() -> io::Result<Vec<(u32, String)>> {
    #[cfg(target_os = "windows")]
    {
        windows::devices()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "audio-devices lists Windows waveform outputs; use wpctl status for Linux sinks",
        ))
    }
}
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{
        fs::{self, OpenOptions},
        os::unix::fs::OpenOptionsExt,
        path::PathBuf,
        process::{Command as Process, Stdio},
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    pub(super) struct ToneFile(PathBuf);
    impl ToneFile {
        pub fn create(side: Side, seconds: u8) -> io::Result<Self> {
            let wav = tone_wav(side, seconds)?;
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos();
            for attempt in 0..16 {
                let path = std::env::temp_dir().join(format!(
                    "ugreen-finder-{}-{stamp}-{attempt}.wav",
                    std::process::id()
                ));
                match OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)
                {
                    Ok(mut file) => {
                        let tone = Self(path);
                        file.write_all(&wav)?;
                        return Ok(tone);
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(e),
                }
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "could not allocate finder tone file",
            ))
        }
    }
    impl Drop for ToneFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    pub(super) fn play(
        tone: &ToneFile,
        seconds: u8,
        target: &str,
        cancelled: &impl Fn() -> bool,
    ) -> io::Result<()> {
        if cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "finder cancelled before playback",
            ));
        }
        let child = Process::new("pw-play")
            .args([
                "--target",
                target,
                "--properties",
                "node.dont-fallback=true node.dont-reconnect=true node.dont-move=true",
            ])
            .arg(&tone.0)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        wait_player(child, seconds, cancelled)
    }
    fn wait_player(
        mut child: std::process::Child,
        seconds: u8,
        cancelled: &impl Fn() -> bool,
    ) -> io::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(u64::from(seconds) + 5);
        let result = loop {
            if cancelled() {
                break Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "finder playback cancelled",
                ));
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    break if status.success() {
                        Ok(())
                    } else {
                        Err(io::Error::other(format!("pw-play exited with {status}")))
                    }
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Ok(None) => {
                    break Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "finder audio playback exceeded deadline",
                    ))
                }
                Err(e) => break Err(e),
            }
        };
        if result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        result
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn cancellation_reaps_player_and_temp_file_is_private_and_removed() {
            use std::os::unix::fs::PermissionsExt;
            let tone = ToneFile::create(Side::Left, 1).unwrap();
            let path = tone.0.clone();
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            drop(tone);
            assert!(!path.exists());
            let child = Process::new("sleep").arg("10").spawn().unwrap();
            let start = Instant::now();
            assert_eq!(
                wait_player(child, 1, &|| true).unwrap_err().kind(),
                io::ErrorKind::Interrupted
            );
            assert!(start.elapsed() < Duration::from_secs(2));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[test]
    fn tone_has_valid_lengths_and_only_selected_channel() {
        for side in [Side::Left, Side::Right] {
            let wav = tone_wav(side, 1).unwrap();
            assert_eq!(wav.len(), 44 + 48_000 * 4);
            assert_eq!(&wav[..4], b"RIFF");
            assert_eq!(&wav[8..16], b"WAVEfmt ");
            assert_eq!(
                u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize,
                wav.len() - 8
            );
            assert_eq!(
                u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
                wav.len() - 44
            );
            let mut nonzero = false;
            for frame in wav[44..].as_chunks::<4>().0 {
                let left = i16::from_le_bytes(frame[..2].try_into().unwrap());
                let right = i16::from_le_bytes(frame[2..].try_into().unwrap());
                let (active, silent) = match side {
                    Side::Left => (left, right),
                    Side::Right => (right, left),
                };
                assert_eq!(silent, 0);
                assert!(active.abs() <= 3932);
                nonzero |= active != 0;
            }
            assert!(nonzero);
        }
        for seconds in [0, 31, 255] {
            assert!(tone_wav(Side::Left, seconds).is_err());
        }
    }
    #[test]
    fn stop_is_attempted_after_start_and_audio_failures() {
        for fail_start in [false, true] {
            let calls = RefCell::new(Vec::new());
            let result = run(
                |enabled| {
                    calls
                        .borrow_mut()
                        .push(if enabled { "start" } else { "stop" });
                    if enabled && fail_start {
                        Err(io::Error::other("start failed"))
                    } else {
                        Ok(())
                    }
                },
                || {
                    calls.borrow_mut().push("play");
                    Err(io::Error::other("audio failed"))
                },
            );
            assert!(result.is_err());
            assert_eq!(
                *calls.borrow(),
                if fail_start {
                    vec!["start", "stop"]
                } else {
                    vec!["start", "play", "stop"]
                }
            );
        }
        assert!(run(
            |enabled| if enabled {
                Ok(())
            } else {
                Err(io::Error::other("stop failed"))
            },
            || Ok(())
        )
        .is_err());
    }
    #[test]
    fn finder_gate_and_validation_precede_wire_and_audio() {
        let mut client = Client::new(
            std::io::Cursor::new(Vec::new()),
            std::time::Duration::from_secs(1),
        );
        assert_eq!(
            ring(
                &mut client,
                Side::Left,
                1,
                if cfg!(target_os = "windows") {
                    "0"
                } else {
                    "sink"
                }
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Unsupported
        );
        assert!(client.into_inner().into_inner().is_empty());
        for target in ["", "bad\nname"] {
            assert!(validate(1, target).is_err());
        }
        if cfg!(target_os = "windows") {
            assert!(validate(1, "0").is_ok());
            assert!(validate(1, "4294967295").is_err());
        } else {
            assert!(validate(1, "bluez_output.01_02_03_04_05_06.1").is_ok());
        }
    }
}
