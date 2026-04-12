use anyhow::{Context, Result};
use std::{
    env::{self, SplitPaths},
    fmt::Display,
    io::stdin,
    path::{Path, PathBuf},
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

    Ok(split_paths.collect())
}
