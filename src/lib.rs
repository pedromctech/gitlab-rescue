//! `adaptoid` is a tool for updating Dockerfile' packages.
//!

use crate::app_error::AppError;

pub mod app_error;
pub mod app_std;
pub mod get_variable;
pub mod gitlab_instance;

pub trait Command {
    fn perform(&self) -> Result<bool, AppError>;
}
