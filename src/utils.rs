use anyhow::{Context, Result};
use std::{
    env::{self},
    fs::DirEntry,
    io::{Write, stdin, stdout},
    path::PathBuf,
    process,
};
use termion::{cursor, event::Key, input::TermRead};
use thiserror::Error;

use crate::execute_output::ExecuteOutput;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
}

pub enum PipeInput {
    Stream(process::ChildStdout),
    Buffer(ExecuteOutput),
}

pub fn get_input() -> Result<String> {
    let mut stdout = stdout().lock();
    let stdin = stdin().lock();
    let mut input = String::new();

    let mut input_pos: usize = 0;

    write!(stdout, "\r\n$ ")?;
    stdout.flush()?;

    for k in stdin.keys() {
        match k.as_ref().unwrap() {
            Key::Char('\t') => print!("TAB"), // TODO: impl autocomplete
            Key::Char('\n') => {
                write!(stdout, "\r\n")?;
                stdout.flush()?;
                break;
            }
            Key::Char(c) => {
                input.insert(input_pos, *c);
                input_pos += 1;

                write!(stdout, "{}", &input[input_pos - 1..])?;

                let tail = input.len() - input_pos;
                if tail > 0 {
                    write!(stdout, "{}", cursor::Left(tail as u16))?;
                }
            }
            Key::Backspace => {
                if input_pos > 0 {
                    input_pos -= 1;
                    input.remove(input_pos);

                    write!(stdout, "{}", cursor::Left(1))?;
                    write!(stdout, "{} ", &input[input_pos..])?;

                    let tail = input.len() - input_pos + 1;
                    write!(stdout, "{}", cursor::Left(tail as u16))?;
                }
            }
            Key::Left => {
                if input_pos > 0 {
                    input_pos -= 1;

                    write!(stdout, "{}", cursor::Left(1))?;
                }
            }
            Key::Right => {
                if input_pos < input.len() {
                    input_pos += 1;

                    write!(stdout, "{}", cursor::Right(1))?;
                }
            }

            Key::Up => print!("↑"),   // TODO: impl history
            Key::Down => print!("↓"), // TODO: impl history
            _ => {
                print!("{:?}", k)
            }
        }

        stdout.flush()?;
    }

    Ok(input.trim().to_string())
}

pub fn get_paths() -> Result<Vec<PathBuf>> {
    let paths = env::var_os("PATH").context("Getting PATH evn variable")?;
    let split_paths = env::split_paths(&paths).filter(|path| path.is_dir());

    Ok(split_paths
        // .inspect(|x| println!("{x:?}"))
        .collect())
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
