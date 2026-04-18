use anyhow::{Ok, Result};
use std::io::{self, Write};

use crate::{
    builtin_commands::BuiltinCommand,
    utils::{get_paths, get_user_input},
};

mod builtin_commands;
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

        match input.into() {
            BuiltinCommand::ChangeDirectory(args) => BuiltinCommand::builtin_cd(args),
            BuiltinCommand::Pwd => BuiltinCommand::builtin_pwd(),
            BuiltinCommand::Type(args) => BuiltinCommand::builtin_type(&paths, args),
            BuiltinCommand::Echo(args) => BuiltinCommand::builtin_echo(args),
            BuiltinCommand::NotFound(input) => BuiltinCommand::builtin_not_found(&paths, input),
            BuiltinCommand::Exit => break,
        }
    }

    Ok(())
}
