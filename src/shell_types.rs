use ansi_term::Colour::Yellow;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Shell types
#[derive(Clone, Copy, Debug)]
pub enum ShellType {
    /// Posix shell (bash, zsh)
    Posix,
    /// Fish shell
    Fish,
}

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ShellType::Posix => write!(f, "{}", Yellow.bold().paint("POSIX")),
            ShellType::Fish => write!(f, "{}", Yellow.bold().paint("FISH")),
        }
    }
}

impl ShellType {
    /// Generate a `export` command according to shell type
    ///
    /// # Example
    ///
    /// ```rust
    /// let shell = ShellType::Posix;
    /// let expected = "export HOME=\"/home/user\"".to_owned();
    /// assert_eq!(export_command("HOME".to_owned(), "/home/user".to_owned()), expected);
    /// ```
    ///
    pub fn export_command(&self, variable: String, value: String) -> String {
        match self {
            ShellType::Posix => format!("export {}=\"{}\"", variable, value),
            ShellType::Fish => format!("set -gx {} \"{}\"", variable, value),
        }
    }
}
