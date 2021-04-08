use crate::{
    app_error::{AppError, Result},
    gitlab_instance::GitLabInstance,
    Command,
};
use clap::ArgMatches;
use std::{convert::TryFrom, env};

#[derive(Debug)]
pub struct GetVariable {
    /// Name of GitLab CI/CD variable
    name: String,
    /// Name of GitLab CI/CD environment
    environment: String,
    /// If variable is not found in defined environment (-e option), try with "All" environment.
    from_all_if_missing: bool,
    /// GitLab instance information
    gitlab_instance: GitLabInstance,
}

impl TryFrom<&ArgMatches<'_>> for GetVariable {
    type Error = AppError;
    fn try_from(argm: &ArgMatches<'_>) -> Result<Self> {
        Ok(GetVariable {
            name: String::from(argm.value_of("name").unwrap()),
            environment: String::from(argm.value_of("environment").unwrap()),
            from_all_if_missing: argm.is_present("from-all-if-missing"),
            gitlab_instance: GitLabInstance::new(
                match argm.value_of("url") {
                    Some(val) => val.to_string(),
                    None => env::var("GITLAB_API_URL")
                        .unwrap_or(String::from("https://gitlab.com/api/v4")),
                },
                match argm.value_of("project-id") {
                    Some(val) => val.to_string(),
                    None => env::var("GITLAB_PROJECT_ID").unwrap_or(String::new()),
                },
                match argm.value_of("token") {
                    Some(val) => val.to_string(),
                    None => env::var("GITLAB_API_TOKEN").unwrap_or(String::new()),
                },
            )?,
        })
    }
}

impl Command for GetVariable {
    fn perform(&self) -> Result<bool> {
        println!("Get variable from info: {:?}", self);
        Ok(true)
    }
}
