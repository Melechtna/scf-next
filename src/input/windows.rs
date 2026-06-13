use super::{KEY_CNT, BTN_CNT, MOUSE_SENSITIVITY};
use std::sync::mpsc::{channel, Receiver};

pub struct InputDevices {
    keyboard_state: Vec<bool>,
    mouse_state: Vec<bool>,
    receiver: Receiver<rdev::Event>,
    _listener: std::thread::JoinHandle<()>,
    mouse_x: f32,
    mouse_y: f32,
    scroll_up: i32,
    scroll_down: i32,
    prev_mouse_x: f64,
    prev_mouse_y: f64,
    has_prev_mouse: bool,
}

fn key_to_evdev(key: &rdev::Key) -> Option<u16> {
    match key {
        rdev::Key::Alt => Some(56),
        rdev::Key::AltGr => Some(100),
        rdev::Key::Backspace => Some(14),
        rdev::Key::CapsLock => Some(58),
        rdev::Key::ControlLeft => Some(29),
        rdev::Key::ControlRight => Some(97),
        rdev::Key::Delete => Some(111),
        rdev::Key::DownArrow => Some(108),
        rdev::Key::End => Some(107),
        rdev::Key::Escape => Some(1),
        rdev::Key::F1 => Some(59),
        rdev::Key::F2 => Some(60),
        rdev::Key::F3 => Some(61),
        rdev::Key::F4 => Some(62),
        rdev::Key::F5 => Some(63),
        rdev::Key::F6 => Some(64),
        rdev::Key::F7 => Some(65),
        rdev::Key::F8 => Some(66),
        rdev::Key::F9 => Some(67),
        rdev::Key::F10 => Some(68),
        rdev::Key::F11 => Some(87),
        rdev::Key::F12 => Some(88),
        rdev::Key::Home => Some(102),
        rdev::Key::LeftArrow => Some(105),
        rdev::Key::MetaLeft => Some(125),
        rdev::Key::MetaRight => Some(126),
        rdev::Key::PageDown => Some(109),
        rdev::Key::PageUp => Some(104),
        rdev::Key::Return => Some(28),
        rdev::Key::RightArrow => Some(106),
        rdev::Key::ShiftLeft => Some(42),
        rdev::Key::ShiftRight => Some(54),
        rdev::Key::Space => Some(57),
        rdev::Key::Tab => Some(15),
        rdev::Key::UpArrow => Some(103),
        rdev::Key::PrintScreen => Some(120),
        rdev::Key::ScrollLock => Some(70),
        rdev::Key::Pause => Some(119),
        rdev::Key::NumLock => Some(69),
        rdev::Key::BackQuote => Some(41),
        rdev::Key::Num1 => Some(2),
        rdev::Key::Num2 => Some(3),
        rdev::Key::Num3 => Some(4),
        rdev::Key::Num4 => Some(5),
        rdev::Key::Num5 => Some(6),
        rdev::Key::Num6 => Some(7),
        rdev::Key::Num7 => Some(8),
        rdev::Key::Num8 => Some(9),
        rdev::Key::Num9 => Some(10),
        rdev::Key::Num0 => Some(11),
        rdev::Key::Minus => Some(12),
        rdev::Key::Equal => Some(13),
        rdev::Key::KeyQ => Some(16),
        rdev::Key::KeyW => Some(17),
        rdev::Key::KeyE => Some(18),
        rdev::Key::KeyR => Some(19),
        rdev::Key::KeyT => Some(20),
        rdev::Key::KeyY => Some(21),
        rdev::Key::KeyU => Some(22),
        rdev::Key::KeyI => Some(23),
        rdev::Key::KeyO => Some(24),
        rdev::Key::KeyP => Some(25),
        rdev::Key::LeftBracket => Some(26),
        rdev::Key::RightBracket => Some(27),
        rdev::Key::KeyA => Some(30),
        rdev::Key::KeyS => Some(31),
        rdev::Key::KeyD => Some(32),
        rdev::Key::KeyF => Some(33),
        rdev::Key::KeyG => Some(34),
        rdev::Key::KeyH => Some(35),
        rdev::Key::KeyJ => Some(36),
        rdev::Key::KeyK => Some(37),
        rdev::Key::KeyL => Some(38),
        rdev::Key::SemiColon => Some(39),
        rdev::Key::Quote => Some(40),
        rdev::Key::BackSlash => Some(43),
        rdev::Key::IntlBackslash => Some(86),
        rdev::Key::KeyZ => Some(44),
        rdev::Key::KeyX => Some(45),
        rdev::Key::KeyC => Some(46),
        rdev::Key::KeyV => Some(47),
        rdev::Key::KeyB => Some(48),
        rdev::Key::KeyN => Some(49),
        rdev::Key::KeyM => Some(50),
        rdev::Key::Comma => Some(51),
        rdev::Key::Dot => Some(52),
        rdev::Key::Slash => Some(53),
        rdev::Key::Insert => Some(110),
        rdev::Key::KpReturn => Some(96),
        rdev::Key::KpMinus => Some(74),
        rdev::Key::KpPlus => Some(78),
        rdev::Key::KpMultiply => Some(55),
        rdev::Key::KpDivide => Some(98),
        rdev::Key::Kp0 => Some(82),
        rdev::Key::Kp1 => Some(79),
        rdev::Key::Kp2 => Some(80),
        rdev::Key::Kp3 => Some(81),
        rdev::Key::Kp4 => Some(75),
        rdev::Key::Kp5 => Some(76),
        rdev::Key::Kp6 => Some(77),
        rdev::Key::Kp7 => Some(71),
        rdev::Key::Kp8 => Some(72),
        rdev::Key::Kp9 => Some(73),
        rdev::Key::KpDelete => Some(83),
        rdev::Key::Function => None,
        rdev::Key::Unknown(_) => None,
    }
}

fn button_to_idx(button: &rdev::Button) -> Option<usize> {
    match button {
        rdev::Button::Left => Some(0),
        rdev::Button::Right => Some(1),
        rdev::Button::Middle => Some(2),
        _ => None,
    }
}

impl InputDevices {
    pub fn open_all() -> Self {
        let (tx, rx) = channel();
        let handle = std::thread::spawn(move || {
            if let Err(e) = rdev::listen(move |event| {
                let _ = tx.send(event);
            }) {
                eprintln!("rdev listen error: {:?}", e);
            }
        });
        InputDevices {
            keyboard_state: vec![false; KEY_CNT],
            mouse_state: vec![false; BTN_CNT],
            receiver: rx,
            _listener: handle,
            mouse_x: 0.0,
            mouse_y: 0.0,
            scroll_up: 0,
            scroll_down: 0,
            prev_mouse_x: 0.0,
            prev_mouse_y: 0.0,
            has_prev_mouse: false,
        }
    }

    pub fn poll(&mut self) -> (&[bool], &[bool], f32, f32, i32, i32) {
        self.scroll_up = 0;
        self.scroll_down = 0;
        let mut had_mouse = false;
        while let Ok(event) = self.receiver.try_recv() {
            match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if let Some(code) = key_to_evdev(&key) {
                        if (code as usize) < KEY_CNT {
                            self.keyboard_state[code as usize] = true;
                        }
                    }
                }
                rdev::EventType::KeyRelease(key) => {
                    if let Some(code) = key_to_evdev(&key) {
                        if (code as usize) < KEY_CNT {
                            self.keyboard_state[code as usize] = false;
                        }
                    }
                }
                rdev::EventType::ButtonPress(button) => {
                    if let Some(idx) = button_to_idx(&button) {
                        self.mouse_state[idx] = true;
                    }
                }
                rdev::EventType::ButtonRelease(button) => {
                    if let Some(idx) = button_to_idx(&button) {
                        self.mouse_state[idx] = false;
                    }
                }
                rdev::EventType::MouseMove { x, y } => {
                    had_mouse = true;
                    if self.has_prev_mouse {
                        let dx = ((x - self.prev_mouse_x) as f32).clamp(-100.0, 100.0);
                        let dy = ((y - self.prev_mouse_y) as f32).clamp(-100.0, 100.0);
                        self.mouse_x += dx * MOUSE_SENSITIVITY;
                        self.mouse_y += dy * MOUSE_SENSITIVITY;
                    }
                    self.prev_mouse_x = x;
                    self.prev_mouse_y = y;
                    self.has_prev_mouse = true;
                }
                rdev::EventType::Wheel { delta_y, .. } => {
                    if delta_y > 0 {
                        self.scroll_up += delta_y as i32;
                    } else {
                        self.scroll_down += (-delta_y) as i32;
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
