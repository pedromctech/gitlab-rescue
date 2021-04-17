use crate::gitlab_api::{GitLabApi, GitLabApiV4};

pub fn api_client(url: &str, token: &str) -> impl GitLabApi + 'static {
    GitLabApiV4::new(url.to_owned(), token.to_owned())
}
