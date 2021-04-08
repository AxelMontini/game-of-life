use std::error::Error;

pub(crate) const WIDTH: usize = 256;
pub(crate) const INTERVAL_SEC: f32 = 1.0 / 60.0;

#[cfg(feature = "cpucompute")]
pub mod cpu;
#[cfg(feature = "window")]
pub mod display;
#[cfg(feature = "gpucompute")]
pub mod gpu;
#[cfg(feature = "term")]
pub mod term;

#[cfg(all(feature = "cpucompute", feature = "gpucompute"))]
compile_error!(
    "feature \"cpucompute\" and feature \"gpucompute\" cannot be enabled at the same time"
);

#[cfg(all(feature = "term", feature = "window"))]
compile_error!("feature \"term\" and feature \"window\" cannot be enabled at the same time");

#[cfg(feature = "window")]
fn main() -> Result<(), Box<dyn Error>> {
    display::run()
}

#[cfg(feature = "term")]
fn main() -> Result<(), Box<dyn Error>> {
    term::run()
}
