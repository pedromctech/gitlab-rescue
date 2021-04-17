use crate::gitlab_api::GitLabApiV4;
use crate::gitlab_api::Pagination;
use crate::IO;
use crate::{
    api_client::api_client,
    app_error::{AppError::InvalidInput, Result},
    app_info, ceil_div, extract_environment, extract_token, extract_url,
    gitlab_api::{GitLabApi, GitLabVariable},
    Performable,
};
use clap::ArgMatches;
use num_cpus;
use std::collections::HashMap;
use std::convert::From;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use urlencoding::encode;

const DEFAULT_PAGE: usize = 1;
const DEFAULT_PER_PAGE: usize = 20;
const MAX_PER_PAGE: usize = 100;

#[derive(Clone, Debug)]
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
    page: usize,
    /// Number of items to list per page
    per_page: usize,
    /// Export group variables if project belongs to a group
    with_group_vars: bool,
    /// GitLab URL
    url: String,
    /// GitLab API Token
    token: String,
    /// Parallelism
    parallel: usize,
}

impl From<&ArgMatches<'_>> for DotEnvCommand {
    fn from(argm: &ArgMatches<'_>) -> Self {
        assert!(argm.is_present("GITLAB_PROJECT"));
        DotEnvCommand {
            gitlab_project: encode(argm.value_of("GITLAB_PROJECT").unwrap()),
            environment: extract_environment!(argm),
            output: argm.value_of("output").map(|v| v.to_owned()),
            shell: if let Some("fish") = argm.value_of("shell") { ShellType::Fish } else { ShellType::Posix },
            folder: argm.value_of("folder").map(|v| v.to_owned()),
            all: argm.is_present("all"),
            page: argm
                .value_of("page")
                .map_or_else(|| DEFAULT_PAGE, |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { DEFAULT_PAGE }),
            per_page: argm
                .value_of("per-page")
                .map_or_else(|| DEFAULT_PER_PAGE, |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { DEFAULT_PER_PAGE }),
            with_group_vars: argm.is_present("with-group-vars"),
            url: extract_url!(argm),
            token: extract_token!(argm),
            parallel: argm
                .value_of("parallel")
                .map_or_else(|| num_cpus::get(), |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { num_cpus::get() }),
        }
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
    fn create_folder(folder: &str, variables: Vec<GitLabVariable>) {
        todo!();
    }
    fn get_vars(&self) -> Result<Vec<GitLabVariable>> {
        if self.all {
            self.get_all_vars()
        } else {
            self.get_paginated_vars()
        }
    }
    fn get_paginated_vars(&self) -> Result<Vec<GitLabVariable>> {
        Ok(api_client(&self.url, &self.token)
            .list_from_project(&self.gitlab_project, &self.environment, self.page, self.per_page)?
            .0)
    }
    fn get_all_vars(&self) -> Result<Vec<GitLabVariable>> {
        api_client(&self.url, &self.token)
            .list_from_project(&self.gitlab_project, &self.environment, self.page, self.per_page)
            .and_then(|(list, pag)| match list.len() >= pag.x_total {
                true => Ok(list),
                _ => Ok([list, self.get_remaining_variables(pag.x_total - pag.x_per_page)?].concat()),
            })
    }
    fn create_vars_files(&self) -> Result<Vec<GitLabVariable>> {
        todo!();
    }
    // fn print_variables_to_stdout(&self) -> Result<String> {}
    // fn print_variables_to_file(&self) -> Result<String> {}
    fn get_remaining_variables(&self, remaining: usize) -> Result<Vec<GitLabVariable>> {
        let pool = ThreadPool::new(self.parallel);
        let (tx, rx) = channel();
        let (num_requests, records_per_request) = match ceil_div!(remaining, self.parallel) <= MAX_PER_PAGE {
            true => (self.parallel, ceil_div!(remaining, self.parallel)),
            _ => (ceil_div!(remaining, MAX_PER_PAGE), MAX_PER_PAGE),
        };
        (1..=num_requests).for_each(|i| {
            let (tx, api, project, environment) = (tx.clone(), api_client(&self.url, &self.token), self.gitlab_project.clone(), self.environment.clone());
            pool.execute(move || tx.send(api.list_from_project(&project, &environment, i + 1, records_per_request)).expect(""));
        });
        rx.iter()
            .take(num_requests)
            .fold(Ok(vec![]), |a: Result<Vec<GitLabVariable>>, res| a.and_then(|acc| Ok([acc, res.map(|(l, _)| l)?].concat())))
    }
}

impl Performable for DotEnvCommand {
    fn get_action(self) -> IO<Result<()>> {
        IO::unit(move || {
            app_info!("Getting variables from project {}...", self.gitlab_project);
            self.get_vars().map(|l| println!("{:?}", l))
        })
        // IO::unit(move || {
        //     app_info!("Getting variables from project {}...", self.gitlab_project);
        //     self.get_vars();
        // Create folder with files (if there is "File" variables) INPUT folder, list variables (all or pag)
        // Generate dotenv command list (Fish OR Posix) INPUT: shell type, list variables
        // Create output file for dotenv OR print dotenv INPUT: output file, dotenv commands
        // })
        // .map(|l| println!("{}", self.folder.unwrap()));
    }
}

trait DotEnv {
    /// This generates a list of commands (dotenv script)
    fn generate_dotenv(&self, _list_of_variables: Result<Vec<GitLabVariable>>) -> Vec<String>;
}

#[derive(Clone, Copy, Debug)]
pub enum ShellType {
    Posix,
    Fish,
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
