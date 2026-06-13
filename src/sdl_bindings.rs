#![allow(dead_code, non_camel_case_types)]

use std::ffi::CStr;

pub type Uint32 = u32;
pub type Uint16 = u16;
pub type Uint8 = u8;
pub type Sint16 = i16;
pub type Sint32 = i32;
pub type Uint64 = u64;
pub type SDL_JoystickID = Sint32;

#[repr(C)]
pub struct SDL_Gamepad([u8; 0]);

#[repr(C)]
pub struct SDL_Joystick([u8; 0]);

pub const SDL_INIT_GAMEPAD: Uint32 = 0x00002000;
pub const SDL_INIT_JOYSTICK: Uint32 = 0x00000200;

pub const SDL_GAMEPAD_BUTTON_SOUTH: u32 = 0;
pub const SDL_GAMEPAD_BUTTON_EAST: u32 = 1;
pub const SDL_GAMEPAD_BUTTON_WEST: u32 = 2;
pub const SDL_GAMEPAD_BUTTON_NORTH: u32 = 3;
pub const SDL_GAMEPAD_BUTTON_BACK: u32 = 4;
pub const SDL_GAMEPAD_BUTTON_GUIDE: u32 = 5;
pub const SDL_GAMEPAD_BUTTON_START: u32 = 6;
pub const SDL_GAMEPAD_BUTTON_LEFT_STICK: u32 = 7;
pub const SDL_GAMEPAD_BUTTON_RIGHT_STICK: u32 = 8;
pub const SDL_GAMEPAD_BUTTON_LEFT_SHOULDER: u32 = 9;
pub const SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER: u32 = 10;
pub const SDL_GAMEPAD_BUTTON_DPAD_UP: u32 = 11;
pub const SDL_GAMEPAD_BUTTON_DPAD_DOWN: u32 = 12;
pub const SDL_GAMEPAD_BUTTON_DPAD_LEFT: u32 = 13;
pub const SDL_GAMEPAD_BUTTON_DPAD_RIGHT: u32 = 14;
pub const SDL_GAMEPAD_BUTTON_MISC1: u32 = 15;
pub const SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1: u32 = 16;
pub const SDL_GAMEPAD_BUTTON_LEFT_PADDLE1: u32 = 17;
pub const SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2: u32 = 18;
pub const SDL_GAMEPAD_BUTTON_LEFT_PADDLE2: u32 = 19;
pub const SDL_GAMEPAD_BUTTON_TOUCHPAD: u32 = 20;
pub const SDL_GAMEPAD_BUTTON_MISC2: u32 = 21;
pub const SDL_GAMEPAD_BUTTON_MISC3: u32 = 22;
pub const SDL_GAMEPAD_BUTTON_MISC4: u32 = 23;
pub const SDL_GAMEPAD_BUTTON_MISC5: u32 = 24;
pub const SDL_GAMEPAD_BUTTON_MISC6: u32 = 25;

pub const SDL_GAMEPAD_AXIS_LEFTX: u32 = 0;
pub const SDL_GAMEPAD_AXIS_LEFTY: u32 = 1;
pub const SDL_GAMEPAD_AXIS_RIGHTX: u32 = 2;
pub const SDL_GAMEPAD_AXIS_RIGHTY: u32 = 3;
pub const SDL_GAMEPAD_AXIS_LEFT_TRIGGER: u32 = 4;
pub const SDL_GAMEPAD_AXIS_RIGHT_TRIGGER: u32 = 5;

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum SDL_GamepadCapSenseType {
    Invalid = -1,
    LeftStick = 0,
    RightStick = 1,
    LeftGrip = 2,
    RightGrip = 3,
}

#[link(name = "SDL3")]
unsafe extern "C" {
    pub fn SDL_SetHint(name: *const i8, value: *const i8) -> bool;
    pub fn SDL_Init(flags: Uint32) -> Sint32;
    pub fn SDL_Quit();
    pub fn SDL_GetGamepads(count: *mut Sint32) -> *mut SDL_JoystickID;
    pub fn SDL_IsGamepad(instance_id: SDL_JoystickID) -> bool;
    pub fn SDL_OpenGamepad(instance_id: SDL_JoystickID) -> *mut SDL_Gamepad;
    pub fn SDL_CloseGamepad(gamepad: *mut SDL_Gamepad);
    pub fn SDL_GamepadConnected(gamepad: *mut SDL_Gamepad) -> bool;
    pub fn SDL_PumpEvents();
    pub fn SDL_UpdateGamepads();
    pub fn SDL_GetGamepadVendor(gamepad: *mut SDL_Gamepad) -> Uint16;
    pub fn SDL_GetGamepadProduct(gamepad: *mut SDL_Gamepad) -> Uint16;
    pub fn SDL_GetGamepadName(gamepad: *mut SDL_Gamepad) -> *const i8;
    pub fn SDL_GetGamepadButton(gamepad: *mut SDL_Gamepad, button: Uint32) -> bool;
    pub fn SDL_GetGamepadAxis(gamepad: *mut SDL_Gamepad, axis: Uint32) -> Sint16;
    pub fn SDL_GetNumGamepadTouchpads(gamepad: *mut SDL_Gamepad) -> Sint32;
    pub fn SDL_GetNumGamepadTouchpadFingers(gamepad: *mut SDL_Gamepad, touchpad: Sint32) -> Sint32;
    pub fn SDL_GetGamepadJoystick(gamepad: *mut SDL_Gamepad) -> *mut SDL_Joystick;
    pub fn SDL_GamepadHasCapSense(gamepad: *mut SDL_Gamepad, type_: SDL_GamepadCapSenseType) -> bool;
    pub fn SDL_GetGamepadCapSense(gamepad: *mut SDL_Gamepad, type_: SDL_GamepadCapSenseType) -> bool;
    pub fn SDL_GetGamepadTouchpadFinger(
        gamepad: *mut SDL_Gamepad,
        touchpad: Sint32,
        finger: Sint32,
        down: *mut bool,
        x: *mut f32,
        y: *mut f32,
        pressure: *mut f32,
    ) -> bool;
    pub fn SDL_GetGamepadType(gamepad: *mut SDL_Gamepad) -> Uint16;
    pub fn SDL_GetJoystickNameForID(instance_id: SDL_JoystickID) -> *const i8;
    pub fn SDL_GetJoysticks(count: *mut Sint32) -> *mut SDL_JoystickID;
    pub fn SDL_GetJoystickVendorForID(instance_id: SDL_JoystickID) -> Uint16;
    pub fn SDL_GetJoystickProductForID(instance_id: SDL_JoystickID) -> Uint16;
    pub fn SDL_OpenJoystick(instance_id: SDL_JoystickID) -> *mut SDL_Joystick;
    pub fn SDL_CloseJoystick(joystick: *mut SDL_Joystick);
    pub fn SDL_JoystickConnected(joystick: *mut SDL_Joystick) -> bool;
    pub fn SDL_GetJoystickName(joystick: *mut SDL_Joystick) -> *const i8;
    pub fn SDL_GetJoystickVendor(joystick: *mut SDL_Joystick) -> Uint16;
    pub fn SDL_GetJoystickProduct(joystick: *mut SDL_Joystick) -> Uint16;
    pub fn SDL_GetNumJoystickButtons(joystick: *mut SDL_Joystick) -> Sint32;
    pub fn SDL_GetNumJoystickAxes(joystick: *mut SDL_Joystick) -> Sint32;
    pub fn SDL_GetNumJoystickHats(joystick: *mut SDL_Joystick) -> Sint32;
    pub fn SDL_GetJoystickHat(joystick: *mut SDL_Joystick, hat: Sint32) -> Uint8;
    pub fn SDL_GetJoystickButton(joystick: *mut SDL_Joystick, button: Uint32) -> bool;
    pub fn SDL_GetJoystickAxis(joystick: *mut SDL_Joystick, axis: Uint32) -> Sint16;
    pub fn SDL_GetGamepadMapping(gamepad: *mut SDL_Gamepad) -> *mut i8;
    pub fn SDL_UpdateJoysticks();
    pub fn SDL_free(ptr: *mut std::ffi::c_void);
}

pub fn set_hint(name: &str, value: &str) -> bool {
    let c_name = std::ffi::CString::new(name).unwrap();
    let c_val = std::ffi::CString::new(value).unwrap();
    unsafe { SDL_SetHint(c_name.as_ptr(), c_val.as_ptr()) }
}

pub fn sdl_str(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}
