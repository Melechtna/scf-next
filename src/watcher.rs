use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::mpsc;
use tauri::Emitter;

pub fn start(dirs: &[std::path::PathBuf], handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[scf-next] Failed to create file watcher: {}", e);
            return;
        }
    };

    for dir in dirs {
        if !dir.is_dir() { continue; }
        if let Err(e) = watcher.watch(dir, RecursiveMode::Recursive) {
            eprintln!("[scf-next] Failed to watch directory {}: {}", dir.display(), e);
        }
    }

    for event in rx {
        match event {
            Ok(evt) => match evt.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    eprintln!("[scf-next] Controllers changed, notifying frontend");
                    let _ = handle.emit("controllers-changed", ());
                }
                _ => continue,
            },
            Err(e) => eprintln!("[scf-next] Watcher error: {}", e),
        }
    }
}
