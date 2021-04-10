//! `gitlab-rescue` is a CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)
//!
pub mod api_client;
pub mod app_error;
pub mod app_std;
pub mod get_variable;
pub mod gitlab;

use crate::app_error::Result;

pub trait Performable {
    fn perform(&self, name: &str) -> Result<String>;
}
