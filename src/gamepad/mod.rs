pub mod detect;
pub mod state;
pub mod util;

use serde::Serialize;

pub use state::GamepadState;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TouchpadFinger {
    pub down: bool,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

pub fn fill_kbm(state: &mut GamepadState, keyboard: &[bool], mouse: &[bool], scroll_up: i32, scroll_down: i32) {
    state.keyboard = keyboard.iter().enumerate()
        .filter(|(_, &p)| p)
        .map(|(i, _)| i as u16)
        .collect();
    state.mouse_buttons = mouse.iter().enumerate()
        .filter(|(_, &p)| p)
        .map(|(i, _)| i as u8)
        .collect();
    if scroll_up > 0 || scroll_down > 0 {
        state.scroll = Some([scroll_up, scroll_down]);
    }
}
