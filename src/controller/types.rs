use std::collections::HashMap;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Clone, Serialize)]
pub struct ControllerDef {
    pub controller: super::ControllerInfo,
    #[serde(default)]
    pub defaults: super::ButtonDefaults,
    #[serde(default)]
    pub buttons: Vec<super::ButtonDef>,
    #[serde(default)]
    pub modes: Vec<super::ModeDef>,
    #[serde(default)]
    pub mode_overrides: HashMap<String, super::ModeOverride>,
}

#[derive(Deserialize)]
pub struct RawControllerDef {
    pub controller: super::ControllerInfo,
    #[serde(default)]
    pub defaults: super::ButtonDefaults,
    #[serde(default)]
    pub buttons: Vec<super::RawButtonDef>,
    #[serde(default)]
    pub modes: Vec<super::ModeDef>,
    #[serde(default)]
    pub mode_overrides: HashMap<String, super::ModeOverride>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControllerInfo {
    pub name: String,
    pub base: String,
    pub icon: String,
    #[serde(default = "default_hold_ms")]
    pub paint_hold_ms: u64,
    #[serde(default = "default_paint_color")]
    pub color: String,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub product: Option<String>,
}

fn default_hold_ms() -> u64 { 100 }
fn default_paint_color() -> String { "#00c8ff".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonDefaults {
    #[serde(default = "default_travel")]
    pub travel: f32,
    #[serde(default = "default_paint")]
    pub paint: String,
    #[serde(default = "default_transparency")]
    pub transparency: f32,
}

impl Default for ButtonDefaults {
    fn default() -> Self {
        Self {
            travel: default_travel(),
            paint: default_paint(),
            transparency: default_transparency(),
        }
    }
}

fn default_travel() -> f32 { 15.0 }
fn default_paint() -> String { "standard".into() }
fn default_transparency() -> f32 { 1.0 }

#[derive(Deserialize)]
pub struct RawButtonDef {
    pub id: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub image_up: Option<String>,
    #[serde(default)]
    pub image_down: Option<String>,
    #[serde(default)]
    pub input: Option<super::InputDef>,
    #[serde(default)]
    pub travel: Option<f32>,
    #[serde(default)]
    pub paint: Option<String>,
    #[serde(default)]
    pub transparency: Option<f32>,
    #[serde(default)]
    pub capsense: Option<u32>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub pad_size_x: Option<f32>,
    #[serde(default)]
    pub pad_size_y: Option<f32>,
    #[serde(default)]
    pub pad_angle: Option<f32>,
    #[serde(default)]
    pub pad_shape: Option<String>,
    #[serde(default)]
    pub pad_center_x: Option<f32>,
    #[serde(default)]
    pub pad_center_y: Option<f32>,
    #[serde(default)]
    pub pad_sensitivity: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ButtonDef {
    pub id: String,
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_up: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_down: Option<String>,
    pub input: super::InputDef,
    pub travel: f32,
    pub paint: String,
    pub transparency: f32,
    pub capsense: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_size_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_size_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_angle: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_center_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_center_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad_sensitivity: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum InputDef {
    #[serde(rename = "button")]
    Button { index: u32 },
    #[serde(rename = "trigger")]
    Trigger { axis: u32, threshold: Option<f32> },
    #[serde(rename = "stick")]
    Stick { axis_x: u32, axis_y: u32, press_button: Option<u32> },
    #[serde(rename = "touchpad")]
    Touchpad { index: u32, press_button: Option<u32> },
    #[serde(rename = "keyboard")]
    Keyboard { key: u16 },
    #[serde(rename = "mouse")]
    Mouse { button: u8 },
    #[serde(rename = "mouse_move")]
    MouseMove,
    #[serde(rename = "scroll_wheel")]
    ScrollWheel,
    #[serde(rename = "hat")]
    Hat { index: u32, direction: u32 },
    #[serde(rename = "multi")]
    Multi { indices: Vec<u32> },
    #[serde(rename = "none")]
    None,
}

impl Serialize for InputDef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        match self {
            InputDef::Button { index } => {
                let mut s = serializer.serialize_struct("InputDef", 2)?;
                s.serialize_field("type", "button")?;
                s.serialize_field("index", index)?;
                s.end()
            }
            InputDef::Trigger { axis, threshold } => {
                let mut s = serializer.serialize_struct("InputDef", 3)?;
                s.serialize_field("type", "trigger")?;
                s.serialize_field("axis", axis)?;
                s.serialize_field("threshold", threshold)?;
                s.end()
            }
            InputDef::Stick { axis_x, axis_y, press_button } => {
                let mut s = serializer.serialize_struct("InputDef", 4)?;
                s.serialize_field("type", "stick")?;
                s.serialize_field("axis_x", axis_x)?;
                s.serialize_field("axis_y", axis_y)?;
                s.serialize_field("press_button", press_button)?;
                s.end()
            }
            InputDef::Touchpad { index, press_button } => {
                let mut s = serializer.serialize_struct("InputDef", 3)?;
                s.serialize_field("type", "touchpad")?;
                s.serialize_field("index", index)?;
                s.serialize_field("press_button", press_button)?;
                s.end()
            }
            InputDef::Keyboard { key } => {
                let mut s = serializer.serialize_struct("InputDef", 2)?;
                s.serialize_field("type", "keyboard")?;
                s.serialize_field("key", key)?;
                s.end()
            }
            InputDef::Mouse { button } => {
                let mut s = serializer.serialize_struct("InputDef", 2)?;
                s.serialize_field("type", "mouse")?;
                s.serialize_field("button", button)?;
                s.end()
            }
            InputDef::MouseMove => {
                let mut s = serializer.serialize_struct("InputDef", 1)?;
                s.serialize_field("type", "mouse_move")?;
                s.end()
            }
            InputDef::ScrollWheel => {
                let mut s = serializer.serialize_struct("InputDef", 1)?;
                s.serialize_field("type", "scroll_wheel")?;
                s.end()
            }
            InputDef::Hat { index, direction } => {
                let mut s = serializer.serialize_struct("InputDef", 3)?;
                s.serialize_field("type", "hat")?;
                s.serialize_field("index", index)?;
                s.serialize_field("direction", direction)?;
                s.end()
            }
            InputDef::Multi { indices } => {
                let mut s = serializer.serialize_struct("InputDef", 2)?;
                s.serialize_field("type", "multi")?;
                s.serialize_field("indices", indices)?;
                s.end()
            }
            InputDef::None => {
                let mut s = serializer.serialize_struct("InputDef", 1)?;
                s.serialize_field("type", "none")?;
                s.end()
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModeDef {
    pub name: String,
    pub vendor: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct ModeOverride {
    pub buttons: HashMap<String, super::ButtonRemap>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct ButtonRemap {
    pub index: Option<u32>,
    pub axis: Option<u32>,
    pub image: Option<String>,
}
