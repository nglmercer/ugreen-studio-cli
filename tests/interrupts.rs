//! Exercise native signal handling in isolated processes, never the test runner.
#[cfg(target_os = "linux")]
#[test]
fn native_interrupt_probe() {
    let Ok(signal) = std::env::var("UGREEN_INTERRUPT_PROBE") else {
        return;
    };
    let signal: i32 = signal.parse().unwrap();
    let mut before: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: querying a valid signal with writable output in this isolated child.
    assert_eq!(
        unsafe { libc::sigaction(signal, std::ptr::null(), &mut before) },
        0
    );
    let guard = ugreen_cli::interrupt::Guard::install().unwrap();
    assert!(!guard.interrupted());
    assert!(ugreen_cli::interrupt::Guard::install().is_err());
    // SAFETY: the child has installed the handler for this signal; no I/O in handler.
    assert_eq!(unsafe { libc::raise(signal) }, 0);
    assert!(guard.interrupted());
    drop(guard);
    let mut after: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: query after guard destruction, without delivering another signal.
    assert_eq!(
        unsafe { libc::sigaction(signal, std::ptr::null(), &mut after) },
        0
    );
    assert_eq!(before.sa_sigaction, after.sa_sigaction);
    assert_eq!(
        before.sa_flags & libc::SA_RESTART,
        after.sa_flags & libc::SA_RESTART
    );
    for signal in 1..=64 {
        // SAFETY: compare valid initialized signal masks.
        assert_eq!(
            unsafe { libc::sigismember(&before.sa_mask, signal) },
            unsafe { libc::sigismember(&after.sa_mask, signal) }
        );
    }
    let guard = ugreen_cli::interrupt::Guard::install().unwrap();
    assert!(!guard.interrupted());
}
#[cfg(target_os = "linux")]
#[test]
fn sigint_and_sigterm_cancel_and_restore_in_child_processes() {
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "native_interrupt_probe", "--nocapture"])
            .env("UGREEN_INTERRUPT_PROBE", signal.to_string())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{:?}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
