//! Host Attach Layer (The Hell)
//!
//! Isolating host-specific quirks and window handle management.

pub mod clap_attach;
pub mod host_quirks;
pub mod standalone;
pub mod vst3_attach;

pub enum HostType {
    Cubase,
    Reaper,
    StudioOne,
    Logic,
    Ableton,
    Unknown,
}

pub struct HostContext {
    pub host: HostType,
    pub dpi_scale: f32,
    pub platform: Platform,
}

pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

/// Host Compatibility Matrix — Records, not religion.
pub struct CompatibilityMatrix {
    pub resize_works: bool,
    pub focus_stealing: bool,
    pub keyboard_transparency: bool,
}

impl CompatibilityMatrix {
    pub fn for_host(host: HostType, platform: Platform) -> Self {
        match (host, platform) {
            (HostType::Cubase, Platform::Windows) => Self {
                resize_works: true,
                focus_stealing: true, // Typical Cubase
                keyboard_transparency: false,
            },
            // Add more quirks as discovered
            _ => Self {
                resize_works: true,
                focus_stealing: false,
                keyboard_transparency: true,
            },
        }
    }
}
