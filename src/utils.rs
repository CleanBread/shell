use anyhow::{Context, Result};
use std::{
    env::{self},
    fmt::Display,
    fs::DirEntry,
    io::stdin,
    path::PathBuf,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
}

pub fn get_user_input() -> Result<String> {
    let mut user_input = String::new();
    stdin()
        .read_line(&mut user_input)
        .context("reading use input")?;

    Ok(user_input.trim().to_string())
}

pub fn pritn_error(message: impl Display) {
    eprintln!("{}", message);
}

pub fn get_paths() -> Result<Vec<PathBuf>> {
    let paths = env::var_os("PATH").context("Getting PATH evn variable")?;
    let split_paths = env::split_paths(&paths).filter(|path| path.is_dir());

    Ok(split_paths.inspect(|x| println!("{x:?}")).collect())
}

#[cfg(unix)]
fn is_executable(entry: &DirEntry) -> bool {
    use std::os::unix::fs::PermissionsExt;
    entry
        .metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(entry: &DirEntry) -> bool {
    matches!(
        entry.path().extension().and_then(|e| e.to_str()),
        Some("exe" | "bat" | "cmd")
    )
}

pub fn find_in_path(
    paths: &[PathBuf],
    file_name: &str,
    permissions_mode: Option<u32>,
) -> Option<DirEntry> {
    for path in paths {
        let Ok(entries) = path.read_dir() else {
            continue;
        };

        for entry in entries.flatten() {
            if entry.file_name() != file_name {
                continue;
            }

            if permissions_mode.is_some() && !is_executable(&entry) {
                continue;
            }

            return Some(entry);
        }
    }

    None
}
