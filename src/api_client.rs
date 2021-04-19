use crate::gitlab_api::{GitLabApi, GitLabApiV4};

pub const DEFAULT_ENVIRONMENT: &str = "All";
pub const MAX_PER_PAGE: usize = 100;

pub fn api_client(url: &str, token: &str) -> impl GitLabApi + 'static {
    GitLabApiV4::new(url.to_owned(), token.to_owned())
}
