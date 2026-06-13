use std::fs::OpenOptions;
use std::os::fd::OwnedFd;
use std::path::Path;

use super::{KEY_CNT, BTN_CNT, MOUSE_SENSITIVITY};
use evdev::{Device, KeyCode, RelativeAxisCode};

pub struct InputDevices {
    devices: Vec<Device>,
    device_is_mouse: Vec<bool>,
    keyboard_state: Vec<bool>,
    mouse_state: Vec<bool>,
    mouse_x: f32,
    mouse_y: f32,
    scroll_up: i32,
    scroll_down: i32,
    fallback_key_state: Vec<Vec<bool>>,
}

fn try_open_device(path: &Path) -> Option<Device> {
    let fd: OwnedFd = OpenOptions::new()
        .read(true)
        .open(path)
        .ok()?
        .into();

    Device::from_fd(fd).ok()
}

fn is_kbm_device(d: &Device) -> bool {
    let keys = match d.supported_keys() {
        Some(k) => k,
        None => return false,
    };
    let has_keyboard = keys.contains(KeyCode::KEY_A)
        || keys.contains(KeyCode::KEY_1);
    let has_mouse = keys.contains(KeyCode::BTN_LEFT)
        || keys.contains(KeyCode::BTN_RIGHT)
        || keys.contains(KeyCode::BTN_MIDDLE);
    has_keyboard || has_mouse
}

fn has_relative_axes(d: &Device) -> bool {
    d.supported_relative_axes().map_or(false, |axes| {
        axes.contains(RelativeAxisCode::REL_X)
    })
}

impl InputDevices {
    pub fn open_all() -> Self {
        let mut devices: Vec<Device> = Vec::new();
        let mut device_is_mouse: Vec<bool> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let mut paths = std::collections::HashSet::new();
        for (path, _) in evdev::enumerate() {
            paths.insert(path);
        }
        for i in 0..256 {
            let path = std::path::PathBuf::from(format!("/dev/input/event{}", i));
            paths.insert(path);
        }

        for path in &paths {
            if let Some(dev) = try_open_device(path) {
                let _ = dev.set_nonblocking(true);
                let name = match dev.name() {
                    Some(n) => n.to_string(),
                    None => String::new(),
                };
                if seen.insert(name) {
                    let is_mouse = has_relative_axes(&dev);
                    if is_kbm_device(&dev) || is_mouse {
                        device_is_mouse.push(is_mouse);
                        devices.push(dev);
                    }
                }
            }
        }

        let dev_count = devices.len();
        eprintln!("[scf-next] {} input devices active", dev_count);

        InputDevices {
            devices,
            device_is_mouse,
            keyboard_state: vec![false; KEY_CNT],
            mouse_state: vec![false; BTN_CNT],
            mouse_x: 0.0,
            mouse_y: 0.0,
            scroll_up: 0,
            scroll_down: 0,
            fallback_key_state: vec![vec![false; KEY_CNT]; dev_count],
        }
    }

    pub fn poll(&mut self) -> (&[bool], &[bool], f32, f32, i32, i32) {
        self.keyboard_state.fill(false);
        self.mouse_state.fill(false);
        self.scroll_up = 0;
        self.scroll_down = 0;

        let mut had_mouse = false;

        for (i, (dev, &is_mouse)) in self.devices.iter_mut().zip(self.device_is_mouse.iter()).enumerate() {
            if let Ok(events) = dev.fetch_events() {
                for event in events {
                    match event.destructure() {
                        evdev::EventSummary::Key(_, key, value) => {
                            let c = key.0 as usize;
                            if c < KEY_CNT {
                                self.fallback_key_state[i][c] = value == 1 || value == 2;
                            }
                        }
                        evdev::EventSummary::RelativeAxis(_, code, value) if is_mouse => {
                            if code == RelativeAxisCode::REL_X {
                                self.mouse_x += value as f32 * MOUSE_SENSITIVITY;
                                had_mouse = true;
                            } else if code == RelativeAxisCode::REL_Y {
                                self.mouse_y += value as f32 * MOUSE_SENSITIVITY;
                                had_mouse = true;
                            } else if code == RelativeAxisCode::REL_WHEEL {
                                if value > 0 {
                                    self.scroll_up += value;
                                } else {
                                    self.scroll_down += -value;
                                }
                            }
                        }
                        evdev::EventSummary::RelativeAxis(_, code, value) => {
                            if code == RelativeAxisCode::REL_WHEEL {
                                if value > 0 {
                                    self.scroll_up += value;
                                } else {
                                    self.scroll_down += -value;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Keyboard state from events only (EVIOCGKEY unreliable on some hardware)
            for (c, &pressed) in self.fallback_key_state[i].iter().enumerate() {
                if pressed {
                    if c >= 0x110 && c <= 0x112 {
                        self.mouse_state[c - 0x110] = true;
                    } else {
                        self.keyboard_state[c] = true;
                    }
                }
            }

            // Mouse buttons get an ioctl supplement (works reliably per testing)
            if let Ok(keys) = dev.get_key_state() {
                for code in keys.iter() {
                    let c = code.0 as usize;
                    if c < KEY_CNT && c >= 0x110 && c <= 0x112 {
                        self.mouse_state[c - 0x110] = true;
                    }
                }
            }
        }

        if !had_mouse {
            self.mouse_x *= 0.85;
            self.mouse_y *= 0.85;
            if self.mouse_x.abs() < 0.001 { self.mouse_x = 0.0; }
            if self.mouse_y.abs() < 0.001 { self.mouse_y = 0.0; }
        }

        let dist = (self.mouse_x * self.mouse_x + self.mouse_y * self.mouse_y).sqrt();
        if dist > 1.0 {
            self.mouse_x /= dist;
            self.mouse_y /= dist;
        }

        (&self.keyboard_state, &self.mouse_state, self.mouse_x, self.mouse_y, self.scroll_up, self.scroll_down)
    }
}
