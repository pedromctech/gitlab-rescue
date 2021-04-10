use crate::app_error::Result;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitLabVariable {
    variable_type: String,
    key: String,
    pub value: String,
    protected: bool,
    masked: bool,
    environment_scope: String,
}

pub trait GitLabApi {
    fn new(gitlab_api_url: String, gitlab_token: String) -> Self;
    /// Get a variable value from a specific GitLab project
    fn get_from_project(&self, project_id: &str, name: &str, environment: &str) -> Result<GitLabVariable>;
    /// Get a variable value from a specific GitLab group
    fn get_from_group(&self, group_id: &str, name: &str) -> Result<GitLabVariable>;
}

#[derive(Debug)]
pub struct GitLabApiV4 {
    url: String,
    token: String,
}

impl GitLabApiV4 {
    fn get(&self, endpoint: &str) -> Result<GitLabVariable> {
        Ok(Client::builder()
            .build()?
            .get(format!("{}/{}", self.url, endpoint))
            .header("PRIVATE-TOKEN", &self.token)
            .send()?
            .error_for_status()?
            .json()?)
    }
}

impl<'a> GitLabApi for GitLabApiV4 {
    fn new(url: String, token: String) -> Self {
        GitLabApiV4 {
            url: format!("{}/api/v4", url),
            token: token,
        }
    }

    fn get_from_project(&self, project_id: &str, name: &str, environment: &str) -> Result<GitLabVariable> {
        self.get(&format!("projects/{}/variables/{}?filter[environment_scope]={}", project_id, name, environment))
    }

    fn get_from_group(&self, group_id: &str, name: &str) -> Result<GitLabVariable> {
        self.get(&format!("groups/{}/variables/{}", group_id, name))
    }
}
