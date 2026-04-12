use anyhow::{Ok, Result};
use std::io::{self, Write};

use crate::{
    builtin_commands::BuiltinCommand,
    utils::{CustomError, get_paths, get_user_input, pritn_error},
};

mod builtin_commands;
mod utils;

pub fn run() -> Result<()> {
    let paths = get_paths()?;

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let builtin = get_user_input()?.as_str().into();

        match builtin {
            BuiltinCommand::Exit => break,
            BuiltinCommand::Type(args) => BuiltinCommand::builtin_type(&paths, args),
            BuiltinCommand::Echo(text) => BuiltinCommand::builtin_echo(text),
            BuiltinCommand::NotFound(command) => BuiltinCommand::builtin_not_found(command),
        }
    }

    Ok(())
}
