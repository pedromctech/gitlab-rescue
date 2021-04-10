use crate::{
    app_error::Result,
    gitlab::{GitLabApi, GitLabApiV4},
};

pub trait Command {
    fn api_v4(&self, url: &str, token: &str) -> GitLabApiV4 {
        GitLabApiV4::new(url.to_owned(), token.to_owned())
    }
    fn perform(&self, name: &str) -> Result<String>;
}
