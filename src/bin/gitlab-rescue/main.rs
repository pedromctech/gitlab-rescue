mod clap_app;

use crate::clap_app::app;
use gitlab_rescue::app_error::{handle_error, AppError::InvalidInput, Result};
use gitlab_rescue::dotenv::DotEnvCommand;
use gitlab_rescue::get_variable::GetVariableCommand;
use gitlab_rescue::io::IO;
use gitlab_rescue::Performable;
use std::process;

/// Returns action according to command introduced by user
fn get_action() -> IO<Result<()>> {
    match app().get_matches().subcommand() {
        ("get", Some(args)) => GetVariableCommand::from(args).get_action(),
        ("dotenv", Some(args)) => DotEnvCommand::from(args).get_action(),
        _ => IO::unit(|| Err(InvalidInput("Command is not valid. For more information try --help.".to_owned()))),
    }
}

/// Main action that applies the effect returned by command
fn main() {
    match get_action().apply() {
        Ok(_) => process::exit(0),
        Err(e) => handle_error(e),
    }
}
