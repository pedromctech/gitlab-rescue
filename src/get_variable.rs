use crate::api_client::{api_client, DEFAULT_ENVIRONMENT};
use crate::app_error::Result;
use crate::gitlab_api::GitLabApi;
use crate::io::IO;
use crate::{app_info, app_success, extract_token, extract_url, Performable};
use clap::ArgMatches;
use std::convert::From;
use std::env;
use urlencoding::encode;

/// Arguments for `get` command
#[derive(Debug, Clone)]
pub struct GetVariableCommand {
    /// Variable name
    name: String,
    /// Project ID or URL-encoded NAMESPACE/PROJECT_NAME
    gitlab_project: Option<String>,
    /// Group ID or URL-encoded path of the group
    gitlab_group: Option<String>,
    /// Name of GitLab CI/CD environment
    environment: String,
    /// If variable is not found in defined environment (-e option), try with "All" environment.
    from_all_if_missing: bool,
    /// GitLab URL
    url: String,
    /// GitLab API Token
    token: String,
}

impl From<&ArgMatches<'_>> for GetVariableCommand {
    fn from(argm: &ArgMatches<'_>) -> Self {
        GetVariableCommand {
            name: argm.value_of("VARIABLE_NAME").unwrap().to_owned(),
            gitlab_project: argm.value_of("project").map(|p| encode(p)),
            gitlab_group: argm.value_of("project").map(|g| g.to_owned()),
            environment: argm.value_of("environment").map_or_else(|| "All".to_owned(), |v| v.to_owned()),
            from_all_if_missing: argm.is_present("from-all-if-missing"),
            url: extract_url!(argm),
            token: extract_token!(argm),
        }
    }
}

impl Performable for GetVariableCommand {
    fn get_action(self) -> IO<Result<()>> {
        IO::unit(move || {
            (match self.gitlab_project.as_ref() {
                Some(p) => {
                    app_info!("Getting variable from project {}...", p);
                    self.get_variable_from_project(p)
                }
                None => {
                    app_info!("Getting variable from group {}...", &self.gitlab_group.as_ref().unwrap());
                    self.get_variable_from_group()
                }
            })
            .map(|v| {
                app_success!("Variable {} obtained successfully", self.name);
                println!("{}", v)
            })
        })
    }
}

impl GetVariableCommand {
    /// Returns the variable value obtained from GitLab API in specified `[group]`
    fn get_variable_from_group(&self) -> Result<String> {
        api_client(&self.url, &self.token)
            .get_from_group(self.gitlab_group.as_ref().unwrap(), &self.name)
            .map(|g| g.value)
    }

    /// Returns the variable value obtained from GitLab API in specified `[project]`
    fn get_variable_from_project(&self, p: &str) -> Result<String> {
        let api = api_client(&self.url, &self.token);
        api_client(&self.url, &self.token)
            .get_from_project(p, &self.name, &self.environment)
            .map(|g| g.value)
            .or_else(|e| match self.environment != DEFAULT_ENVIRONMENT && self.from_all_if_missing {
                true => api.get_from_project(p, &self.name, DEFAULT_ENVIRONMENT).map(|g| g.value),
                _ => Err(e),
            })
    }
}
