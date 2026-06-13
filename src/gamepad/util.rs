use crate::sdl_bindings::*;

fn format_id(id: u16) -> String {
    format!("{:04x}", id)
}

fn match_id(vendor: &str, product: &str, vid: u16, pid: u16) -> bool {
    vendor == format_id(vid) && product == format_id(pid)
}

pub fn open_matching_gamepad(vendor: &str, product: &str) -> *mut SDL_Gamepad {
    unsafe {
        SDL_PumpEvents();
        let mut count: Sint32 = 0;
        let ids = SDL_GetJoysticks(&mut count);
        let result = if count > 0 && !ids.is_null() {
            let mut found = std::ptr::null_mut();
            for i in 0..count {
                let id = *ids.add(i as usize);
                if !SDL_IsGamepad(id) { continue; }
                let vid = SDL_GetJoystickVendorForID(id);
                let pid = SDL_GetJoystickProductForID(id);
                if match_id(vendor, product, vid, pid) {
                    let gp = SDL_OpenGamepad(id);
                    if !gp.is_null() {
                        found = gp;
                        break;
                    }
                }
            }
            found
        } else {
            std::ptr::null_mut()
        };
        if !ids.is_null() {
            SDL_free(ids as *mut std::ffi::c_void);
        }
        result
    }
}

pub fn open_matching_joystick(vendor: &str, product: &str) -> *mut SDL_Joystick {
    unsafe {
        SDL_PumpEvents();
        let mut count: Sint32 = 0;
        let ids = SDL_GetJoysticks(&mut count);
        let result = if count > 0 && !ids.is_null() {
            let mut found = std::ptr::null_mut();
            for i in 0..count {
                let id = *ids.add(i as usize);
                let vid = SDL_GetJoystickVendorForID(id);
                let pid = SDL_GetJoystickProductForID(id);
                if match_id(vendor, product, vid, pid) {
                    let js = SDL_OpenJoystick(id);
                    if !js.is_null() {
                        found = js;
                        break;
                    }
                }
            }
            found
        } else {
            std::ptr::null_mut()
        };
        if !ids.is_null() {
            SDL_free(ids as *mut std::ffi::c_void);
        }
        result
    }
}

pub fn open_first_gamepad() -> *mut SDL_Gamepad {
    unsafe {
        SDL_PumpEvents();
        let mut count: Sint32 = 0;
        let ids = SDL_GetGamepads(&mut count);
        let result = if count > 0 && !ids.is_null() {
            let id = *ids;
            let gp = SDL_OpenGamepad(id);
            if !gp.is_null() {
                let detected = super::detect::detect_gamepad_button_count(gp);
                if detected > 0 {
                    eprintln!("[scf-next] Gamepad mapping reports {} buttons", detected);
                }
                gp
            } else {
                std::ptr::null_mut()
            }
        } else {
            std::ptr::null_mut()
        };
        if !ids.is_null() {
            SDL_free(ids as *mut std::ffi::c_void);
        }
        result
    }
}

pub fn open_first_joystick() -> *mut SDL_Joystick {
    unsafe {
        SDL_PumpEvents();
        let mut count: Sint32 = 0;
        let ids = SDL_GetJoysticks(&mut count);
        let result = if count > 0 && !ids.is_null() {
            let id = *ids;
            eprintln!("[scf-next] Trying to open joystick instance {} ({})",
                id, crate::sdl_bindings::sdl_str(SDL_GetJoystickNameForID(id)));
            let js = SDL_OpenJoystick(id);
            if !js.is_null() {
                eprintln!("[scf-next] Joystick opened: {} buttons, {} axes",
                    SDL_GetNumJoystickButtons(js), SDL_GetNumJoystickAxes(js));
                js
            } else {
                std::ptr::null_mut()
            }
        } else {
            std::ptr::null_mut()
        };
        if !ids.is_null() {
            SDL_free(ids as *mut std::ffi::c_void);
        }
        result
    }
}

pub fn close_joystick(joystick: *mut SDL_Joystick) {
    if !joystick.is_null() {
        unsafe { SDL_CloseJoystick(joystick) };
    }
}
