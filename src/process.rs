use cfg_if::cfg_if;
use std::fs::OpenOptions;
use std::io::Write;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum StorePidError {
    #[error("Failed to find project folder")]
    NoProjectDir,
    #[error("Failed to open PID file: {0}")]
    NotOpened(std::io::Error),
    #[error("Failed to write PID to file: {0}")]
    WriteFailed(std::io::Error),
}

pub fn store_pid() -> Result<(), StorePidError> {
    let id = std::process::id();

    let Some(project_dirs) = directories::ProjectDirs::from("io.github", "tancop", "Conductor")
    else {
        return Err(StorePidError::NoProjectDir);
    };

    let cache_dir = project_dirs.cache_dir();

    if let Err(e) = std::fs::create_dir_all(cache_dir) {
        return Err(StorePidError::NotOpened(e));
    }

    match OpenOptions::new()
        .write(true)
        .create(true)
        .open(cache_dir.join("conductor.pid"))
    {
        Ok(mut pid_file) => match pid_file.write_all(id.to_string().as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(StorePidError::WriteFailed(e)),
        },
        Err(e) => Err(StorePidError::NotOpened(e)),
    }
}

#[derive(Debug, Error)]
pub enum KillError {
    #[error("Failed to find project folder")]
    NoProjectDir,
    #[error("Failed to open PID file: {0}")]
    NotOpened(std::io::Error),
    #[error("Failed to spawn kill command")]
    CommandNotSpawned,
    #[error("Kill command failed: {0}")]
    CommandFailed(std::io::Error),
}

pub async fn kill_other_instances() -> Result<(), KillError> {
    let Some(dir_path) = directories::ProjectDirs::from("io.github", "tancop", "Conductor") else {
        return Err(KillError::NoProjectDir);
    };

    let Ok(pid) = std::fs::read_to_string(dir_path.cache_dir().join("conductor.pid")) else {
        return Err(KillError::NotOpened(std::io::Error::last_os_error()));
    };

    #[allow(unused_assignments)] // might be None on some platforms
    let mut cmd: Option<&mut Command> = None;

    cfg_if! {
        if #[cfg(windows)] {
            let mut new_cmd = Command::new("taskkill");
            cmd = Some(new_cmd.args(&["/f", "/pid", &pid]));
        } else if #[cfg(unix)] {
            let mut new_cmd = Command::new("kill");
            cmd = Some(new_cmd.args(&[&pid]));
        }
    }

    let Some(cmd) = cmd else {
        return Err(KillError::CommandNotSpawned);
    };

    match cmd.output().await {
        Ok(out) => {
            log::debug!("Kill command returned {}", out.status);
            Ok(())
        }
        Err(e) => Err(KillError::CommandFailed(e)),
    }
}
