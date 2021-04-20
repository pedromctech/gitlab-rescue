use ansi_term::Colour::Red;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::process;

/// Specification for application errors
#[derive(Clone, Debug)]
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

impl From<reqwest::header::ToStrError> for AppError {
    fn from(e: reqwest::header::ToStrError) -> AppError {
        AppError::Api(format!("{}", e))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> AppError {
        AppError::InvalidInput(format!("{}", e))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> AppError {
        AppError::Cli(format!("{}", e))
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

/// Print error in STDERR and exit with error code (1)
///
/// # Arguments
///
/// * `err` - [AppError](enum@AppError) object
///
pub fn handle_error(err: AppError) {
    eprintln!("{}", err);
    process::exit(1);
}

/// Result is a type that represents either success ([Ok]) or failure ([AppError]).
pub type Result<T> = std::result::Result<T, AppError>;
