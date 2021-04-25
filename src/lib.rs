//! `gitlab-rescue` is a CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)
//!

mod macros;

pub mod api_client;
pub mod app_error;
pub mod clap_app;
pub mod dotenv;
pub mod gen;
pub mod get_variable;
mod gitlab_api;
pub mod io;
pub mod shell_types;

use crate::app_error::Result;
use crate::io::IO;

/// Trait for all `gitlab-rescue` commands
pub trait Performable {
    /// Get action that contains a IO object with an effect
    fn get_action(self) -> IO<Result<()>>;
}
