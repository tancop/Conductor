use cfg_if::cfg_if;
use std::fs::File;
use std::io::ErrorKind;
#[allow(unused_imports)] // only used on Windows
use std::path::Path;

pub fn enable_cef_debugging() -> Result<(), std::io::Error> {
    #[allow(unused_assignments)] // might be None on some platforms
    let mut file_path = None;

    cfg_if! {
        if #[cfg(windows)] {
            file_path = Some(Path::new(r"C:\Program Files (x86)\Steam\.cef-enable-remote-debugging"));
        } else if #[cfg(target_os = "linux")] {
            #[allow(deprecated)] // behavior fixed in rust 1.85, will be un-deprecated
            let Some(mut home_dir) = std::env::home_dir() else {
                return Err(std::io::Error::new(ErrorKind::NotFound, "Failed to get home directory"));
            };
            home_dir.push(".steam/steam/.cef-enable-remote-debugging");
            file_path = Some(home_dir.as_path());
        } else if #[cfg(target_os = "macos")] {
            #[allow(deprecated)]
            let Some(mut home_dir) = std::env::home_dir() else {
                return Err(std::io::Error::new(ErrorKind::NotFound, "Failed to get home directory"));
            };
            home_dir.push("Library/Application Support/Steam/.cef-enable-remote-debugging");
            file_path = Some(home_dir.as_path());
        }
    }

    let Some(file_path) = file_path else {
        return Err(std::io::Error::new(
            ErrorKind::Unsupported,
            "Steam is not supported on this platform",
        ));
    };

    if !file_path.exists() {
        if let Err(e) = File::create(file_path) {
            let path_string = file_path.to_string_lossy();
            log::error!(
                "Failed to enable CEF debugging. Try creating an empty file manually at {}",
                path_string
            );
            return Err(e);
        }

        log::warn!("CEF remote debugging was not enabled, restart Steam if it's running!");
    }

    Ok(())
}
