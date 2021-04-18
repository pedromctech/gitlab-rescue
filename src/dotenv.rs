use crate::app_error::AppError;
use crate::gitlab_api::GitLabApiV4;
use crate::gitlab_api::GitLabProject;
use crate::gitlab_api::Pagination;
use ansi_term::Colour::Yellow;
use crate::IO;
use crate::{
    api_client::api_client,
    app_error::{AppError::InvalidInput, Result},
    app_info, app_warning, ceil_div, extract_environment, extract_token, extract_url, floor_div,
    gitlab_api::{GitLabApi, GitLabVariable},
    Performable,
};
use clap::ArgMatches;
use num_cpus;
use std::collections::HashMap;
use std::convert::From;
use std::env;
use std::fs;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use urlencoding::encode;

const DEFAULT_PAGE: usize = 1;
const DEFAULT_PER_PAGE: usize = 20;
const MAX_PER_PAGE: usize = 100;
const GITLAB_ENV_ALL: &str = "*";

#[derive(Clone, Debug)]
pub struct DotEnvCommand {
    /// Project ID or URL-encoded NAMESPACE/PROJECT_NAME
    gitlab_project: GitLabProject,
    /// Name of GitLab CI/CD environment
    environment: String,
    /// Write dotenv to a file instead of stdout
    output_file: Option<String>,
    /// Generate dotenv for this shell type.
    shell: ShellType,
    /// Path where variables with type "File" will be stored.
    folder: String,
    /// List all varibles (without this option, only 20 variables are showed).
    all: bool,
    /// Page number
    page: usize,
    /// Number of items to list per page
    per_page: usize,
    /// Export group variables if project belongs to a group
    with_group_vars: bool,
    /// Parallelism
    parallel: usize,
}

impl From<&ArgMatches<'_>> for DotEnvCommand {
    fn from(argm: &ArgMatches<'_>) -> Self {
        assert!(argm.is_present("GITLAB_PROJECT"));
        DotEnvCommand {
            gitlab_project: GitLabProject {
                name: encode(argm.value_of("GITLAB_PROJECT").unwrap()),
                url: extract_url!(argm),
                token: extract_token!(argm),
                variables: vec![],
            },
            environment: extract_environment!(argm),
            output_file: argm.value_of("output").map(|v| v.to_owned()),
            shell: if let Some("fish") = argm.value_of("shell") { ShellType::Fish } else { ShellType::Posix },
            folder: argm.value_of("folder").map_or_else(|| format!(".env.{}", extract_environment!(argm)), |v| v.to_owned()),
            all: argm.is_present("all"),
            page: argm
                .value_of("page")
                .map_or_else(|| DEFAULT_PAGE, |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { DEFAULT_PAGE }),
            per_page: argm
                .value_of("per-page")
                .map_or_else(|| DEFAULT_PER_PAGE, |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { DEFAULT_PER_PAGE }),
            with_group_vars: argm.is_present("with-group-vars"),
            parallel: argm
                .value_of("parallel")
                .map_or_else(|| num_cpus::get(), |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { num_cpus::get() }),
        }
    }
}

fn get_vars(cmd: &DotEnvCommand) -> Result<Vec<GitLabVariable>> {
    api_client(&cmd.gitlab_project.url, &cmd.gitlab_project.token)
        .list_from_project(&cmd.gitlab_project.name, if cmd.all { 1 } else { cmd.page }, if cmd.all { MAX_PER_PAGE } else { cmd.per_page })
        .and_then(|(list, pag)| match cmd.all && list.len() < pag.x_total {
            true => Ok([list, get_remaining_variables(&cmd.gitlab_project, num_requests(pag.x_total - pag.x_per_page), cmd.parallel)?].concat()),
            _ => Ok(list),
        })
        .map(|l| l.into_iter().filter(|v| v.environment_scope == GITLAB_ENV_ALL || v.environment_scope == cmd.environment).collect())
}

fn get_remaining_variables(project: &GitLabProject, requests: usize, parallel: usize) -> Result<Vec<GitLabVariable>> {
    let pool = ThreadPool::new(parallel);
    let (tx, rx) = channel();
    (0..requests)
        .fold(rx, |acc, i| {
            let (tx, project) = (tx.clone(), project.clone());
            pool.execute(move || tx.send(api_client(&project.url, &project.token).list_from_project(&project.name, i + 2, MAX_PER_PAGE)).expect(""));
            acc
        })
        .iter()
        .take(requests)
        .fold(Ok(vec![]), |a: Result<Vec<GitLabVariable>>, res| a.and_then(|acc| Ok([acc, res.map(|(l, _)| l)?].concat())))
}

/// Calculate number of needed requests to get remaining variables
///
/// # Arguments
///
/// * `r` - Remaining records
///
fn num_requests(r: usize) -> usize {
    match r > MAX_PER_PAGE {
        true => (r as f64 / MAX_PER_PAGE as f64).ceil() as usize,
        _ => 1,
    }
}

fn create_files(folder: String, variables: Vec<GitLabVariable>) -> IO<std::result::Result<(), std::io::Error>> {
    IO::unit(move || {
        fs::create_dir_all(folder.clone()).and_then(|_| {
            variables
                .iter()
                .filter(|v| v.variable_type == "file")
                .map(|v| (format!("{}/{}.var", folder, v.key), v.value.as_bytes()))
                .fold(Ok(()), |acc, (file, contents)| {
                    acc.and(File::create(file.clone()).map(|mut f| f.write_all(contents)).map(|_| app_info!("File {} created successfully", file)))
                })
        })
    })
}

impl Performable for DotEnvCommand {
    fn get_action(mut self) -> IO<Result<()>> {
        let unit = IO::unit(move || {
            app_info!("Getting variables from project {}...", self.gitlab_project.name);
            self.gitlab_project.variables.extend(get_vars(&self)?);
            Ok(self)
        })
        .flat_map(|res: Result<DotEnvCommand>| {
            app_info!("Creating files for variables of type File...");
            res.clone()
                .map_or_else(|e| IO::unit(|| Err(e)), |cmd| create_files(cmd.folder, cmd.gitlab_project.variables).map(|_| res))
        })
        .map(|res: Result<DotEnvCommand>| {
            res.map(|cmd| {
                app_info!("Creating dotenv command list for {} shell...", cmd.shell);
            })
        });
        // IO::unit(move || {
        //     app_info!("Getting variables from project {}...", self.gitlab_project);
        //     self.get_vars();
        // Create folder with files (if there is "File" variables) INPUT folder, list variables (all or pag)
        // Generate dotenv command list (Fish OR Posix) INPUT: shell type, list variables
        // Create output file for dotenv OR print dotenv INPUT: output file, dotenv commands
        // })
        // .map(|l| println!("{}", self.folder.unwrap()));
        // todo!();
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

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ShellType::Posix => write!(f, "{}", Yellow.bold().paint("POSIX")),
            ShellType::Fish => write!(f, "{}", Yellow.bold().paint("Fish")),
        }
    }
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
