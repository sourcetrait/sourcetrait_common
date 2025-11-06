use crate::*;

#[cfg(target_os = "linux")]
pub const PLATFORM: Platform<LinuxCrossPlatform> = Platform::new(LinuxCrossPlatform);
#[cfg(target_os = "macos")]
pub const PLATFORM: Platform<MacOsCrossPlatform> = Platform::new(MacOsCrossPlatform);
#[cfg(target_os = "windows")]
pub const PLATFORM: Platform<WindowsCrossPlatform> = Platform::new(WindowsCrossPlatform);
