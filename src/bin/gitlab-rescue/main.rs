mod clap_app;

use crate::clap_app::app;
use gitlab_rescue::{
    app_error::{handle_error, AppError::InvalidInput, Result},
    get_variable::GetVariable,
};
use std::{convert::TryFrom, process};

fn run_gitlab_rescue() -> Result<bool> {
    use gitlab_rescue::Command;
    match app().get_matches().subcommand() {
        ("get", Some(args)) => GetVariable::try_from(args)?.perform(),
        _ => Err(InvalidInput(
            "Introduced command is not valid. For more information try --help.".to_owned(),
        )),
    }
}

fn main() {
    match run_gitlab_rescue() {
        Ok(true) => process::exit(0),
        Ok(false) => process::exit(1),
        Err(err) => handle_error(err),
    }
}
