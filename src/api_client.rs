use crate::gitlab_api::{GitLabApi, GitLabApiV4};

pub const DEFAULT_ENVIRONMENT: &str = "All";

/// Returns a `GitLabApi` object used for connecting to GitLab API
///
/// # Arguments
///
/// * `url` - GitLab instance URL
/// * `token` - Token used to connect to GitLab API
///
/// # Example
///
/// ```
/// use gitlab_rescue::api_client::api_client;
/// let api = api_client("https://gitlab.com", "A_GITLAB_TOKEN");
/// ```
pub fn api_client(url: &str, token: &str) -> impl GitLabApi + 'static {
    GitLabApiV4::new(url.to_owned(), token.to_owned())
}
