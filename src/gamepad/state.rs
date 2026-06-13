use super::TouchpadFinger;
use crate::sdl_bindings::*;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GamepadState {
    pub connected: bool,
    pub vendor: u16,
    pub product: u16,
    pub name: String,
    pub is_gamepad: bool,
    pub num_buttons: u32,
    pub num_axes: u32,
    pub buttons: [bool; 32],
    pub axes: [f32; 8],
    pub touchpads: Vec<TouchpadFinger>,
    pub cap_sense: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hats: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub keyboard: Vec<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mouse_buttons: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mouse: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll: Option<[i32; 2]>,
}

impl GamepadState {
    pub fn poll(gamepad: *mut SDL_Gamepad) -> Self {
        if gamepad.is_null() || unsafe { !SDL_GamepadConnected(gamepad) } {
            return Self::default();
        }

        unsafe {
            SDL_PumpEvents();
            SDL_UpdateGamepads();
        }

        let mut state = Self::default();
        state.connected = true;
        state.is_gamepad = true;

        unsafe {
            state.vendor = SDL_GetGamepadVendor(gamepad);
            state.product = SDL_GetGamepadProduct(gamepad);
            state.name = crate::sdl_bindings::sdl_str(SDL_GetGamepadName(gamepad));

            let js = SDL_GetGamepadJoystick(gamepad);
            state.num_buttons = SDL_GetNumJoystickButtons(js) as u32;
            state.num_axes = SDL_GetNumJoystickAxes(js) as u32;

            for i in 0..32 {
                state.buttons[i] = SDL_GetGamepadButton(gamepad, i as u32);
            }
            for i in (state.num_buttons as usize..32).rev() {
                if state.buttons[i] {
                    state.num_buttons = i as u32 + 1;
                    break;
                }
            }

            for i in 0..8 {
                let raw = SDL_GetGamepadAxis(gamepad, i as u32);
                state.axes[i] = (raw as f32) / 32767.0;
            }

            let num_hats = SDL_GetNumJoystickHats(js);
            if num_hats > 0 {
                state.hats = Vec::with_capacity(num_hats as usize);
                for h in 0..num_hats {
                    state.hats.push(SDL_GetJoystickHat(js, h));
                }
            }

            let num_tp = SDL_GetNumGamepadTouchpads(gamepad);
            if num_tp > 0 {
                state.touchpads = Vec::with_capacity(num_tp as usize);
                for tp in 0..num_tp {
                    let mut down = false;
                    let mut x = 0.5f32;
                    let mut y = 0.5f32;
                    let mut pressure = 0.0f32;
                    SDL_GetGamepadTouchpadFinger(gamepad, tp, 0, &mut down, &mut x, &mut y, &mut pressure);
                    state.touchpads.push(TouchpadFinger { down, x, y, pressure });
                }
            }

            let mut cs = [0.0f32; 4];
            for (i, ct) in [
                SDL_GamepadCapSenseType::LeftGrip,
                SDL_GamepadCapSenseType::RightGrip,
                SDL_GamepadCapSenseType::LeftStick,
                SDL_GamepadCapSenseType::RightStick,
            ].iter().enumerate() {
                if SDL_GamepadHasCapSense(gamepad, *ct) {
                    cs[i] = if SDL_GetGamepadCapSense(gamepad, *ct) { 1.0 } else { 0.0 };
                }
            }
            state.cap_sense = Some(cs);
        }

        state
    }

    pub fn poll_joystick(joystick: *mut SDL_Joystick) -> Self {
        if joystick.is_null() || unsafe { !SDL_JoystickConnected(joystick) } {
            return Self::default();
        }

        unsafe {
            SDL_PumpEvents();
            SDL_UpdateJoysticks();
        }

        let mut state = Self::default();
        state.connected = true;
        state.is_gamepad = false;

        unsafe {
            state.vendor = SDL_GetJoystickVendor(joystick);
            state.product = SDL_GetJoystickProduct(joystick);
            state.name = crate::sdl_bindings::sdl_str(SDL_GetJoystickName(joystick));
            state.num_buttons = SDL_GetNumJoystickButtons(joystick) as u32;
            state.num_axes = SDL_GetNumJoystickAxes(joystick) as u32;

            for i in 0..32.min(state.num_buttons as usize) {
                state.buttons[i] = SDL_GetJoystickButton(joystick, i as u32);
            }

            for i in 0..8.min(state.num_axes as usize) {
                let raw = SDL_GetJoystickAxis(joystick, i as u32);
                state.axes[i] = (raw as f32) / 32767.0;
            }

            let num_hats = SDL_GetNumJoystickHats(joystick);
            if num_hats > 0 {
                state.hats = Vec::with_capacity(num_hats as usize);
                for h in 0..num_hats {
                    state.hats.push(SDL_GetJoystickHat(joystick, h));
                }
            }
        }

        state
    }
}
