use crate::api_client::DEFAULT_ENVIRONMENT;
use crate::api_client::MAX_PER_PAGE;
use crate::app_error::AppError;
use crate::gitlab_api::GitLabProject;
use crate::gitlab_api::GitLabVariableType;
use crate::gitlab_api::Pagination;
use crate::IO;
use crate::{
    api_client::api_client,
    app_error::{AppError::InvalidInput, Result},
    app_info, app_warning, ceil_div, extract_environment, extract_token, extract_url, floor_div,
    gitlab_api::{GitLabApi, GitLabVariable},
    Performable,
};
use ansi_term::Colour::Yellow;
use clap::ArgMatches;
use std::collections::HashMap;
use std::convert::From;
use std::env;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use urlencoding::encode;

#[derive(Clone, Copy, Debug)]
pub enum ShellType {
    Posix,
    Fish,
}

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ShellType::Posix => write!(f, "{}", Yellow.bold().paint("POSIX")),
            ShellType::Fish => write!(f, "{}", Yellow.bold().paint("FISH")),
        }
    }
}

impl ShellType {
    fn export_command(&self, variable: String, value: String) -> String {
        match self {
            ShellType::Posix => format!("export {}=\"{}\"", variable, value),
            ShellType::Fish => format!("set -gx {} \"{}\"", variable, value),
        }
    }
}

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
            environment: get_env_from_args(argm),
            output_file: argm.value_of("output").map(|v| v.to_owned()),
            shell: if let Some("fish") = argm.value_of("shell") { ShellType::Fish } else { ShellType::Posix },
            folder: argm.value_of("folder").map_or_else(|| format!(".env.{}", get_env_from_args(argm)), |v| v.to_owned()),
            per_page: numeric_param_from_args(argm, "per-page", 50),
            with_group_vars: argm.is_present("with-group-vars"),
            parallel: numeric_param_from_args(argm, "parallel", num_cpus::get()),
        }
    }
}

fn get_env_from_args(args: &clap::ArgMatches) -> String {
    args.value_of("environment").map_or_else(|| DEFAULT_ENVIRONMENT.to_owned(), |v| v.to_owned())
}

fn numeric_param_from_args(argm: &clap::ArgMatches, param: &str, default: usize) -> usize {
    argm.value_of(param)
        .map_or_else(|| default, |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { default })
}

impl Performable for DotEnvCommand {
    fn get_action(mut self) -> IO<Result<()>> {
        IO::unit(move || {
            app_info!("Getting variables from project {}...", self.gitlab_project.name);
            self.gitlab_project.variables.extend(get_vars(&self)?);
            Ok(Rc::clone(&Rc::new(self)))
        })
        .flat_map(|res: Result<Rc<DotEnvCommand>>| {
            app_info!("Creating files for variables of type File...");
            match res {
                Ok(ref cmd) => create_files(cmd.folder.clone(), cmd.gitlab_project.variables.clone()).map(|_| res),
                Err(e) => IO::unit(|| Err(e)),
            }
        })
        .map(|res: Result<Rc<DotEnvCommand>>| {
            app_info!("Creating dotenv command list...");
            res.map(
                |cmd| match (generate_commands(cmd.shell, cmd.gitlab_project.variables.clone(), cmd.folder.clone()), &cmd.output_file) {
                    (list, Some(f)) => write_file(f.to_string(), bytes_from_list(&list)).unwrap_or(list.into_iter().for_each(|c| println!("{}", c))),
                    (list, None) => list.into_iter().for_each(|c| println!("{}", c)),
                },
            )
        })
    }
}

fn bytes_from_list(list: &Vec<String>) -> &'static [u8] {
    todo!();
}

fn get_vars(cmd: &DotEnvCommand) -> Result<Vec<GitLabVariable>> {
    api_client(&cmd.gitlab_project.url, &cmd.gitlab_project.token)
        .list_from_project(&cmd.gitlab_project.name, 1, cmd.per_page)
        .and_then(|(list, pag)| match list.len() < pag.x_total {
            true => Ok([
                list,
                get_remaining_vars(&cmd.gitlab_project, num_requests(pag.x_total, cmd.per_page), cmd.per_page, cmd.parallel)?,
            ]
            .concat()),
            _ => Ok(list),
        })
        .map(|list| {
            list.into_iter()
                .filter(|v| v.environment_scope == DEFAULT_ENVIRONMENT || v.environment_scope == cmd.environment)
                .collect()
        })
}

fn get_remaining_vars(project: &GitLabProject, requests: usize, per_request: usize, parallel: usize) -> Result<Vec<GitLabVariable>> {
    let pool = ThreadPool::new(parallel);
    let (tx, rx) = channel();
    (0..requests)
        .fold(rx, |acc, i| {
            let (tx, project) = (tx.clone(), project.clone());
            pool.execute(move || {
                tx.send(api_client(&project.url, &project.token).list_from_project(&project.name, i + 2, per_request))
                    .expect("Thread Error")
            });
            acc
        })
        .iter()
        .take(requests)
        .fold(Ok(vec![]), |a: Result<Vec<GitLabVariable>>, res| a.and_then(|acc| Ok([acc, res.map(|(l, _)| l)?].concat())))
}

/// Calculate number of needed requests to get remaining variables. Number of requests is 1 if number of
/// remaining values are less or equal to maximum number of records per request.
///
/// # Arguments
///
/// * `total` - Number of records
/// * `per_request` - Maximum number of records to get per request
///
fn num_requests(total: usize, per_request: usize) -> usize {
    match total - per_request <= per_request {
        true => 1,
        _ => ((total - per_request) as f64 / per_request as f64).ceil() as usize,
    }
}

fn write_file(file: String, content: &[u8]) -> std::io::Result<()> {
    File::create(&file)
        .map(|mut f| f.write_all(content))
        .map(|_| app_info!("File {} created successfully", file))
}

fn create_files(folder: String, variables: Vec<GitLabVariable>) -> IO<std::result::Result<(), std::io::Error>> {
    IO::unit(move || {
        fs::create_dir_all(folder.clone()).and_then(|_| {
            variables
                .iter()
                .filter(|v| matches!(v.variable_type, GitLabVariableType::File))
                .map(|v| (format!("{}/{}.var", folder, v.key), v.value.as_bytes()))
                .fold(Ok(()), |acc, (file, content)| acc.and(write_file(file, content)))
        })
    })
}

fn generate_commands(shell: ShellType, variables: Vec<GitLabVariable>, folder: String) -> Vec<String> {
    variables.iter().fold(vec![], |mut acc, v| {
        acc.push(shell.export_command(
            v.key.clone(),
            match v.variable_type {
                GitLabVariableType::File => format!("{}/{}.var", folder, v.key),
                GitLabVariableType::EnvVar => v.value.clone(),
            },
        ));
        acc
    })
}
