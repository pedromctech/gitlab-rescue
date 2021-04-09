//! `adaptoid` is a tool for updating Dockerfile' packages.
//!

use crate::app_error::AppError;

pub mod app_error;
pub mod app_std;
pub mod get_variable;
pub mod gitlab;

pub trait ProjectCommand {
    fn perform(&self, project_id: &str, name: &str) -> Result<String, AppError>;
}

pub trait GroupCommand {
    fn perform(&self, group_id: &str, name: &str) -> Result<String, AppError>;
}
