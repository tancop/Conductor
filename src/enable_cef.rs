use cfg_if::cfg_if;
use std::fs::File;
use std::path::Path;

pub fn enable_cef_debugging() -> bool {
    let mut file_path = None;

    cfg_if! {
        if #[cfg(windows)] {
            file_path = Some(Path::new(r"C:\Program Files (x86)\Steam\.cef-enable-remote-debugging"));
        } else if #[cfg(target_os = "linux")] {
            file_path = Some(Path::new("~/.steam/steam/.cef-enable-remote-debugging"));
        } else if #[cfg(target_os = "macos")] {
            file_path = Some(Path::new("~/Library/Application Support/Steam/.cef-enable-remote-debugging"));
        }
    }

    let Some(file_path) = file_path else {
        return false;
    };

    if !file_path.exists() {
        if let Err(e) = File::create(file_path) {
            let path_string = file_path.to_string_lossy();
            log::error!("Failed to enable CEF debugging: {}", e);
            log::info!("Try creating an empty file manually at {}", path_string);
            return false;
        }

        log::warn!("CEF remote debugging was not enabled, restart Steam if it's running")
    }

    true
}
