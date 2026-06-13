use std::path::Path;

pub fn load_controller(path: &Path) -> Result<super::ControllerDef, String> {
    let toml_path = path.join("controller.toml");
    let content = std::fs::read_to_string(&toml_path)
        .map_err(|e| format!("Failed to read {}: {}", toml_path.display(), e))?;
    let raw: super::RawControllerDef = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", toml_path.display(), e))?;

    let d = &raw.defaults;

    let mut buttons: Vec<super::ButtonDef> = raw.buttons.iter().map(|b| {
        super::ButtonDef {
            id: b.id.clone(),
            image: b.image.clone().unwrap_or_default(),
            image_up: b.image_up.clone(),
            image_down: b.image_down.clone(),
            input: b.input.clone().unwrap_or(super::InputDef::None),
            travel: b.travel.unwrap_or(d.travel),
            paint: b.paint.clone().unwrap_or_else(|| d.paint.clone()),
            transparency: b.transparency.unwrap_or(d.transparency),
            capsense: b.capsense,
            reference: b.reference.clone(),
            pad_size_x: b.pad_size_x,
            pad_size_y: b.pad_size_y,
            pad_angle: b.pad_angle,
            pad_shape: b.pad_shape.clone(),
            pad_center_x: b.pad_center_x,
            pad_center_y: b.pad_center_y,
            pad_sensitivity: b.pad_sensitivity,
        }
    }).collect();

    {
        let id_to_idx: std::collections::HashMap<&str, usize> = buttons.iter()
            .enumerate()
            .map(|(i, b)| (b.id.as_str(), i))
            .collect();
        let pairs: Vec<(usize, usize)> = raw.buttons.iter().enumerate()
            .filter_map(|(i, raw_btn)| {
                raw_btn.reference.as_ref()
                    .and_then(|r| id_to_idx.get(r.as_str()).map(|&j| (i, j)))
            })
            .collect();
        for (i, j) in pairs {
            let ref_input = buttons[j].input.clone();
            let ref_travel = buttons[j].travel;
            let ref_image = buttons[j].image.clone();
            let ref_image_up = buttons[j].image_up.clone();
            let ref_image_down = buttons[j].image_down.clone();
            let ref_pad_size_x = buttons[j].pad_size_x;
            let ref_pad_size_y = buttons[j].pad_size_y;
            let ref_pad_angle = buttons[j].pad_angle;
            let ref_pad_shape = buttons[j].pad_shape.clone();
            let ref_pad_sensitivity = buttons[j].pad_sensitivity;
            let btn = &mut buttons[i];
            if raw.buttons[i].image.is_none() { btn.image = ref_image; }
            if raw.buttons[i].image_up.is_none() { btn.image_up = ref_image_up; }
            if raw.buttons[i].image_down.is_none() { btn.image_down = ref_image_down; }
            if raw.buttons[i].input.is_none() { btn.input = ref_input; }
            if raw.buttons[i].travel.is_none() { btn.travel = ref_travel; }
            if raw.buttons[i].pad_size_x.is_none() { btn.pad_size_x = ref_pad_size_x; }
            if raw.buttons[i].pad_size_y.is_none() { btn.pad_size_y = ref_pad_size_y; }
            if raw.buttons[i].pad_angle.is_none() { btn.pad_angle = ref_pad_angle; }
            if raw.buttons[i].pad_shape.is_none() { btn.pad_shape = ref_pad_shape; }
            if raw.buttons[i].pad_sensitivity.is_none() { btn.pad_sensitivity = ref_pad_sensitivity; }
        }
    }

    Ok(super::ControllerDef {
        controller: raw.controller,
        defaults: raw.defaults,
        buttons,
        modes: raw.modes,
        mode_overrides: raw.mode_overrides,
    })
}

pub fn save_controller_color(path: &Path, color: &str) -> Result<(), String> {
    let toml_path = path.join("controller.toml");
    let content = std::fs::read_to_string(&toml_path)
        .map_err(|e| format!("Failed to read {}: {}", toml_path.display(), e))?;
    let mut doc = content.parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Failed to parse {}: {}", toml_path.display(), e))?;
    doc["controller"]["color"] = toml_edit::value(color);
    std::fs::write(&toml_path, doc.to_string())
        .map_err(|e| format!("Failed to write {}: {}", toml_path.display(), e))?;
    Ok(())
}

pub fn detect_mode(def: &super::ControllerDef, vendor: u16, product: u16) -> Option<String> {
    let vendor_hex = format!("{:04x}", vendor);
    let product_hex = format!("{:04x}", product);
    for m in &def.modes {
        if let (Some(v), Some(p)) = (&m.vendor, &m.product) {
            if v == &vendor_hex && p == &product_hex {
                return Some(m.name.clone());
            }
        }
    }
    None
}

pub fn find_controllers_base() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let c = dir.join("controllers");
            if c.is_dir() { dirs.push(c); }
        }
    }

    if cfg!(target_os = "linux") {
        if let Some(cfg) = std::env::var_os("XDG_CONFIG_HOME") {
            let c = std::path::PathBuf::from(cfg).join("scf-next/controllers");
            if c.is_dir() { dirs.push(c); }
        } else if let Some(home) = std::env::var_os("HOME") {
            let c = std::path::PathBuf::from(home).join(".config/scf-next/controllers");
            if c.is_dir() { dirs.push(c); }
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            let c = std::path::PathBuf::from(home).join("Library/Application Support/scf-next/controllers");
            if c.is_dir() { dirs.push(c); }
        }
    } else if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("LOCALAPPDATA") {
            let c = std::path::PathBuf::from(appdata).join("scf-next/controllers");
            if c.is_dir() { dirs.push(c); }
        }
    }

    let c = std::env::current_dir().map(|d| d.join("controllers")).unwrap_or_default();
    if c.is_dir() && !dirs.contains(&c) { dirs.push(c); }

    if dirs.is_empty() {
        dirs.push(std::path::PathBuf::from("controllers"));
    }

    dirs
}
