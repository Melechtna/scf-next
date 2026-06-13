use crate::sdl_bindings::*;

pub(crate) fn detect_gamepad_button_count(gamepad: *mut SDL_Gamepad) -> u32 {
    unsafe {
        let mapping_ptr = SDL_GetGamepadMapping(gamepad);
        if mapping_ptr.is_null() {
            return 0;
        }
        let mapping = crate::sdl_bindings::sdl_str(mapping_ptr as *const i8);
        SDL_free(mapping_ptr as *mut std::ffi::c_void);

        const BUTTON_KEYS: &[(&str, u32)] = &[
            ("a", 0), ("b", 1), ("x", 2), ("y", 3),
            ("back", 4), ("guide", 5), ("start", 6),
            ("leftstick", 7), ("rightstick", 8),
            ("leftshoulder", 9), ("rightshoulder", 10),
            ("dpup", 11), ("dpdown", 12), ("dpleft", 13), ("dpright", 14),
            ("misc1", 15),
            ("paddle1", 16), ("paddle2", 17), ("paddle3", 18), ("paddle4", 19),
            ("touchpad", 20),
            ("misc2", 21), ("misc3", 22), ("misc4", 23), ("misc5", 24), ("misc6", 25),
        ];

        let mut max_idx = 0u32;
        for (key, idx) in BUTTON_KEYS {
            if mapping.contains(&format!(",{}:", key)) {
                max_idx = max_idx.max(*idx);
            }
        }

        if max_idx > 0 { max_idx + 1 } else { 0 }
    }
}
