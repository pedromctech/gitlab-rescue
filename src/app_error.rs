use ansi_term::Colour::Red;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    process,
};

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    Cli(String),
}

impl Error for AppError {}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            AppError::InvalidInput(e) => {
                write!(f, "{} {}", Red.bold().paint("[InvalidInputError]"), e)
            }
            AppError::Cli(e) => write!(f, "{} {}", Red.bold().paint("[CliError]"), e),
        }
    }
}

pub fn handle_error(err: AppError) {
    eprintln!("{}", err);
    process::exit(1);
}

pub type Result<T> = std::result::Result<T, AppError>;
