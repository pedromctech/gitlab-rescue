use crate::api_client::{api_client, DEFAULT_ENVIRONMENT};
use crate::app_error::{AppError, Result};
use crate::gitlab_api::{GitLabApi, GitLabProject, GitLabVariable, GitLabVariableType};
use crate::shell_types::ShellType;
use crate::IO;
use crate::{app_info, app_warning, extract_token, extract_url, Performable};
use clap::ArgMatches;
use std::convert::From;
use std::env;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::{fs, fs::File};
use threadpool::ThreadPool;
use urlencoding::encode;

/// Arguments for `dotenv` command
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
    /// GitLab instance URL
    url: String,
    /// Token to connect to GitLab instance API
    token: String,
}

impl From<&ArgMatches<'_>> for DotEnvCommand {
    fn from(argm: &ArgMatches<'_>) -> Self {
        assert!(argm.is_present("GITLAB_PROJECT"));
        DotEnvCommand {
            gitlab_project: GitLabProject {
                name: encode(argm.value_of("GITLAB_PROJECT").unwrap()),
                variables: vec![],
            },
            environment: get_env_from_args(argm),
            output_file: argm.value_of("output").map(|v| v.to_owned()),
            shell: if let Some("fish") = argm.value_of("shell") { ShellType::Fish } else { ShellType::Posix },
            folder: argm.value_of("folder").map_or_else(|| format!(".env.{}", get_env_from_args(argm)), |v| v.to_owned()),
            per_page: numeric_param_from_args(argm, "per-page", 50),
            with_group_vars: argm.is_present("with-group-vars"),
            parallel: numeric_param_from_args(argm, "parallel", num_cpus::get()),
            url: extract_url!(argm),
            token: extract_token!(argm),
        }
    }
}

impl Performable for DotEnvCommand {
    fn get_action(mut self) -> IO<Result<()>> {
        IO::unit(move || {
            app_info!("Getting variables from project {}...", self.gitlab_project.name);
            self.gitlab_project.variables.extend(self.get_vars()?);
            Ok(Rc::clone(&Rc::new(self)))
        })
        .flat_map(|res: Result<Rc<DotEnvCommand>>| {
            app_info!("Creating files for variables of type File...");
            match res {
                Ok(ref cmd) => create_files(cmd.folder.clone(), cmd.gitlab_project.variables.clone()).map(|r| r.and_then(|_| res).or_else(|e| Err(e))),
                Err(e) => IO::unit(|| Err(e)),
            }
        })
        .flat_map(|res: Result<Rc<DotEnvCommand>>| match res {
            Ok(ref cmd) => {
                app_info!("Creating dotenv command list...");
                match (generate_commands(cmd.shell, &cmd.gitlab_project.variables, &cmd.folder), &cmd.output_file) {
                    (list, Some(f)) => write_file(f.to_string(), list.join("\n").as_bytes().to_vec()).map(|r| {
                        r.or_else(|e| {
                            app_warning!("Error creating output file ({}). Printing commands in STDOUT...", e);
                            Ok(list.into_iter().for_each(|c| println!("{}", c)))
                        })
                    }),
                    (list, None) => IO::unit(|| Ok(list.into_iter().for_each(|c| println!("{}", c)))),
                }
            }
            Err(e) => IO::unit(|| Err(e)),
        })
    }
}

impl DotEnvCommand {
    /// Returns a list with all `[project]` variables from GitLab API.
    fn get_vars(&self) -> Result<Vec<GitLabVariable>> {
        api_client(&self.url, &self.token)
            .list_from_project(&self.gitlab_project.name, 1, self.per_page)
            .and_then(|(list, pag)| match list.len() < pag.x_total {
                true => Ok([list, self.get_remaining_vars(num_requests(pag.x_total, self.per_page))?].concat()),
                _ => Ok(list),
            })
            .map(|list| {
                list.into_iter()
                    .filter(|v| v.environment_scope == DEFAULT_ENVIRONMENT || v.environment_scope == self.environment)
                    .collect()
            })
    }

    /// Returns a list with remaining variables that could not be obtained in the first request
    ///
    /// # Arguments
    ///
    /// * `requests` - Number of requests to make to obtain the remaining variables
    ///
    fn get_remaining_vars(&self, requests: usize) -> Result<Vec<GitLabVariable>> {
        let pool = ThreadPool::new(self.parallel);
        let (tx, rx) = channel();
        (0..requests)
            .fold(rx, |acc, i| {
                let (tx, project, url, token, pp) = (tx.clone(), self.gitlab_project.clone(), self.url.clone(), self.token.clone(), self.per_page);
                pool.execute(move || tx.send(api_client(&url, &token).list_from_project(&project.name, i + 2, pp)).expect("Thread Error"));
                acc
            })
            .iter()
            .take(requests)
            .fold(Ok(vec![]), |a: Result<Vec<GitLabVariable>>, res| a.and_then(|acc| Ok([acc, res.map(|(l, _)| l)?].concat())))
    }
}

/// Returns environment name from [ArgMatches](struct@clap::ArgMatches) object
///
/// # Arguments
///
/// * `args` - Reference of [ArgMatches](ArgMatches) object
///
fn get_env_from_args(args: &clap::ArgMatches) -> String {
    args.value_of("environment").map_or_else(|| DEFAULT_ENVIRONMENT.to_owned(), |v| v.to_owned())
}

/// Returns param of type `usize` from [ArgMatches](struct@clap::ArgMatches) object
///
/// # Arguments
///
/// * `args`    - Reference of [ArgMatches](ArgMatches) object
/// * `param`   - Name of parameter to extract
/// * `default` - If parameter is not present, return `default`
///
fn numeric_param_from_args(argm: &clap::ArgMatches, param: &str, default: usize) -> usize {
    argm.value_of(param)
        .map_or_else(|| default, |v| if v.parse::<usize>().is_ok() { v.parse::<usize>().unwrap() } else { default })
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

/// Returns an IO object that creates a file and write content to it.
///
/// # Example
///
/// ```rust
/// let content = "This is the content";
/// let action = write_file("/home/me/my-file.txt".to_owned(), content.as_bytes().to_vec()); // Here nothing happens
/// action.apply(); // This creates the file
/// ```
///
fn write_file(file: String, content: Vec<u8>) -> IO<Result<()>> {
    IO::unit(move || {
        File::create(&file)
            .map(|mut f| f.write_all(&content))
            .map(|_| app_info!("File {} created successfully", file))
            .or_else(|e| Err(AppError::from(e)))
    })
}

/// Returns an IO object that creates files for GitLab variables of type "File"
///
/// # Arguments
///
/// * `folder` - Folder where files will be created
/// * `variables` - List of GitLab variables
///
fn create_files(folder: String, variables: Vec<GitLabVariable>) -> IO<Result<()>> {
    IO::unit(move || match fs::create_dir_all(folder.clone()) {
        Ok(_) => Ok(variables
            .iter()
            .filter(|v| matches!(v.variable_type, GitLabVariableType::File))
            .map(|v| (format!("{}/{}.var", folder, v.key), v.value.as_bytes().to_vec()))
            .collect::<Vec<(String, Vec<u8>)>>()),
        Err(e) => {
            app_warning!("Folder {} could not be created", folder);
            Err(AppError::from(e))
        }
    })
    .flat_map(|res: Result<Vec<(String, Vec<u8>)>>| match res {
        Ok(list) => list.into_iter().fold(IO::unit(|| Ok(())), |_, (file, content)| write_file(file, content)),
        Err(e) => IO::unit(|| Err(e)),
    })
}

/// Generates a list of commands for exporting all variables in user's shell
///
/// # Arguments
///
/// * `shell` - [ShellType](enum@ShellType)
/// * `variables` - List of GitLab variables
/// * `folder` - Folder where variables of type "File" are located
///
fn generate_commands(shell: ShellType, variables: &Vec<GitLabVariable>, folder: &String) -> Vec<String> {
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
