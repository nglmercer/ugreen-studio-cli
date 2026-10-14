//! Bounded Linux subprocess collection; every early return kills and reaps the child.
use std::{
    io::{self, Read},
    os::fd::{AsRawFd, RawFd},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
fn nonblocking(fd: RawFd) -> io::Result<()> {
    // SAFETY: the borrowed pipe fd remains valid during these calls.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
/// Always kill/reap a child if an early error interrupts output collection.
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

const MAX_COMMAND_OUTPUT: usize = 1024 * 1024;

fn drain_pipe(reader: &mut impl Read, output: &mut Vec<u8>) -> io::Result<bool> {
    let mut buffer = [0; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(count) => {
                if output.len() + count > MAX_COMMAND_OUTPUT {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "subprocess output exceeded 1 MiB",
                    ));
                }
                output.extend_from_slice(&buffer[..count]);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

pub(crate) fn collect_output(mut command: Command, timeout: Duration) -> io::Result<String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "could not run {program} (install your distribution's BlueZ tools): {error}"
                ),
            )
        })?;
    let mut child = ChildGuard(child);
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing subprocess stdout"))?;
    let mut stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("missing subprocess stderr"))?;
    nonblocking(stdout.as_raw_fd())?;
    nonblocking(stderr.as_raw_fd())?;
    let deadline = Instant::now() + timeout;
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut exit = None;
    loop {
        let out_done = drain_pipe(&mut stdout, &mut out)?;
        let err_done = drain_pipe(&mut stderr, &mut err)?;
        if exit.is_none() {
            exit = child.0.try_wait()?;
        }
        if let Some(status) = exit {
            if out_done && err_done {
                let stdout = std::str::from_utf8(&out)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let stderr = String::from_utf8_lossy(&err);
                if !status.success() {
                    return Err(io::Error::other(format!(
                        "{program} failed ({status}): {} {}",
                        stderr.trim(),
                        stdout.trim()
                    )));
                }
                if !stderr.trim().is_empty() {
                    return Err(io::Error::other(format!(
                        "{program} reported: {}",
                        stderr.trim()
                    )));
                }
                return Ok(stdout.to_owned());
            }
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("{program} timed out"),
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dump_timeout_and_output_limit_reap_child() {
        let path =
            std::env::temp_dir().join(format!("ugreen-process-test-{}.pid", std::process::id()));
        let mut command = Command::new("sh");
        command
            .args(["-c", "echo $$ > \"$1\"; exec sleep 30", "ugreen-test"])
            .arg(&path);
        assert_eq!(
            collect_output(command, Duration::from_millis(100))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        let pid: i32 = std::fs::read_to_string(&path)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        // SAFETY: signal zero only probes existence; it does not send a signal.
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
        std::fs::remove_file(path).unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "head -c 1048577 /dev/zero"]);
        assert_eq!(
            collect_output(command, Duration::from_secs(2))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
