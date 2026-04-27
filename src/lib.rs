use anyhow::{Ok, Result};
use std::io::{self, Write, stderr, stdout};

use crate::{
    builtin_commands::BuiltinCommand,
    execute_output::ExecuteOutput,
    utils::{get_paths, get_user_input},
};

mod builtin_commands;
mod execute_output;
mod utils;

pub fn run() -> Result<()> {
    let paths = get_paths()?;

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let input = get_user_input()?;

        if input.is_empty() {
            continue;
        }

        let parsed_input = BuiltinCommand::parse_input(&input);
        let (command, args) = parsed_input.split_first().unwrap();

        let command: BuiltinCommand = command.as_str().into();
        let ExecuteOutput { out, err, exit } = command.execute(&args, &paths);

        stdout().write(&out).ok();
        stdout().flush()?;

        stderr().write(&err).ok();
        stderr().flush()?;

        if exit {
            break;
        }
    }

    Ok(())
}
