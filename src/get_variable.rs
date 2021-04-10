use crate::{api_client::api_client, app_error::Result, gitlab::GitLabApi, Performable};
use clap::ArgMatches;
use std::{convert::From, env};
use urlencoding::encode;

#[derive(Debug)]
pub struct GetVariable {
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

impl From<&ArgMatches<'_>> for GetVariable {
    fn from(argm: &ArgMatches<'_>) -> Self {
        GetVariable {
            gitlab_project: if let Some(v) = argm.value_of("project") { Some(encode(v)) } else { None },
            gitlab_group: if let Some(v) = argm.value_of("group") { Some(v.to_owned()) } else { None },
            environment: match argm.value_of("environment") {
                Some("All") | None => "*".to_owned(),
                Some(val) => val.to_string(),
            },
            from_all_if_missing: argm.is_present("from-all-if-missing"),
            url: match argm.value_of("url") {
                Some(s) => s.to_owned(),
                None => env::var("GITLAB_URL").unwrap_or(String::from("https://gitlab.com")),
            },
            token: match argm.value_of("token") {
                Some(s) => s.to_owned(),
                None => env::var("GITLAB_API_TOKEN").unwrap_or(String::new()),
            },
        }
    }
}

impl Performable for GetVariable {
    fn perform(&self, name: &str) -> Result<String> {
        assert_ne!(self.gitlab_project.as_ref().xor(self.gitlab_group.as_ref()), None);
        let api_client = api_client("v4", &self.url, &self.token);
        match &self.gitlab_project {
            Some(p) => match api_client.get_from_project(&p, name, &self.environment) {
                Ok(v) => Ok(v.value),
                Err(err) => {
                    if self.from_all_if_missing {
                        Ok(api_client.get_from_project(&self.gitlab_project.as_ref().unwrap(), name, "*")?.value)
                    } else {
                        Err(err)
                    }
                }
            },
            None => Ok(api_client.get_from_group(self.gitlab_group.as_ref().unwrap(), name)?.value),
        }
    }
}
