use anyhow::{Ok, Result};
use std::io::{self, Write};

use crate::{
    commands::Command,
    utils::{CustomError, get_user_input, pritn_error},
};

mod commands;
mod utils;

pub fn run() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let command = get_user_input()?.as_str().into();

        match command {
            Command::Echo(text) => {
                println!("{}", text);
            }
            Command::Exit => break,
            Command::NotFound(command_string) => {
                pritn_error(CustomError::CommandNotFound(command_string));
            }
        }
    }

    Ok(())
}
