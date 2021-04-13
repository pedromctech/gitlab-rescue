use crate::api_client::api_client;
use crate::app_error::Result;
use crate::gitlab_api::GitLabApi;
use crate::io::IO;
use crate::Performable;
use crate::{app_info, app_success, extract_environment, extract_token, extract_url};
use clap::ArgMatches;
use std::convert::From;
use std::env;
use urlencoding::encode;

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
            gitlab_project: if let Some(v) = argm.value_of("project") { Some(encode(v)) } else { None },
            gitlab_group: if let Some(v) = argm.value_of("group") { Some(v.to_owned()) } else { None },
            environment: extract_environment!(argm),
            from_all_if_missing: argm.is_present("from-all-if-missing"),
            url: extract_url!(argm),
            token: extract_token!(argm),
        }
    }
}

impl GetVariableCommand {
    fn get_variable_from_group(&self, api: &impl GitLabApi) -> Result<String> {
        app_info!("Getting variable from group {}...", &self.gitlab_group.as_ref().unwrap());
        api.get_from_group(self.gitlab_group.as_ref().unwrap(), &self.name).map(|g| g.value)
    }

    fn get_variable_from_project(&self, api: &impl GitLabApi, p: &str) -> Result<String> {
        app_info!("Getting variable from project {}...", p);
        api.get_from_project(p, &self.name, &self.environment).and_then(|var| Ok(var.value)).or_else(|err| {
            if self.environment != "*" && self.from_all_if_missing {
                app_info!("Getting variable from \"All\" environment...");
                api.get_from_project(p, &self.name, "*").and_then(|g| Ok(g.value))
            } else {
                Err(err)
            }
        })
    }

    fn print_result(&self, value: String) {
        app_success!("Variable {} obtained successfully", &self.name);
        println!("{}", value);
    }
}

impl Performable for GetVariableCommand {
    fn get_action(self) -> IO<Result<()>> {
        IO::unit(move || {
            let api = api_client("v4", &self.url, &self.token);
            self.gitlab_project
                .as_ref()
                .map_or_else(|| self.get_variable_from_group(&api), |p| self.get_variable_from_project(&api, p))
                .map(|v| self.print_result(v))
        })
    }
}
