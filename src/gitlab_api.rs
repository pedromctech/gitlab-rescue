use crate::app_error::{AppError::Cli, Result};
use reqwest::blocking::{Client as BlockingClient, Response as BlockingResponse};
use serde::{Deserialize, Serialize};

/// GitLab project infomation
#[derive(Clone, Debug, PartialEq)]
pub struct GitLabProject {
    /// GitLab project name
    pub name: String,
    /// Project's variables
    pub variables: Vec<GitLabVariable>,
}

/// GitLab variable type
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum GitLabVariableType {
    #[serde(rename = "env_var")]
    /// Environment variable type
    EnvVar,
    #[serde(rename = "file")]
    /// File type
    File,
}

/// GitLab variable information
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GitLabVariable {
    /// The type of a variable. Available types are: env_var and file
    pub variable_type: GitLabVariableType,
    /// The key of the variable
    pub key: String,
    /// The value of a variable
    pub value: String,
    /// Variable's environment
    pub environment_scope: String,
}

impl GitLabVariable {
    /// Clone [GitLabVariable](struct@GitLabVariable) object parsing `environment_scope` attribute
    fn clone_from_response(&self) -> GitLabVariable {
        GitLabVariable {
            environment_scope: if self.environment_scope == "*" {
                "All".to_owned()
            } else {
                self.environment_scope.clone()
            },
            variable_type: self.variable_type,
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

pub trait GitLabApi {
    /// Returns a new [GitLabApi](trait@GitLabApi) object
    fn new(gitlab_api_url: String, gitlab_token: String) -> Self;
    /// Get a variable value from a specific GitLab project
    fn get_from_project(&self, project: &str, name: &str, env: &str) -> Result<GitLabVariable>;
    /// Get a variable value from a specific GitLab group
    fn get_from_group(&self, group: &str, name: &str) -> Result<GitLabVariable>;
    /// List variables from a specific GitLab project
    fn list_from_project(&self, project: &str, page: usize, per_page: usize) -> Result<(Vec<GitLabVariable>, usize)>;
}

/// Implementation of [GitLabApi](trait@GitLabApi) v4
#[derive(Clone, Debug)]
pub struct GitLabApiV4 {
    url: String,
    token: String,
}

impl<'a> GitLabApi for GitLabApiV4 {
    fn new(url: String, token: String) -> Self {
        GitLabApiV4 {
            url: format!("{}/api/v4", url),
            token,
        }
    }

    fn get_from_project(&self, project: &str, name: &str, env: &str) -> Result<GitLabVariable> {
        self.get(&format!(
            "projects/{}/variables/{}?filter[environment_scope]={}",
            project,
            name,
            if env == "All" { "*" } else { env }
        ))
    }

    fn get_from_group(&self, group: &str, name: &str) -> Result<GitLabVariable> {
        self.get(&format!("groups/{}/variables/{}", group, name))
    }

    fn list_from_project(&self, project: &str, page: usize, per_page: usize) -> Result<(Vec<GitLabVariable>, usize)> {
        self.list(&format!("projects/{}/variables?page={}&per_page={}", project, page, per_page))
    }
}

impl GitLabApiV4 {
    /// Return a [GitLabVariable](struct@GitLabVariable) object with variable information from GitLabAPI
    ///
    /// # Arguments
    ///
    /// * `endpoint` - GitLab API endpoint to consume
    ///
    fn get(&self, endpoint: &str) -> Result<GitLabVariable> {
        Ok(BlockingClient::builder()
            .build()?
            .get(format!("{}/{}", self.url, endpoint))
            .header("PRIVATE-TOKEN", &self.token)
            .send()?
            .error_for_status()?
            .json::<GitLabVariable>()?
            .clone_from_response())
    }

    /// Return a list of [GitLabVariable](struct@GitLabVariable) objects from GitLabAPI
    ///
    /// # Arguments
    ///
    /// * `endpoint` - GitLab API endpoint to consume
    ///
    fn list(&self, endpoint: &str) -> Result<(Vec<GitLabVariable>, usize)> {
        let res = BlockingClient::builder()
            .build()?
            .get(format!("{}/{}", self.url, endpoint))
            .header("PRIVATE-TOKEN", &self.token)
            .send()?
            .error_for_status()?;
        let total = get_pagination_header(&res, "x-total")?;
        Ok((res.json::<Vec<GitLabVariable>>()?.iter().map(|v| v.clone_from_response()).collect(), total))
    }
}

/// Return numeric header from GitLab API response
///
/// # Arguments
///
/// * `res`    - Reference of BlockingResponse object
/// * `header` - Header to extract
///
fn get_pagination_header(res: &BlockingResponse, header: &str) -> Result<usize> {
    match res.headers().get(header).and_then(|h| h.to_str().ok()) {
        Some(h) if h.is_empty() => Ok(0),
        Some(h) if h.parse::<usize>().is_ok() => Ok(h.parse::<usize>().unwrap()),
        _ => Err(Cli(format!("Header {} not valid in GitLab response", header))),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::api_client::DEFAULT_ENVIRONMENT;
    use crate::gen::tests::{gen_alpha_char, gen_bool, gen_char, gen_usize_from_range};
    use httpmock::{MockServer, Then, When};
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GEN_NAME: String = gen_alpha_char(5);
        pub static ref GEN_PROJECT_NAME: String = gen_alpha_char(5);
        pub static ref GEN_GROUP_NAME: String = gen_alpha_char(5);
        pub static ref GEN_TOKEN: String = gen_alpha_char(5);
        pub static ref GEN_ENVIRONMENT: String = "A".to_owned();
        pub static ref GEN_GITLAB_VARIABLE: GitLabVariable = GitLabVariable {
            environment_scope: GEN_ENVIRONMENT.clone(),
            ..gen_variable(None)
        };
        pub static ref GEN_GITLAB_VARIABLE_ALL: GitLabVariable = GitLabVariable {
            environment_scope: DEFAULT_ENVIRONMENT.to_string(),
            ..gen_variable(None)
        };
        pub static ref GEN_GITLAB_PROJECT: GitLabProject = GitLabProject {
            name: GEN_PROJECT_NAME.to_string(),
            variables: vec![],
        };
    }

    pub fn gen_variable(var_type: Option<GitLabVariableType>) -> GitLabVariable {
        GitLabVariable {
            key: gen_alpha_char(5).to_uppercase(),
            value: gen_alpha_char(5),
            environment_scope: gen_char(b"ABC*"),
            variable_type: var_type.map_or_else(|| if gen_bool() { GitLabVariableType::EnvVar } else { GitLabVariableType::File }, |t| t),
        }
    }

    pub fn gen_variable_list(size: usize) -> Vec<GitLabVariable> {
        (0..size).into_iter().fold(vec![], |mut acc: Vec<GitLabVariable>, _| {
            acc.push(gen_variable(None));
            acc
        })
    }

    pub fn httpmock_group_variable() -> impl FnOnce(When, Then) {
        move |when, then| {
            when.method("GET").path(format!("/api/v4/groups/{}/variables/{}", GEN_GROUP_NAME.clone(), GEN_NAME.clone()));
            then.status(200).header("Content-Type", "application/json").json_body_obj(&GEN_GITLAB_VARIABLE.clone());
        }
    }

    pub fn httpmock_project_variable(env: String) -> impl FnOnce(When, Then) {
        let (var_all, var_env) = (GEN_GITLAB_VARIABLE_ALL.clone(), GEN_GITLAB_VARIABLE.clone());
        move |when, then| {
            when.method("GET")
                .path(format!("/api/v4/projects/{}/variables/{}", *GEN_PROJECT_NAME, GEN_NAME.clone()))
                .query_param("filter[environment_scope]", &env);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(if env == "*" { &var_all } else { &var_env });
        }
    }

    pub fn httpmock_list_variables(total: usize, per_page: usize) -> impl FnOnce(When, Then) {
        move |when, then| {
            when.method("GET").path(format!("/api/v4/projects/{}/variables", GEN_GITLAB_PROJECT.name.clone()));
            then.status(200)
                .header("Content-Type", "application/json")
                .header("x-total", &total.to_string())
                .json_body_obj(&gen_variable_list(if per_page < total { per_page } else { total }));
        }
    }

    #[test]
    fn test_should_get_variable_list_from_project() {
        let num_variables = gen_usize_from_range(10, 300);
        let server = MockServer::start();
        let mock = server.mock(httpmock_list_variables(num_variables, num_variables));
        GitLabApiV4::new(server.base_url(), gen_alpha_char(5))
            .list_from_project(&GEN_GITLAB_PROJECT.name, 1, num_variables)
            .map_or_else(|_| panic!(), |(l, _)| assert_eq!(l.len(), num_variables));
        mock.assert();
    }

    #[test]
    fn test_should_get_a_variable_from_project() {
        let server = MockServer::start();
        let mock = server.mock(httpmock_project_variable(GEN_ENVIRONMENT.clone()));
        GitLabApiV4::new(server.base_url(), gen_alpha_char(5))
            .get_from_project(&GEN_GITLAB_PROJECT.name, &GEN_NAME, &GEN_ENVIRONMENT)
            .map_or_else(|_| panic!(), |v| assert_eq!(v, *GEN_GITLAB_VARIABLE));
        mock.assert();
    }

    #[test]
    fn test_should_get_a_variable_from_group() {
        let server = MockServer::start();
        let mock = server.mock(httpmock_group_variable());
        GitLabApiV4::new(server.base_url(), gen_alpha_char(5))
            .get_from_group(&GEN_GROUP_NAME, &GEN_NAME)
            .map_or_else(|_| panic!(), |v| assert_eq!(v, *GEN_GITLAB_VARIABLE));
        mock.assert();
    }
}
