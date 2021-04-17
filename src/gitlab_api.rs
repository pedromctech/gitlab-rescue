extern crate proc_macro;
use crate::app_error::{AppError::Cli, Result};
use reqwest::blocking::{Client as BlockingClient, Response as BlockingResponse};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GitLabVariable {
    /// The type of a variable. Available types are: env_var and file
    pub variable_type: String,
    /// The key of the variable
    key: String,
    /// The value of a variable
    pub value: String,
    /// Variable's environment
    pub environment_scope: String,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    /// The index of the next page.
    x_next_page: usize,
    /// The index of the current page (starting at 1).
    x_page: usize,
    /// The number of items per page.
    pub x_per_page: usize,
    /// The total number of items.
    pub x_total: usize,
    /// The total number of pages.
    x_total_pages: usize,
}

pub trait GitLabApi {
    fn new(gitlab_api_url: String, gitlab_token: String) -> Self;
    /// Get a variable value from a specific GitLab project
    fn get_from_project(&self, project: &str, name: &str, environment: &str) -> Result<GitLabVariable>;
    /// Get a variable value from a specific GitLab group
    fn get_from_group(&self, group: &str, name: &str) -> Result<GitLabVariable>;
    /// List variables from a specific GitLab project
    fn list_from_project(&self, project: &str, environment: &str, page: usize, per_page: usize) -> Result<(Vec<GitLabVariable>, Pagination)>;
}

#[derive(Clone, Debug)]
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

    fn list_from_project(&self, project: &str, environment: &str, page: usize, per_page: usize) -> Result<(Vec<GitLabVariable>, Pagination)> {
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
            x_next_page: self.get_pagination_header(&res, "x-next-page")?,
            x_page: self.get_pagination_header(&res, "x-page")?,
            x_per_page: self.get_pagination_header(&res, "x-per-page")?,
            x_total: self.get_pagination_header(&res, "x-total")?,
            x_total_pages: self.get_pagination_header(&res, "x-total-pages")?,
        };
        Ok((res.json()?, pag))
    }

    fn get_pagination_header(&self, res: &BlockingResponse, header: &str) -> Result<usize> {
        match res.headers().get(header) {
            Some(h) if h.to_str()?.is_empty() => Ok(0),
            Some(h) if h.to_str()?.parse::<usize>().is_ok() => Ok(h.to_str()?.parse::<usize>().unwrap()),
            _ => Err(Cli(format!("Header {} not valid in GitLab response", header))),
        }
    }
}
