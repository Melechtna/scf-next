use std::path::Path;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ControllerEntry {
    pub name: String,
    pub path: String,
    pub icon_data_url: Option<String>,
    #[serde(default)]
    pub is_controller: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupEntry {
    pub name: String,
    pub path: String,
    pub icon_data_url: Option<String>,
}

pub fn scan_controllers(bases: &[std::path::PathBuf]) -> Vec<ControllerEntry> {
    let mut entries = Vec::new();
    for base in bases {
        if !base.is_dir() { continue; }
        let Ok(rd) = std::fs::read_dir(base) else { continue };
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            collect_controllers(&path, &mut entries);
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries.dedup_by(|a, b| a.path == b.path);
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

fn collect_controllers(dir: &Path, entries: &mut Vec<ControllerEntry>) {
    if dir.join("controller.toml").exists() {
        if let Some(entry) = make_controller_entry(dir) {
            entries.push(entry);
        }
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_controllers(&path, entries);
        }
    }
}

fn make_controller_entry(dir: &Path) -> Option<ControllerEntry> {
    let icon_path = dir.join("images/Icon.webp");
    let icon_data = icon_path.exists()
        .then(|| crate::controller::image_to_data_url(&icon_path).ok())
        .flatten();
    crate::controller::load_controller(dir).ok().map(|def| ControllerEntry {
        name: def.controller.name,
        path: dir.to_string_lossy().to_string(),
        icon_data_url: icon_data,
        is_controller: true,
    })
}

pub fn scan_groups(bases: &[std::path::PathBuf]) -> Vec<GroupEntry> {
    let mut groups = Vec::new();
    for base in bases {
        if !base.is_dir() { continue; }
        let Ok(rd) = std::fs::read_dir(base) else { continue };
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            if path.join("controller.toml").exists() {
                let icon_path = path.join("images/Icon.webp");
                let icon_data = icon_path.exists()
                    .then(|| crate::controller::image_to_data_url(&icon_path).ok())
                    .flatten();
                if let Ok(def) = crate::controller::load_controller(&path) {
                    groups.push(GroupEntry {
                        name: def.controller.name,
                        path: path.to_string_lossy().to_string(),
                        icon_data_url: icon_data,
                    });
                }
            } else if has_controller_descendant(&path) {
                let icon_path = path.join("Icon.webp");
                let icon_data = icon_path.exists()
                    .then(|| crate::controller::image_to_data_url(&icon_path).ok())
                    .flatten();
                let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                groups.push(GroupEntry {
                    name,
                    path: path.to_string_lossy().to_string(),
                    icon_data_url: icon_data,
                });
            }
        }
    }
    // Dedup by path
    groups.sort_by(|a, b| a.path.cmp(&b.path));
    groups.dedup_by(|a, b| a.path == b.path);
    groups.sort_by(|a, b| a.name.cmp(&b.name));
    groups
}

fn has_controller_descendant(dir: &Path) -> bool {
    if dir.join("controller.toml").exists() {
        return true;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return false };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() && has_controller_descendant(&path) {
            return true;
        }
    }
    false
}

pub fn list_group_controllers(group_path: &Path) -> Vec<ControllerEntry> {
    let mut entries = Vec::new();
    if !group_path.is_dir() {
        return entries;
    }
    if let Some(entry) = make_controller_entry(group_path) {
        entries.push(entry);
    }
    let Ok(rd) = std::fs::read_dir(group_path) else { return entries };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let dir_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        if path.join("controller.toml").exists() {
            if crate::controller::load_controller(&path).is_ok() {
                let icon_path = path.join("images/Icon.webp");
                let icon_data = icon_path.exists()
                    .then(|| crate::controller::image_to_data_url(&icon_path).ok())
                    .flatten();
                entries.push(ControllerEntry {
                    name: dir_name,
                    path: path.to_string_lossy().to_string(),
                    icon_data_url: icon_data,
                    is_controller: true,
                });
            }
        } else if has_controller_descendant(&path) {
            let icon_path = path.join("Icon.webp");
            let icon_data = icon_path.exists()
                .then(|| crate::controller::image_to_data_url(&icon_path).ok())
                .flatten();
            entries.push(ControllerEntry {
                name: dir_name,
                path: path.to_string_lossy().to_string(),
                icon_data_url: icon_data,
                is_controller: false,
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}
