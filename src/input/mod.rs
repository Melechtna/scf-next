pub const KEY_CNT: usize = 768;
pub const BTN_CNT: usize = 32;
const MOUSE_SENSITIVITY: f32 = 0.005;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(not(target_os = "linux"))]
pub use windows::*;
