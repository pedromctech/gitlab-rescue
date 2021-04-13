use crate::IO;
use crate::{
    api_client::api_client,
    app_error::{AppError::InvalidInput, Result},
    extract_environment, extract_token, extract_url,
    gitlab_api::{GitLabApi, GitLabVariable},
    numeric_arg, Performable,
};
use clap::ArgMatches;
use num_cpus;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::{convert::From, env, path::Path};
use urlencoding::encode;

#[derive(Debug)]
pub struct DotEnvCommand {
    /// Project ID or URL-encoded NAMESPACE/PROJECT_NAME
    gitlab_project: String,
    /// Name of GitLab CI/CD environment
    environment: String,
    /// Write dotenv to a file instead of stdout
    output: Option<String>,
    /// Generate dotenv for this shell type.
    shell: ShellType,
    /// Path where variables with type "File" will be stored.
    folder: Option<String>,
    /// List all varibles (without this option, only 20 variables are showed).
    all: bool,
    /// Page number
    page: u32,
    /// Number of items to list per page
    per_page: u32,
    /// Export group variables if project belongs to a group
    with_group_vars: bool,
    /// GitLab URL
    url: String,
    /// GitLab API Token
    token: String,
}

#[derive(Debug)]
pub enum ShellType {
    Posix,
    Fish,
}

impl From<&ArgMatches<'_>> for DotEnvCommand {
    fn from(argm: &ArgMatches<'_>) -> Self {
        assert!(argm.is_present("GITLAB_PROJECT"));
        DotEnvCommand {
            gitlab_project: encode(argm.value_of("GITLAB_PROJECT").unwrap()),
            environment: extract_environment!(argm),
            output: if let Some(v) = argm.value_of("output") { Some(v.to_owned()) } else { None },
            shell: if let Some("fish") = argm.value_of("shell") { ShellType::Fish } else { ShellType::Posix },
            folder: if let Some(v) = argm.value_of("folder") { Some(v.to_owned()) } else { None },
            all: argm.is_present("all"),
            page: numeric_arg!(argm, "page", 1),
            per_page: numeric_arg!(argm, "per-page", 20),
            with_group_vars: argm.is_present("with-group-vars"),
            url: extract_url!(argm),
            token: extract_token!(argm),
        }
    }
}

impl Performable for DotEnvCommand {
    fn get_action(self) -> IO<Result<()>> {
        IO::unit(move || {
            let api = api_client("v4", &self.url, &self.token);
            // Get list (all vars OR paginated vars)
            // Generate dotenv command list (Fish OR Posix)
            // Create folder with files (if there is "File" variables)
            // Create output file for dotenv OR print dotenv
        });
        todo!();
    }
}

impl DotEnvCommand {
    fn create_file(var: GitLabVariable) -> GitLabVariable {
        if var.variable_type == "file" {
            println!("Create folder if not exists. If Error, use default folder");
            println!("Create file");
        }
        var
    }
    fn get_vars(&self) -> Result<Vec<GitLabVariable>> {
        if self.all { self.get_all_vars() } else { self.get_paginated_vars() }
    }
    fn get_paginated_vars(&self) -> Result<Vec<GitLabVariable>> {
        let api_client = api_client("v4", &self.url, &self.token);
        let res = api_client.list_from_project(&self.gitlab_project, &self.environment, self.page, self.per_page)?.0;
        println!("{:?}", res);
        Ok(res)
    }
    fn get_all_vars(&self) -> Result<Vec<GitLabVariable>> {
        let api_client = api_client("v4", &self.url, &self.token);
        todo!()
    }
    fn create_vars_files(&self) -> Result<Vec<GitLabVariable>> {
        todo!();
    }
    // fn print_variables_to_stdout(&self) -> Result<String> {}
    // fn print_variables_to_file(&self) -> Result<String> {}
}

trait DotEnv {
    /// This generates a list of commands (dotenv script)
    fn generate_dotenv(&self, _list_of_variables: Result<Vec<GitLabVariable>>) -> Vec<String>;
}

impl DotEnv for ShellType {
    fn generate_dotenv(&self, variables: Result<Vec<GitLabVariable>>) -> Vec<String> {
        match self {
            ShellType::Posix => self.generate_posix_dotenv(variables),
            ShellType::Fish => self.generate_fish_dotenv(variables),
        }
    }
}

impl ShellType {
    fn generate_posix_dotenv(&self, _variables: Result<Vec<GitLabVariable>>) -> Vec<String> {
        vec!["export 1".to_owned(), "export 2".to_owned(), "export 3".to_owned()]
    }

    fn generate_fish_dotenv(&self, _variables: Result<Vec<GitLabVariable>>) -> Vec<String> {
        vec!["export 1".to_owned(), "export 2".to_owned(), "export 3".to_owned()]
    }
}
