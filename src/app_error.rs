use ansi_term::Colour::Red;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    process,
};

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    Api(String),
    Cli(String),
}

impl Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> AppError {
        AppError::Api(format!("{}", e))
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            AppError::InvalidInput(e) => write!(f, "{} {}", Red.bold().paint("[InvalidInputError]"), e),
            AppError::Api(e) => write!(f, "{} {}", Red.bold().paint("[ApiError]"), e),
            AppError::Cli(e) => write!(f, "{} {}", Red.bold().paint("[CliError]"), e),
        }
    }
}

pub fn handle_error(err: AppError) {
    eprintln!("{}", err);
    process::exit(1);
}

pub type Result<T> = std::result::Result<T, AppError>;
