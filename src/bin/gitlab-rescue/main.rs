use gitlab_rescue::app_error::AppError::InvalidInput;
use gitlab_rescue::clap_app::app;
use gitlab_rescue::dotenv::DotEnvCommand;
use gitlab_rescue::get_variable::GetVariableCommand;
use gitlab_rescue::io::IO;
use gitlab_rescue::Performable;
use std::process;

/// Main action that applies the effect returned by command
fn main() {
    match app().get_matches().subcommand() {
        ("get", Some(args)) => GetVariableCommand::from(args).get_action(),
        ("dotenv", Some(args)) => DotEnvCommand::from(args).get_action(),
        _ => IO::unit(|| Err(InvalidInput("Command is not valid. For more information try --help.".to_owned()))),
    }
    .apply()
    .unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
}
