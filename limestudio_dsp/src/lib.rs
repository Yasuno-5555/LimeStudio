// Initial empty lib.rs
pub mod wavelets;
pub mod convolution;
pub mod processor;
pub mod wrapper;
pub mod gatherer;
pub mod utils;
pub mod monitor;

pub fn version() -> &'static str {
    "0.1.0"
}
