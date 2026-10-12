//! Host cleanup after an acknowledged factory reset. The caller must close
//! the RFCOMM session first; this module never sends a reset instruction.
use crate::{
    bluetooth::{self, BluetoothAddress},
    device::DeviceRegistry,
};
use std::io;

pub fn cleanup(address: &BluetoothAddress) -> io::Result<()> {
    cleanup_with(address, bluetooth::unpair, |address| {
        let mut registry = DeviceRegistry::load();
        registry.forget(address);
        registry.save_checked()
    })
}
fn cleanup_with(
    address: &BluetoothAddress,
    unpair: impl FnOnce(&BluetoothAddress) -> io::Result<()>,
    forget: impl FnOnce(&BluetoothAddress) -> io::Result<()>,
) -> io::Result<()> {
    unpair(address).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("headset reset acknowledged but host unpair failed: {e}; do not resend reset"),
        )
    })?;
    forget(address).map_err(|e| io::Error::new(e.kind(), format!("headset reset acknowledged and host unpaired but registry cleanup failed: {e}; do not resend reset")))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[test]
    fn cleanup_is_ordered_and_stops_on_unpair_error() {
        let address = BluetoothAddress::from_octets([1; 6]);
        let calls = RefCell::new(Vec::new());
        cleanup_with(
            &address,
            |a| {
                assert_eq!(a, &address);
                calls.borrow_mut().push("unpair");
                Ok(())
            },
            |a| {
                assert_eq!(a, &address);
                calls.borrow_mut().push("forget");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(*calls.borrow(), ["unpair", "forget"]);
        let error = cleanup_with(
            &address,
            |_| Err(io::Error::other("denied")),
            |_| panic!("must not forget after unpair failure"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("do not resend reset"));
        let error =
            cleanup_with(&address, |_| Ok(()), |_| Err(io::Error::other("disk full"))).unwrap_err();
        assert!(error
            .to_string()
            .contains("host unpaired but registry cleanup failed"));
    }
}
