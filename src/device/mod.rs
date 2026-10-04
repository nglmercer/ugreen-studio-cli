//! Persistent device state: the last selected headset and its
//! confirmed model, remembered across runs.

pub mod registry;

pub use registry::{DeviceEntry, DeviceRegistry};
