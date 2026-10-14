//! Command handlers. Each owns one command's behavior,
//! independent of parsing and translation; the registry
//! routes invocations here.
pub mod analysis;
pub mod app;
pub mod device;
pub mod settings;

pub mod host;
