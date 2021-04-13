//! `gitlab-rescue` is a CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)
//!

mod macros;

mod api_client;
pub mod app_error;
pub mod dotenv;
pub mod get_variable;
mod gitlab_api;
pub mod io;

use crate::io::IO;
use crate::app_error::Result;

pub trait Performable {
    fn get_action(self) -> IO<Result<()>>;
}
