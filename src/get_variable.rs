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
#[derive(Clone, Debug, PartialEq)]
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

impl Performable for GetVariableCommand {
    fn get_action(self) -> IO<Result<()>> {
        IO::unit(move || {
            app_info!("Getting variable {} from GitLab API...", &self.name);
            match self.gitlab_project.as_ref() {
                Some(_) => Ok(get_variable_from_project(&self)?),
                None => Ok(get_variable_from_group(&self)?),
            }
            .map(|v| {
                app_success!("Variable {} obtained successfully", self.name);
                println!("{}", v)
            })
        })
    }
}

impl From<&ArgMatches<'_>> for GetVariableCommand {
    fn from(argm: &ArgMatches<'_>) -> Self {
        GetVariableCommand {
            name: argm.value_of("VARIABLE_NAME").unwrap().to_owned(),
            gitlab_project: argm.value_of("project").map(|p| encode(p)),
            gitlab_group: argm.value_of("group").map(|g| g.to_owned()),
            environment: argm.value_of("environment").map_or_else(|| "All".to_owned(), |v| v.to_owned()),
            from_all_if_missing: argm.is_present("from-all-if-missing"),
            url: extract_url!(argm),
            token: extract_token!(argm),
        }
    }
}

/// Returns the variable value obtained from GitLab API in specified `[group]`
fn get_variable_from_group(cmd: &GetVariableCommand) -> Result<String> {
    api_client(&cmd.url, &cmd.token)
        .get_from_group(cmd.gitlab_group.as_ref().unwrap(), &cmd.name)
        .map(|g| g.value)
}

/// Returns the variable value obtained from GitLab API in specified `[project]`
fn get_variable_from_project(cmd: &GetVariableCommand) -> Result<String> {
    api_client(&cmd.url, &cmd.token)
        .get_from_project(cmd.gitlab_project.as_ref().unwrap(), &cmd.name, &cmd.environment)
        .map(|g| g.value)
        .or_else(|e| match cmd.environment != DEFAULT_ENVIRONMENT && cmd.from_all_if_missing {
            true => api_client(&cmd.url, &cmd.token)
                .get_from_project(cmd.gitlab_project.as_ref().unwrap(), &cmd.name, DEFAULT_ENVIRONMENT)
                .map(|g| g.value),
            _ => Err(e),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clap_app::app;
    use crate::gitlab_api::tests::*;
    use httpmock::MockServer;

    fn gen_getvar_command(url: &str, from_all_if_missing: bool, gitlab_project: Option<String>) -> GetVariableCommand {
        GetVariableCommand {
            name: GEN_NAME.clone(),
            gitlab_group: if gitlab_project.as_ref().clone().is_some() {
                None
            } else {
                Some(GEN_GROUP_NAME.clone())
            },
            gitlab_project,
            environment: GEN_ENVIRONMENT.clone(),
            from_all_if_missing,
            url: url.to_owned(),
            token: GEN_TOKEN.clone(),
        }
    }

    #[test]
    fn get_should_create_variable_cmd_from_cli_args() {
        app()
            .get_matches_from(vec![
                "gitlab-rescue",
                "get",
                &GEN_NAME,
                &format!("-e={}", *GEN_ENVIRONMENT),
                &format!("-p={}", *GEN_PROJECT_NAME),
                "--from-all-if-missing",
                "-u=gitlab.com",
                &format!("-t={}", *GEN_TOKEN),
            ])
            .subcommand_matches("get")
            .map(|a| assert_eq!(GetVariableCommand::from(a), gen_getvar_command("gitlab.com", true, Some(GEN_PROJECT_NAME.to_owned()))))
            .unwrap();
    }

    #[test]
    fn test_should_get_variable_from_group() {
        let server = MockServer::start();
        let mock = server.mock(httpmock_group_variable());
        assert!(get_variable_from_group(&gen_getvar_command(&server.base_url(), false, None)).map_or_else(|_| false, |v| v == GEN_GITLAB_VARIABLE.value));
        mock.assert();
    }

    #[test]
    fn test_should_get_variable_from_project() {
        let server = MockServer::start();
        let mut mock = server.mock(httpmock_project_variable(GEN_GITLAB_VARIABLE.environment_scope.clone()));
        assert!(
            get_variable_from_project(&gen_getvar_command(&server.base_url(), false, Some(GEN_PROJECT_NAME.to_owned()))).map_or_else(|_| false, |v| v == GEN_GITLAB_VARIABLE.value)
        );
        mock.assert();
        mock.delete();
        let mock = server.mock(httpmock_project_variable("*".to_owned()));
        assert!(get_variable_from_project(&gen_getvar_command(&server.base_url(), true, Some(GEN_PROJECT_NAME.to_owned())))
            .map_or_else(|_| false, |v| v == GEN_GITLAB_VARIABLE_ALL.value));
        mock.assert();
    }
}
