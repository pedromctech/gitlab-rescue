use crate::gitlab::{GitLabApi, GitLabApiV4};

pub fn api_client(_version: &str, url: &str, token: &str) -> impl GitLabApi {
    GitLabApiV4::new(url.to_owned(), token.to_owned())
}
