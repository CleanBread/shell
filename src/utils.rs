use anyhow::{Context, Result};
use std::{fmt::Display, io::stdin};
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

// pub fn parse_input(input: &str) -> () {
//   input.split_whitespace()
// }
