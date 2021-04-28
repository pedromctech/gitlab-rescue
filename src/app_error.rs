use ansi_term::Colour::Red;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Specification for application errors
#[derive(Clone, Debug, PartialEq)]
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

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> AppError {
        AppError::InvalidInput(format!("{}", e))
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

/// Result is a type that represents either success ([Ok]) or failure ([AppError]).
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client as BlockingClient;

    #[test]
    fn test_app_error_from_reqwest() {
        BlockingClient::builder()
            .build()
            .unwrap()
            .get("http://bad-url")
            .send()
            .map_or_else(|e| assert!(matches!(AppError::from(e), AppError::Api(_))), |_| panic!());
    }

    #[test]
    fn test_app_error_from_stdio_error() {
        assert!(matches!(AppError::from(std::io::Error::new(std::io::ErrorKind::Other, "Error")), AppError::InvalidInput(_)));
    }

    #[test]
    fn test_invalid_input_error_display() {
        assert_eq!(
            format!("{}", AppError::InvalidInput("An error".to_owned())),
            format!("{} An error", Red.bold().paint("[InvalidInputError]"))
        );
    }

    #[test]
    fn test_api_error_display() {
        assert_eq!(format!("{}", AppError::Api("An error".to_owned())), format!("{} An error", Red.bold().paint("[ApiError]")));
    }

    #[test]
    fn test_cli_error_display() {
        assert_eq!(format!("{}", AppError::Cli("An error".to_owned())), format!("{} An error", Red.bold().paint("[CliError]")));
    }
}
