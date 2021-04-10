mod clap_app;

use crate::clap_app::app;
use gitlab_rescue::{
    app_error::{handle_error, AppError::InvalidInput, Result},
    get_variable::GetVariable,
    Performable,
};

fn run_gitlab_rescue() -> Result<String> {
    match app().get_matches().subcommand() {
        // Get command
        ("get", Some(args)) => GetVariable::from(args).perform(args.value_of("VARIABLE_NAME").unwrap()),
        _ => Err(InvalidInput("Introduced command is not valid. For more information try --help.".to_owned())),
    }
}

fn main() {
    match run_gitlab_rescue() {
        Ok(output) => println!("{}", output),
        Err(err) => handle_error(err),
    }
}
