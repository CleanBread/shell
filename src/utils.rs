use anyhow::{Context, Result};
use libc::{POLLIN, WNOHANG, pollfd};
use std::{
    env,
    fs::DirEntry,
    io::{Write, stdout},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use termion::{cursor, event::Key};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
}

pub fn get_input(key_rx: &Receiver<Key>, job_rx: &Receiver<i32>) -> Result<String> {
    let mut stdout = stdout();
    let mut input = String::new();
    let mut input_pos: usize = 0;

    write!(stdout, "\r\n$ ")?;
    stdout.flush()?;

    loop {
        while let Ok(_) = job_rx.try_recv() {
            write!(stdout, "$ {}", input)?;
            let tail = (input.len() - input_pos) as u16;
            if tail > 0 {
                write!(stdout, "{}", cursor::Left(tail))?;
            }
        }
        stdout.flush()?;

        // wait up to 50ms for a key, then loop back to check sigchld
        match key_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Key::Char('\n')) => {
                write!(stdout, "\r\n")?;
                stdout.flush()?;
                break;
            }
            Ok(Key::Char('\t')) => {}
            Ok(Key::Char(c)) => {
                input.insert(input_pos, c);
                input_pos += 1;

                write!(stdout, "{}", &input[input_pos - 1..])?;

                let tail = input.len() - input_pos;
                if tail > 0 {
                    write!(stdout, "{}", cursor::Left(tail as u16))?;
                }
                stdout.flush()?;
            }
            Ok(Key::Backspace) => {
                if input_pos > 0 {
                    input_pos -= 1;
                    input.remove(input_pos);

                    write!(stdout, "{}", cursor::Left(1))?;
                    write!(stdout, "{} ", &input[input_pos..])?;

                    let tail = input.len() - input_pos + 1;
                    write!(stdout, "{}", cursor::Left(tail as u16))?;
                    stdout.flush()?;
                }
            }
            Ok(Key::Left) => {
                if input_pos > 0 {
                    input_pos -= 1;
                    write!(stdout, "{}", cursor::Left(1))?;
                    stdout.flush()?;
                }
            }
            Ok(Key::Right) => {
                if input_pos < input.len() {
                    input_pos += 1;
                    write!(stdout, "{}", cursor::Right(1))?;
                    stdout.flush()?;
                }
            }
            Ok(Key::Up) | Ok(Key::Down) => {}
            Ok(_) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(input.trim().to_string())
}

pub fn get_paths() -> Result<Vec<PathBuf>> {
    let paths = env::var_os("PATH").context("Getting PATH evn variable")?;
    let split_paths = env::split_paths(&paths).filter(|path| path.is_dir());

    Ok(split_paths.collect())
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
