use crate::app_error::{AppError::InvalidInput, Result};

#[derive(Debug)]
pub struct GitLabInstance {
    /// GitLab API URL
    pub url: String,
    /// GitLab's project ID
    pub project_id: String,
    /// A valid GitLab API token
    pub api_token: String,
}

impl GitLabInstance {
    /// Create a object that stores GitLab instance information
    pub fn new(url: String, project_id: String, api_token: String) -> Result<Self> {
        Ok(GitLabInstance {
            url,
            project_id: if project_id.is_empty() {
                return Err(InvalidInput("Please set a valid GitLab project ID (-p, --project-id or $GITLAB_PROJECT_ID variable)".to_owned()));
            } else {
                project_id
            },
            api_token: if api_token.is_empty() {
                return Err(InvalidInput("Please set a valid GitLab API token (-t, --token or $GITLAB_API_TOKEN variable)".to_owned()));
            } else {
                api_token
            },
        })
    }
}
