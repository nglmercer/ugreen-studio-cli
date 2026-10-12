//! Scoped finder cancellation handlers. Handler callbacks only set a flag;
//! audio termination and headset stop run on the normal execution thread.
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    },
};
static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static OWNER: Mutex<()> = Mutex::new(());

/// CLI-only ownership of process interruption handlers for one finder run.
/// Embedders can instead pass their own cancellation callback to the finder.
pub struct Guard {
    _owner: MutexGuard<'static, ()>,
    #[cfg(target_os = "linux")]
    previous: [libc::sigaction; 2],
}
impl Guard {
    pub fn install() -> io::Result<Self> {
        let owner = OWNER.try_lock().map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "finder interrupt handlers already owned",
            )
        })?;
        INTERRUPTED.store(false, Ordering::Relaxed);
        #[cfg(target_os = "linux")]
        {
            // SAFETY: sigaction consists of scalar fields and a signal mask;
            // zero initialization is valid before sigemptyset populates it.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            // SAFETY: the mask is aligned and writable for this synchronous call.
            if unsafe { libc::sigemptyset(&mut action.sa_mask) } != 0 {
                return Err(io::Error::last_os_error());
            }
            action.sa_sigaction = handle as *const () as usize;
            action.sa_flags = libc::SA_RESTART;
            // SAFETY: both output structures have valid zero representations.
            let mut previous: [libc::sigaction; 2] = unsafe { std::mem::zeroed() };
            for (i, signal) in [libc::SIGINT, libc::SIGTERM].into_iter().enumerate() {
                // SAFETY: valid signal numbers, initialized action and writable old action.
                if unsafe { libc::sigaction(signal, &action, &mut previous[i]) } != 0 {
                    let error = io::Error::last_os_error();
                    if i == 1 {
                        // SAFETY: restore the exact action returned by the successful first call.
                        unsafe {
                            libc::sigaction(libc::SIGINT, &previous[0], std::ptr::null_mut());
                        }
                    }
                    return Err(error);
                }
            }
            Ok(Self {
                _owner: owner,
                previous,
            })
        }
        #[cfg(target_os = "windows")]
        {
            // SAFETY: callback has the documented signature and static lifetime.
            if unsafe { SetConsoleCtrlHandler(Some(handle), 1) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { _owner: owner })
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = owner;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "finder interruption handling requires Linux or Windows",
            ))
        }
    }
    pub fn interrupted(&self) -> bool {
        INTERRUPTED.load(Ordering::Relaxed)
    }
}
#[cfg(target_os = "linux")]
extern "C" fn handle(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}
#[cfg(target_os = "windows")]
unsafe extern "system" fn handle(event: u32) -> i32 {
    if matches!(event, 0 | 1) {
        INTERRUPTED.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}
#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(
        handler: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
}
impl Drop for Guard {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        for (signal, previous) in [libc::SIGINT, libc::SIGTERM]
            .into_iter()
            .zip(&self.previous)
        {
            // SAFETY: restore previously obtained actions while owning registration.
            unsafe {
                libc::sigaction(signal, previous, std::ptr::null_mut());
            }
        }
        #[cfg(target_os = "windows")]
        // SAFETY: remove the same static callback registered by this guard.
        unsafe {
            SetConsoleCtrlHandler(Some(handle), 0);
        }
    }
}
