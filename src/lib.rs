use anyhow::{Ok, Result};
use std::io::{Write, stderr, stdout};
use termion::{
    cursor::{self},
    raw::IntoRawMode,
};

use crate::{
    builtin_commands::BuiltinCommand,
    execute_output::ExecuteOutput,
    utils::{get_paths, get_user_input},
};

mod builtin_commands;
mod execute_output;
mod utils;

pub fn run() -> Result<()> {
    let mut stdout = stdout().lock().into_raw_mode().unwrap();
    let mut stderr = stderr().lock().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        cursor::BlinkingUnderline,
    )
    .unwrap();
    stdout.flush().unwrap();

    let paths = get_paths()?;

    loop {
        let input = get_user_input()?;

        if input.is_empty() {
            continue;
        }

        let parsed_input = BuiltinCommand::parse_input(&input);
        let (command, args) = parsed_input.split_first().unwrap();

        let command: BuiltinCommand = command.as_str().into();
        let ExecuteOutput { out, err, exit } = command.execute(&args, &paths);

        if !out.is_empty() {
            let text = out.trim_end_matches('\n').replace('\n', "\r\n");
            write!(stdout, "\r\n{}", text)?;
            stdout.flush()?;
        }

        if !err.is_empty() {
            let text = err.trim_end_matches('\n').replace('\n', "\r\n");
            write!(stderr, "\r\n{}", text)?;
            stderr.flush()?;
        }

        if exit {
            break;
        }
    }

    stdout.suspend_raw_mode().unwrap();
    stderr.suspend_raw_mode().unwrap();

    Ok(())
}
