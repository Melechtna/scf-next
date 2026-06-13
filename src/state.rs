use serde::Serialize;

pub struct SafePtr(pub *mut crate::sdl_bindings::SDL_Gamepad);

unsafe impl Send for SafePtr {}
unsafe impl Sync for SafePtr {}

impl SafePtr {
    pub fn as_ptr(&self) -> *mut crate::sdl_bindings::SDL_Gamepad {
        self.0
    }
}

pub struct SafeJoystickPtr(pub *mut crate::sdl_bindings::SDL_Joystick);

unsafe impl Send for SafeJoystickPtr {}
unsafe impl Sync for SafeJoystickPtr {}

impl SafeJoystickPtr {
    pub fn as_ptr(&self) -> *mut crate::sdl_bindings::SDL_Joystick {
        self.0
    }
}

pub struct AppInner {
    pub gamepad: SafePtr,
    pub joystick: SafeJoystickPtr,
    pub debug_gamepad: SafePtr,
    pub debug_joystick: SafeJoystickPtr,
    pub input_devices: crate::input::InputDevices,
    pub controllers_base: Vec<std::path::PathBuf>,
    pub loaded_controller: Option<LoadedController>,
    pub controller_hw_ids: Vec<(String, String)>,
}

pub struct LoadedController {
    pub def: crate::controller::ControllerDef,
    pub profile_dir: std::path::PathBuf,
    pub mode: Option<String>,
}

#[derive(Serialize)]
pub struct LoadedDef {
    pub def: crate::controller::ControllerDef,
    pub mode: Option<String>,
    pub color: String,
}
