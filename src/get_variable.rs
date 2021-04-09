use crate::{
    app_error::{AppError, Result},
    gitlab::{GitLabApi, GitLabApiV4},
    GroupCommand, ProjectCommand,
};
use clap::ArgMatches;
use std::{convert::TryFrom, env};

#[derive(Debug)]
pub struct GetVariable {
    /// Name of GitLab CI/CD environment
    environment: String,
    /// If variable is not found in defined environment (-e option), try with "All" environment.
    from_all_if_missing: bool,
    /// GitLab API interface
    gitlab_api: GitLabApiV4,
}

impl TryFrom<&ArgMatches<'_>> for GetVariable {
    type Error = AppError;
    fn try_from(argm: &ArgMatches<'_>) -> Result<Self> {
        Ok(GetVariable {
            environment: match argm.value_of("environment") {
                Some("All") | None => "*".to_owned(),
                Some(val) => val.to_string(),
            },
            from_all_if_missing: argm.is_present("from-all-if-missing"),
            gitlab_api: GitLabApiV4::new(
                match argm.value_of("url") {
                    Some(val) => val.to_owned(),
                    None => env::var("GITLAB_URL").unwrap_or(String::from("https://gitlab.com")),
                },
                match argm.value_of("token") {
                    Some(val) => val.to_owned(),
                    None => env::var("GITLAB_API_TOKEN").unwrap_or(String::new()),
                },
            ),
        })
    }
}

impl ProjectCommand for GetVariable {
    fn perform(&self, project_id: &str, name: &str) -> Result<String> {
        match self.gitlab_api.get_from_project(project_id, name, &self.environment) {
            Ok(gitlab_variable) => Ok(gitlab_variable.value),
            Err(err) => {
                if self.from_all_if_missing {
                    Ok(self.gitlab_api.get_from_project(project_id, name, "*")?.value)
                } else {
                    Err(err)
                }
            }
        }
    }
}

impl GroupCommand for GetVariable {
    fn perform(&self, group_id: &str, name: &str) -> Result<String> {
        Ok(self.gitlab_api.get_from_group(group_id, name)?.value)
    }
}
