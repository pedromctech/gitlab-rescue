use std::fmt::{Display, Formatter, Result as FmtResult};

/// Shell types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShellType {
    /// Posix shell (bash, zsh)
    Posix,
    /// Fish shell
    Fish,
}

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ShellType::Posix => write!(f, "posix"),
            ShellType::Fish => write!(f, "fish"),
        }
    }
}

impl ShellType {
    /// Generate a `export` command according to shell type
    ///
    /// # Example
    ///
    /// ```rust
    /// use gitlab_rescue::shell_types::ShellType;
    ///
    /// let shell = ShellType::Posix;
    /// let expected = "export HOME=\"/home/user\"".to_owned();
    /// assert_eq!(shell.export_command("HOME".to_owned(), "/home/user".to_owned()), expected);
    /// ```
    ///
    pub fn export_command(&self, variable: String, value: String) -> String {
        match self {
            ShellType::Posix => format!("export {}=\"{}\"", variable, value),
            ShellType::Fish => format!("set -gx {} \"{}\"", variable, value),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::gen::tests::gen_bool;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GEN_SHELL_TYPE: ShellType = if gen_bool() { ShellType::Posix } else { ShellType::Fish };
    }
}
