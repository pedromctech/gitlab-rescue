mod clap_app;

use crate::clap_app::app;
use gitlab_rescue::{
    app_error::{handle_error, AppError::InvalidInput, Result},
    dotenv::DotEnvCommand,
    get_variable::GetVariableCommand,
    io::IO,
    Performable,
};
use std::process;

fn get_action() -> IO<Result<()>> {
    match app().get_matches().subcommand() {
        ("get", Some(args)) => GetVariableCommand::from(args).get_action(),
        ("dotenv", Some(args)) => DotEnvCommand::from(args).get_action(),
        _ => IO::unit(|| Err(InvalidInput("Introduced command is not valid. For more information try --help.".to_owned()))),
    }
}

fn main() {
    match get_action().apply() {
        Ok(_) => process::exit(0),
        Err(e) => handle_error(e),
    }
}
