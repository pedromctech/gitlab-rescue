mod clap_app;

use crate::clap_app::app;
use gitlab_rescue::{
    app_error::{handle_error, AppError, AppError::InvalidInput, Result},
    get_variable::GetVariable,
    GroupCommand, ProjectCommand,
};
use std::convert::TryFrom;

fn invalid_input() -> AppError {
    InvalidInput("Introduced command is not valid. For more information try --help.".to_owned())
}

fn run_gitlab_rescue() -> Result<String> {
    match app().get_matches().subcommand() {
        // Project commands
        ("project", Some(args)) => match args.subcommand() {
            ("get", Some(sub_args)) => ProjectCommand::perform(
                &GetVariable::try_from(sub_args)?,
                args.value_of("GITLAB_PROJECT_ID").unwrap(),
                sub_args.value_of("VARIABLE_NAME").unwrap(),
            ),
            _ => Err(invalid_input()),
        },
        // Group commands
        ("group", Some(args)) => match args.subcommand() {
            ("get", Some(sub_args)) => GroupCommand::perform(
                &GetVariable::try_from(sub_args)?,
                args.value_of("GITLAB_GROUP_ID").unwrap(),
                sub_args.value_of("VARIABLE_NAME").unwrap(),
            ),
            _ => Err(invalid_input()),
        },
        // Env command
        ("env", Some(args)) => match args.subcommand() {
            _ => Err(invalid_input()),
        },
        _ => Err(invalid_input()),
    }
}

fn main() {
    match run_gitlab_rescue() {
        Ok(output) => println!("{}", output),
        Err(err) => handle_error(err),
    }
}
