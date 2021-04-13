extern crate proc_macro;
use crate::app_error::Result;
use reqwest::{blocking::Client as BlockingClient};
use serde::Deserialize;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

/// Extract GITLAB_API_TOKEN from clap args
macro_rules! get_header_as_u32 {
    ($response:expr, $header:expr) => {
        match $response.headers().get($header) {
            Some(h) => match h.to_str().ok() {
                Some(val) => val.parse::<u32>().ok(),
                None => None,
            },
            None => None,
        }
    };
}

#[derive(Debug, Deserialize)]
pub struct GitLabVariable {
    /// The type of a variable. Available types are: env_var and file
    pub variable_type: String,
    /// The key of the variable
    key: String,
    /// The value of a variable
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    /// The index of the next page.
    x_next_page: Option<u32>,
    /// The index of the current page (starting at 1).
    x_page: Option<u32>,
    /// The number of items per page.
    x_per_page: Option<u32>,
    /// The index of the previous page.
    x_prev_page: Option<u32>,
    /// The total number of items.
    x_total: Option<u32>,
    /// The total number of pages.
    x_total_pages: Option<u32>,
}

pub trait GitLabApi {
    fn new(gitlab_api_url: String, gitlab_token: String) -> Self;
    /// Get a variable value from a specific GitLab project
    fn get_from_project(&self, project: &str, name: &str, environment: &str) -> Result<GitLabVariable>;
    /// Get a variable value from a specific GitLab group
    fn get_from_group(&self, group: &str, name: &str) -> Result<GitLabVariable>;
    /// List variables from a specific GitLab project
    fn list_from_project(&self, project: &str, environment: &str, page: u32, per_page: u32) -> Result<(Vec<GitLabVariable>, Pagination)>;
}

#[derive(Debug)]
pub struct GitLabApiV4 {
    url: String,
    token: String,
}

impl<'a> GitLabApi for GitLabApiV4 {
    fn new(url: String, token: String) -> Self {
        GitLabApiV4 {
            url: format!("{}/api/v4", url),
            token: token,
        }
    }

    fn get_from_project(&self, project: &str, name: &str, environment: &str) -> Result<GitLabVariable> {
        self.get(&format!("projects/{}/variables/{}?filter[environment_scope]={}", project, name, environment))
    }

    fn get_from_group(&self, group: &str, name: &str) -> Result<GitLabVariable> {
        self.get(&format!("groups/{}/variables/{}", group, name))
    }

    fn list_from_project(&self, project: &str, environment: &str, page: u32, per_page: u32) -> Result<(Vec<GitLabVariable>, Pagination)> {
        self.list(&format!("projects/{}/variables?filter[environment_scope]={}&page={}&per_page={}", project, environment, page, per_page))
    }
}

impl GitLabApiV4 {
    fn get(&self, endpoint: &str) -> Result<GitLabVariable> {
        Ok(BlockingClient::builder()
            .build()?
            .get(format!("{}/{}", self.url, endpoint))
            .header("PRIVATE-TOKEN", &self.token)
            .send()?
            .error_for_status()?
            .json()?)
    }

    fn list(&self, endpoint: &str) -> Result<(Vec<GitLabVariable>, Pagination)> {
        let res = BlockingClient::builder()
            .build()?
            .get(format!("{}/{}", self.url, endpoint))
            .header("PRIVATE-TOKEN", &self.token)
            .send()?
            .error_for_status()?;
        let pag = Pagination {
            x_next_page: get_header_as_u32!(res, "x-next-page"),
            x_page: get_header_as_u32!(res, "x-page"),
            x_per_page: get_header_as_u32!(res, "x-per-page"),
            x_prev_page: get_header_as_u32!(res, "x-prev-page"),
            x_total: get_header_as_u32!(res, "x-total"),
            x_total_pages: get_header_as_u32!(res, "x-total-pages"),
        };
        Ok((res.json()?, pag))
    }
}
