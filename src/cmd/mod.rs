use serde::Serialize;
use std::sync::Mutex;

use crate::state::{AppInner, SafeJoystickPtr, SafePtr, LoadedDef};
use crate::gamepad::util::{open_matching_gamepad, open_matching_joystick, open_first_gamepad, open_first_joystick};

#[derive(Serialize)]
pub struct GamepadDeviceInfo {
    instance_id: i32,
    name: String,
    vendor: u16,
    product: u16,
    is_gamepad: bool,
}

#[tauri::command]
pub fn list_controllers(
    state: tauri::State<'_, Mutex<AppInner>>,
) -> Vec<crate::controller::ControllerEntry> {
    let inner = state.lock().unwrap();
    crate::controller::scan_controllers(&inner.controllers_base)
}

#[tauri::command]
pub fn list_groups(
    state: tauri::State<'_, Mutex<AppInner>>,
) -> Vec<crate::controller::GroupEntry> {
    let inner = state.lock().unwrap();
    crate::controller::scan_groups(&inner.controllers_base)
}

#[tauri::command]
pub fn list_group_controllers(
    path: String,
) -> Vec<crate::controller::ControllerEntry> {
    crate::controller::list_group_controllers(&std::path::PathBuf::from(&path))
}

#[tauri::command]
pub fn load_controller(
    path: String,
    webview_window: tauri::WebviewWindow,
    state: tauri::State<'_, Mutex<AppInner>>,
) -> Result<LoadedDef, String> {
    let dir = std::path::PathBuf::from(&path);
    let def = crate::controller::load_controller(&dir)?;

    let pairs: Vec<(String, String)> = {
        let from_controller = def.controller.vendor.as_ref()
            .zip(def.controller.product.as_ref());
        if let Some((v, p)) = from_controller {
            vec![(v.clone(), p.clone())]
        } else {
            let mut pairs: Vec<(String, String)> = def.modes.iter()
                .filter_map(|m| {
                    let v = m.vendor.as_ref()?;
                    let p = m.product.as_ref()?;
                    Some((v.clone(), p.clone()))
                })
                .collect();
            pairs.sort();
            pairs.dedup();
            pairs
        }
    };
    let has_ids = !pairs.is_empty();

    {
        let mut inner = state.lock().unwrap();
        if !inner.gamepad.as_ptr().is_null() {
            unsafe { crate::sdl_bindings::SDL_CloseGamepad(inner.gamepad.as_ptr()); }
            inner.gamepad = SafePtr(std::ptr::null_mut());
        }
        if !inner.joystick.as_ptr().is_null() {
            crate::gamepad::util::close_joystick(inner.joystick.as_ptr());
            inner.joystick = SafeJoystickPtr(std::ptr::null_mut());
        }
        inner.controller_hw_ids = pairs.clone();
    }

    let gp = if has_ids {
        let mut found = std::ptr::null_mut();
        for (v, p) in &pairs {
            found = open_matching_gamepad(v.as_str(), p.as_str());
            if !found.is_null() { break; }
        }
        found
    } else {
        open_first_gamepad()
    };
    if !gp.is_null() {
        eprintln!("[scf-next] Gamepad connected");
        let mut inner = state.lock().unwrap();
        inner.gamepad = SafePtr(gp);
    }

    let mode = if !gp.is_null() {
        unsafe {
            let vendor = crate::sdl_bindings::SDL_GetGamepadVendor(gp);
            let product = crate::sdl_bindings::SDL_GetGamepadProduct(gp);
            crate::controller::detect_mode(&def, vendor, product)
        }
    } else {
        let js = if has_ids {
            let mut found = std::ptr::null_mut();
            for (v, p) in &pairs {
                found = open_matching_joystick(v.as_str(), p.as_str());
                if !found.is_null() { break; }
            }
            found
        } else {
            open_first_joystick()
        };
        if !js.is_null() {
            eprintln!("[scf-next] Joystick opened for mode detection");
            let mut inner = state.lock().unwrap();
            inner.joystick = SafeJoystickPtr(js);
        }
        let js_ptr = js;
        if !js_ptr.is_null() {
            unsafe {
                let vendor = crate::sdl_bindings::SDL_GetJoystickVendor(js_ptr);
                let product = crate::sdl_bindings::SDL_GetJoystickProduct(js_ptr);
                crate::controller::detect_mode(&def, vendor, product)
            }
        } else {
            None
        }
    };

    let title = def.controller.name.clone();
    let _ = webview_window.set_title(&title);

    let mut inner = state.lock().unwrap();
    inner.loaded_controller = Some(crate::state::LoadedController {
        def: def.clone(),
        profile_dir: dir,
        mode: mode.clone(),
    });

    Ok(LoadedDef { color: def.controller.color.clone(), def, mode })
}

#[tauri::command]
pub fn get_controller_def(
    state: tauri::State<'_, Mutex<AppInner>>,
) -> Option<LoadedDef> {
    let inner = state.lock().unwrap();
    inner.loaded_controller.as_ref().map(|lc| LoadedDef {
        color: lc.def.controller.color.clone(),
        def: lc.def.clone(),
        mode: lc.mode.clone(),
    })
}

#[tauri::command]
pub fn get_button_image(path: String, image: String) -> Result<String, String> {
    let full_path = std::path::PathBuf::from(&path).join(&image);
    crate::controller::image_to_data_url(&full_path)
}

#[tauri::command]
pub fn get_base_image(path: String, image: String) -> Result<String, String> {
    let full_path = std::path::PathBuf::from(&path).join(&image);
    crate::controller::image_to_data_url(&full_path)
}

#[tauri::command]
pub fn poll_gamepad(state: tauri::State<'_, Mutex<AppInner>>) -> crate::gamepad::GamepadState {
    let mut inner = state.lock().unwrap();

    let gp = inner.gamepad.as_ptr();
    if !gp.is_null() && unsafe { !crate::sdl_bindings::SDL_GamepadConnected(gp) } {
        eprintln!("[scf-next] Gamepad lost, re-scanning");
        unsafe { crate::sdl_bindings::SDL_CloseGamepad(gp); }
        inner.gamepad = SafePtr(std::ptr::null_mut());
    }
    if inner.gamepad.as_ptr().is_null() {
        let ids = &inner.controller_hw_ids;
        let gp = if !ids.is_empty() {
            let mut found = std::ptr::null_mut();
            for (v, p) in ids {
                found = open_matching_gamepad(v.as_str(), p.as_str());
                if !found.is_null() { break; }
            }
            found
        } else {
            open_first_gamepad()
        };
        if !gp.is_null() {
            eprintln!("[scf-next] Gamepad connected");
            inner.gamepad = SafePtr(gp);
            crate::gamepad::util::close_joystick(inner.joystick.as_ptr());
            inner.joystick = SafeJoystickPtr(std::ptr::null_mut());
        }
    }
    if !inner.gamepad.as_ptr().is_null() {
        return crate::gamepad::GamepadState::poll(inner.gamepad.as_ptr());
    }

    let js = inner.joystick.as_ptr();
    if !js.is_null() && unsafe { !crate::sdl_bindings::SDL_JoystickConnected(js) } {
        eprintln!("[scf-next] Joystick lost, re-scanning");
        unsafe { crate::sdl_bindings::SDL_CloseJoystick(js); }
        inner.joystick = SafeJoystickPtr(std::ptr::null_mut());
    }
    if inner.joystick.as_ptr().is_null() {
        let ids = &inner.controller_hw_ids;
        let js = if !ids.is_empty() {
            let mut found = std::ptr::null_mut();
            for (v, p) in ids {
                found = open_matching_joystick(v.as_str(), p.as_str());
                if !found.is_null() { break; }
            }
            found
        } else {
            open_first_joystick()
        };
        if !js.is_null() {
            eprintln!("[scf-next] Joystick connected (fallback)");
            inner.joystick = SafeJoystickPtr(js);
        }
    }
    crate::gamepad::GamepadState::poll_joystick(inner.joystick.as_ptr())
}

#[tauri::command]
pub fn resize_window(width: f64, height: f64, webview_window: tauri::WebviewWindow) -> Result<(), String> {
    webview_window.set_size(tauri::LogicalSize { width, height })
        .map_err(|e| format!("set_size: {}", e))
}

#[tauri::command]
pub fn list_gamepad_devices() -> Vec<GamepadDeviceInfo> {
    unsafe {
        crate::sdl_bindings::SDL_PumpEvents();
        let mut count: i32 = 0;
        let ids = crate::sdl_bindings::SDL_GetJoysticks(&mut count);
        let mut devices = Vec::new();
        if !ids.is_null() {
            for i in 0..count {
                let id = *ids.add(i as usize);
                let ptr = crate::sdl_bindings::SDL_GetJoystickNameForID(id);
                let name = if ptr.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string()
                };
                let vendor = crate::sdl_bindings::SDL_GetJoystickVendorForID(id);
                let product = crate::sdl_bindings::SDL_GetJoystickProductForID(id);
                let is_gamepad = crate::sdl_bindings::SDL_IsGamepad(id);
                devices.push(GamepadDeviceInfo { instance_id: id, name, vendor, product, is_gamepad });
            }
            crate::sdl_bindings::SDL_free(ids as *mut std::ffi::c_void);
        }
        devices
    }
}

#[tauri::command]
pub fn get_loaded_controller_path(state: tauri::State<'_, Mutex<AppInner>>) -> Option<String> {
    let inner = state.lock().unwrap();
    inner.loaded_controller.as_ref().map(|lc| lc.profile_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_debug_gamepad(
    instance_id: i32,
    state: tauri::State<'_, Mutex<AppInner>>,
) -> Result<(), String> {
    let mut inner = state.lock().unwrap();
    if !inner.debug_gamepad.as_ptr().is_null() {
        unsafe { crate::sdl_bindings::SDL_CloseGamepad(inner.debug_gamepad.as_ptr()); }
        inner.debug_gamepad = SafePtr(std::ptr::null_mut());
    }
    if !inner.debug_joystick.as_ptr().is_null() {
        crate::gamepad::util::close_joystick(inner.debug_joystick.as_ptr());
        inner.debug_joystick = SafeJoystickPtr(std::ptr::null_mut());
    }
    unsafe {
        crate::sdl_bindings::SDL_PumpEvents();
        let gp = crate::sdl_bindings::SDL_OpenGamepad(instance_id);
        if !gp.is_null() {
            inner.debug_gamepad = SafePtr(gp);
            return Ok(());
        }
        let js = crate::sdl_bindings::SDL_OpenJoystick(instance_id);
        if !js.is_null() {
            inner.debug_joystick = SafeJoystickPtr(js);
            return Ok(());
        }
    }
    Err("Failed to open device".into())
}

#[tauri::command]
pub fn close_debug_gamepad(state: tauri::State<'_, Mutex<AppInner>>) {
    let mut inner = state.lock().unwrap();
    if !inner.debug_gamepad.as_ptr().is_null() {
        unsafe { crate::sdl_bindings::SDL_CloseGamepad(inner.debug_gamepad.as_ptr()); }
        inner.debug_gamepad = SafePtr(std::ptr::null_mut());
    }
    if !inner.debug_joystick.as_ptr().is_null() {
        crate::gamepad::util::close_joystick(inner.debug_joystick.as_ptr());
        inner.debug_joystick = SafeJoystickPtr(std::ptr::null_mut());
    }
}

#[tauri::command]
pub fn poll_gamepad_debug(state: tauri::State<'_, Mutex<AppInner>>) -> crate::gamepad::GamepadState {
    let mut inner = state.lock().unwrap();
    let gp = inner.debug_gamepad.as_ptr();
    if !gp.is_null() && unsafe { !crate::sdl_bindings::SDL_GamepadConnected(gp) } {
        unsafe { crate::sdl_bindings::SDL_CloseGamepad(gp); }
        inner.debug_gamepad = SafePtr(std::ptr::null_mut());
    }
    if !inner.debug_gamepad.as_ptr().is_null() {
        return crate::gamepad::GamepadState::poll(inner.debug_gamepad.as_ptr());
    }
    let js = inner.debug_joystick.as_ptr();
    if !js.is_null() && unsafe { !crate::sdl_bindings::SDL_JoystickConnected(js) } {
        crate::gamepad::util::close_joystick(js);
        inner.debug_joystick = SafeJoystickPtr(std::ptr::null_mut());
    }
    crate::gamepad::GamepadState::poll_joystick(inner.debug_joystick.as_ptr())
}

#[tauri::command]
pub fn poll_debug_kbm(
    state: tauri::State<'_, Mutex<AppInner>>,
) -> crate::gamepad::GamepadState {
    let mut inner = state.lock().unwrap();
    let (kb, ms, mx, my, su, sd) = inner.input_devices.poll();
    let mut gs = crate::gamepad::GamepadState::default();
    gs.connected = true;
    gs.name = "Keyboard & Mouse".into();
    crate::gamepad::fill_kbm(&mut gs, kb, ms, su, sd);
    gs.mouse = Some([mx, my]);
    gs
}

#[tauri::command]
pub fn persist_color(path: String, color: String) -> Result<(), String> {
    crate::controller::save_controller_color(&std::path::PathBuf::from(&path), &color)
}

