use super::HostHeadset;
use std::io;
use std::time::Instant;

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "native RFCOMM is currently supported only on Windows and Linux",
    )
}

#[derive(Debug)]
pub(super) struct Connection;

impl Connection {
    pub(super) fn connect(_: [u8; 6], _: u8, _: Instant) -> io::Result<Self> {
        Err(unsupported())
    }
    pub(super) fn read(&mut self, _: &mut [u8], _: Instant) -> io::Result<usize> {
        Err(unsupported())
    }
    pub(super) fn write(&mut self, _: &[u8], _: Instant) -> io::Result<usize> {
        Err(unsupported())
    }
}

pub(super) fn list_devices() -> io::Result<Vec<HostHeadset>> {
    Err(unsupported())
}
