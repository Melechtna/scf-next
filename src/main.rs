#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd;
mod controller;
mod gamepad;
mod input;
mod sdl_bindings;
mod state;
mod watcher;

use std::sync::Mutex;

use state::{AppInner, SafeJoystickPtr, SafePtr};

fn main() {
    sdl_bindings::set_hint("SDL_JOYSTICK_HIDAPI", "1");
    sdl_bindings::set_hint("SDL_JOYSTICK_HIDAPI_STEAM", "1");

    unsafe {
        sdl_bindings::SDL_Init(sdl_bindings::SDL_INIT_GAMEPAD | sdl_bindings::SDL_INIT_JOYSTICK);
    }

    let controllers_base = controller::find_controllers_base();

    if controllers_base.len() > 1 {
        eprintln!("[scf-next] Controller search paths:");
        for p in &controllers_base {
            eprintln!("  {}", p.display());
        }
    }

    let gamepad_ptr = gamepad::util::open_first_gamepad();
    let joystick_ptr = if gamepad_ptr.is_null() {
        eprintln!("[scf-next] No gamepad found, trying joystick fallback…");
        gamepad::util::open_first_joystick()
    } else {
        std::ptr::null_mut()
    };

    if !gamepad_ptr.is_null() {
        eprintln!("[scf-next] Gamepad opened at startup");
    } else if !joystick_ptr.is_null() {
        eprintln!("[scf-next] Joystick opened at startup (fallback)");
    } else {
        eprintln!("[scf-next] No device found at startup");
    }

    let input_devices = input::InputDevices::open_all();

    tauri::Builder::default()
        .manage(Mutex::new(AppInner {
            gamepad: SafePtr(gamepad_ptr),
            joystick: SafeJoystickPtr(joystick_ptr),
            debug_gamepad: SafePtr(std::ptr::null_mut()),
            debug_joystick: SafeJoystickPtr(std::ptr::null_mut()),
            input_devices,
            controllers_base: controllers_base.clone(),
            loaded_controller: None,
            controller_hw_ids: Vec::new(),
        }))
        .setup(move |app| {
            let handle = app.handle().clone();
            let watch_dirs = controllers_base.clone();
            std::thread::spawn(move || {
                watcher::start(&watch_dirs, handle);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::list_controllers,
            cmd::list_groups,
            cmd::list_group_controllers,
            cmd::load_controller,
            cmd::get_button_image,
            cmd::get_base_image,
            cmd::poll_gamepad,
            cmd::get_loaded_controller_path,
            cmd::get_controller_def,
            cmd::list_gamepad_devices,
            cmd::open_debug_gamepad,
            cmd::close_debug_gamepad,
            cmd::poll_gamepad_debug,
            cmd::poll_debug_kbm,
            cmd::resize_window,
            cmd::persist_color,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run SCF Next");

    unsafe {
        sdl_bindings::SDL_Quit();
    }
}
