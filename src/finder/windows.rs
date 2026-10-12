//! Explicit-device Windows waveform playback; WAVE_MAPPER is never used.
use super::*;
use std::{
    cell::UnsafeCell,
    ffi::c_void,
    mem::{size_of, zeroed},
    ptr::null_mut,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
#[repr(C, packed(1))]
struct Format {
    tag: u16,
    channels: u16,
    rate: u32,
    bytes_per_second: u32,
    alignment: u16,
    bits: u16,
    extra: u16,
}
#[repr(C)]
struct Header {
    data: *mut u8,
    length: u32,
    recorded: u32,
    user: usize,
    flags: u32,
    loops: u32,
    next: *mut Header,
    reserved: usize,
}
#[repr(C)]
struct Caps {
    manufacturer: u16,
    product: u16,
    version: u32,
    name: [u16; 32],
    formats: u32,
    channels: u16,
    reserved: u16,
    support: u32,
}
const _: () = {
    assert!(size_of::<Format>() == 18);
    assert!(size_of::<Header>() == if size_of::<usize>() == 8 { 48 } else { 32 });
    assert!(size_of::<Caps>() == 84);
};
#[link(name = "winmm")]
unsafe extern "system" {
    fn waveOutGetNumDevs() -> u32;
    fn waveOutGetDevCapsW(device: usize, caps: *mut Caps, size: u32) -> u32;
    fn waveOutOpen(
        handle: *mut *mut c_void,
        device: u32,
        format: *const Format,
        callback: usize,
        instance: usize,
        flags: u32,
    ) -> u32;
    fn waveOutPrepareHeader(handle: *mut c_void, header: *mut Header, size: u32) -> u32;
    fn waveOutWrite(handle: *mut c_void, header: *mut Header, size: u32) -> u32;
    fn waveOutReset(handle: *mut c_void) -> u32;
    fn waveOutUnprepareHeader(handle: *mut c_void, header: *mut Header, size: u32) -> u32;
    fn waveOutClose(handle: *mut c_void) -> u32;
}
fn check(code: u32, operation: &str) -> io::Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{operation} failed with multimedia error {code}"
        )))
    }
}
unsafe extern "system" fn callback(
    _: *mut c_void,
    message: u32,
    instance: usize,
    _: usize,
    _: usize,
) {
    if message == 0x3bd {
        // WOM_DONE
        // SAFETY: boxed completion flag remains allocated through buffer return/close.
        unsafe { &*(instance as *const AtomicBool) }.store(true, Ordering::Release);
    }
}
pub(super) fn devices() -> io::Result<Vec<(u32, String)>> {
    // SAFETY: this native query takes no pointers and initiates no playback.
    let count = unsafe { waveOutGetNumDevs() };
    if count > 4096 {
        return Err(io::Error::other("unexpected waveform device count"));
    }
    let mut devices = Vec::new();
    for id in 0..count {
        // SAFETY: this structure has a valid zero representation.
        let mut caps: Caps = unsafe { zeroed() };
        // SAFETY: writable SDK-layout structure, exact size, valid device index.
        check(
            unsafe { waveOutGetDevCapsW(id as usize, &mut caps, size_of::<Caps>() as u32) },
            "waveOutGetDevCapsW",
        )?;
        let len = caps
            .name
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(caps.name.len());
        devices.push((id, String::from_utf16_lossy(&caps.name[..len])));
    }
    Ok(devices)
}
struct Playback {
    handle: *mut c_void,
    header: Option<Box<UnsafeCell<Header>>>,
    samples: Option<Vec<u8>>,
    done: Option<Box<AtomicBool>>,
    prepared: bool,
}
impl Playback {
    fn close(&mut self) -> io::Result<()> {
        if self.handle.is_null() {
            return Ok(());
        }
        // SAFETY: exclusively owned open handle. Reset returns all pending buffers.
        check(unsafe { waveOutReset(self.handle) }, "waveOutReset")?;
        if self.prepared {
            let header = self.header.as_ref().expect("prepared header exists").get();
            // SAFETY: reset returned buffers; header remains pinned and correctly sized.
            check(
                unsafe { waveOutUnprepareHeader(self.handle, header, size_of::<Header>() as u32) },
                "waveOutUnprepareHeader",
            )?;
            self.prepared = false;
        }
        // SAFETY: all prepared buffers have been returned and unprepared.
        check(unsafe { waveOutClose(self.handle) }, "waveOutClose")?;
        self.handle = null_mut();
        Ok(())
    }
}
impl Drop for Playback {
    fn drop(&mut self) {
        if self.close().is_err() {
            // A failed driver cleanup may still retain these pointers. Retain
            // their storage rather than freeing memory still used by the driver.
            if let Some(header) = self.header.take() {
                std::mem::forget(header);
            }
            if let Some(samples) = self.samples.take() {
                std::mem::forget(samples);
            }
            if let Some(done) = self.done.take() {
                std::mem::forget(done);
            }
        }
    }
}
pub(super) fn play(
    side: Side,
    seconds: u8,
    target: &str,
    cancelled: &impl Fn() -> bool,
) -> io::Result<()> {
    let device: u32 = target.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows finder target must be a waveform output device index",
        )
    })?;
    // SAFETY: native device count query without side effects.
    if device >= unsafe { waveOutGetNumDevs() } {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "waveform output device index is unavailable",
        ));
    }
    let mut samples = tone_wav(side, seconds)?;
    if cancelled() {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "finder cancelled before playback",
        ));
    }
    let done = Box::new(AtomicBool::new(false));
    let header = Box::new(UnsafeCell::new(Header {
        data: samples[44..].as_mut_ptr(),
        length: (samples.len() - 44) as u32,
        recorded: 0,
        user: 0,
        flags: 0,
        loops: 0,
        next: null_mut(),
        reserved: 0,
    }));
    let format = Format {
        tag: 1,
        channels: 2,
        rate: 48_000,
        bytes_per_second: 192_000,
        alignment: 4,
        bits: 16,
        extra: 0,
    };
    let mut playback = Playback {
        handle: null_mut(),
        header: Some(header),
        samples: Some(samples),
        done: Some(done),
        prepared: false,
    };
    let done = playback.done.as_ref().unwrap();
    // SAFETY: valid SDK-layout format and handle output. Callback/completion
    // storage stay allocated until close. CALLBACK_FUNCTION is 0x30000.
    check(
        unsafe {
            waveOutOpen(
                &mut playback.handle,
                device,
                &format,
                callback as *const () as usize,
                &**done as *const AtomicBool as usize,
                0x30000,
            )
        },
        "waveOutOpen",
    )?;
    let header = playback.header.as_ref().unwrap().get();
    // SAFETY: header/sample memory is stable and all fields initialized.
    check(
        unsafe { waveOutPrepareHeader(playback.handle, header, size_of::<Header>() as u32) },
        "waveOutPrepareHeader",
    )?;
    playback.prepared = true;
    // SAFETY: header was successfully prepared and lives until driver return.
    check(
        unsafe { waveOutWrite(playback.handle, header, size_of::<Header>() as u32) },
        "waveOutWrite",
    )?;
    let deadline = Instant::now() + Duration::from_secs(u64::from(seconds) + 5);
    let result = loop {
        if cancelled() {
            break Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "finder playback cancelled",
            ));
        }
        if playback.done.as_ref().unwrap().load(Ordering::Acquire) {
            break Ok(());
        }
        if Instant::now() >= deadline {
            break Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "finder playback exceeded deadline",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let cleanup = playback.close();
    match (result, cleanup) {
        (Ok(()), result) => result,
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; audio cleanup failed: {cleanup}"),
        )),
    }
}
