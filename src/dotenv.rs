#![allow(clippy::unit_arg)]

use crate::api_client::{api_client, DEFAULT_ENVIRONMENT};
use crate::app_error::{AppError, Result};
use crate::dotenv::AppError::{Cli, InvalidInput};
use crate::gitlab_api::{GitLabApi, GitLabProject, GitLabVariable, GitLabVariableType};
use crate::shell_types::ShellType;
use crate::IO;
use crate::{app_info, app_warning, extract_token, extract_url, Performable};
use clap::ArgMatches;
use std::convert::From;
use std::env;
use std::io::Write;
use std::sync::mpsc::channel;
use std::{fs, fs::File};
use threadpool::ThreadPool;
use urlencoding::encode;

/// Arguments for `dotenv` command
#[derive(Clone, Debug, PartialEq)]
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

impl Performable for DotEnvCommand {
    fn get_action(self) -> IO<Result<()>> {
        IO::unit(move || {
            app_info!("Getting variables from project {}...", self.gitlab_project.name);
            Ok((self.clone(), get_list_of_variables(&self)?))
        })
        .map(|res: Result<(DotEnvCommand, Vec<GitLabVariable>)>| {
            res.and_then(|(cmd, variables)| {
                app_info!("Creating files for variables of type File...");
                fs::create_dir_all(cmd.folder.clone()).map_err(|e| InvalidInput(format!("Folder {} could not be created. Error: {}", &cmd.folder, e)))?;
                get_files_to_create(&cmd.folder, &variables)
                    .into_iter()
                    .fold(Ok(()), |acc: Result<()>, (file, content)| acc.and(Ok(File::create(file)?.write_all(&content)?)))
                    .map_err(|e| Cli(format!("Some files could not be created. Error: {}", e)))
                    .map(|_| (cmd, variables))
            })
        })
        .map(|res: Result<(DotEnvCommand, Vec<GitLabVariable>)>| {
            res.and_then(|(cmd, variables)| {
                app_info!("Creating dotenv command list...");
                match (generate_commands(cmd.shell, &variables, &cmd.folder), &cmd.output_file) {
                    (list, Some(f)) => File::create(&f)
                        .and_then(|mut f| f.write_all(&format!("{}{}", &list.join("\n"), "\n").as_bytes().to_vec()))
                        .or_else(|e| {
                            app_warning!("Output file could not be created. Error: {}. Printing dotenv in STDOUT...", e);
                            Ok(list.into_iter().for_each(|c| println!("{}", c)))
                        }),
                    (list, None) => Ok(list.into_iter().for_each(|c| println!("{}", c))),
                }
            })
        })
    }
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

/// GitLab requests configuration
#[derive(Clone, Debug, PartialEq)]
pub struct RequestConfig {
    /// GitLab instance URL
    url: String,
    /// GitLab instance URL
    token: String,
    /// GitLab instance URL
    gitlab_project: String,
    /// Page to request
    page: usize,
    /// Number of items to list per page
    per_page: usize,
}

impl RequestConfig {
    fn from(cmd: &DotEnvCommand, page: usize) -> Self {
        RequestConfig {
            url: cmd.url.clone(),
            token: cmd.token.clone(),
            gitlab_project: cmd.gitlab_project.name.clone(),
            page,
            per_page: cmd.per_page,
        }
    }
}

/// Get list of variables to export in dotenv commands
fn get_list_of_variables(cmd: &DotEnvCommand) -> Result<Vec<GitLabVariable>> {
    Ok(list_from_api(RequestConfig::from(&cmd, 1))
        .and_then(|(list, total)| match list.len() == total {
            true => Ok(list),
            _ => Ok([list, remaining_from_api(RequestConfig::from(&cmd, 2), num_requests(total, cmd.per_page), cmd.parallel)?].concat()),
        })?
        .into_iter()
        .filter(|v| v.environment_scope == DEFAULT_ENVIRONMENT || v.environment_scope == cmd.environment)
        .collect())
}

/// Returns a list of variables according to some request parameters
///
/// # Arguments
///
/// * `request` - Request parameters
///
fn list_from_api(request: RequestConfig) -> Result<(Vec<GitLabVariable>, usize)> {
    api_client(&request.url, &request.token).list_from_project(&request.gitlab_project, request.page, request.per_page)
}

/// Returns a list with remaining variables that could not be obtained in the first request
///
/// # Arguments
///
/// * `requests` - Number of requests to make to obtain the remaining variables
///
fn remaining_from_api(request: RequestConfig, num_requests: usize, parallel: usize) -> Result<Vec<GitLabVariable>> {
    let pool = ThreadPool::new(parallel);
    let (tx, rx) = channel();
    (0..num_requests)
        .fold(rx, |acc, i| {
            let (tx, r) = (tx.clone(), RequestConfig { page: i + 2, ..request.clone() });
            pool.execute(move || tx.send(list_from_api(r)).expect("Thread Error"));
            acc
        })
        .into_iter()
        .take(num_requests)
        .fold(Ok(vec![]), |a: Result<Vec<GitLabVariable>>, res| a.and_then(|acc| Ok([acc, res.map(|(l, _)| l)?].concat())))
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
    if total - per_request <= per_request {
        1
    } else {
        ((total - per_request) as f64 / per_request as f64).ceil() as usize
    }
}

/// Get the list of files to create for variables of type "File"
/// # Arguments
///
/// * `folder` - Folder where files will be created
/// * `variables` - List of GitLab variables
///
fn get_files_to_create(folder: &str, variables: &[GitLabVariable]) -> Vec<(String, Vec<u8>)> {
    variables
        .iter()
        .filter(|v| matches!(v.variable_type, GitLabVariableType::File))
        .map(|v| (format!("{}/{}.var", folder, v.key), v.value.as_bytes().to_vec()))
        .collect()
}

/// Generates a list of commands for exporting all variables in user's shell
///
/// # Arguments
///
/// * `shell` - [ShellType](enum@ShellType)
/// * `variables` - List of GitLab variables
/// * `folder` - Folder where variables of type "File" are located
///
fn generate_commands(shell: ShellType, variables: &[GitLabVariable], folder: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clap_app::app;
    use crate::gen::tests::*;
    use crate::gitlab_api::tests::{gen_variable, gen_variable_list, GEN_GITLAB_PROJECT};
    use crate::shell_types::tests::GEN_SHELL_TYPE;
    use httpmock::{MockServer, Then, When};
    use lazy_static::lazy_static;

    lazy_static! {
        static ref GEN_TOTAL: usize = gen_usize_from_range(*GEN_PER_PAGE, 301);
        static ref GEN_PER_PAGE: usize = gen_usize_from_range(10, 101);
        static ref GEN_URL: String = gen_alpha_char(5);
        static ref GEN_TOKEN: String = gen_alpha_char(5);
        static ref GEN_PAGE: usize = gen_usize_from_range(1, 5);
        static ref GEN_ENVIRONMENT: String = "A".to_owned();
        static ref GEN_OUTPUT_FILE: String = gen_alpha_char(5);
        static ref GEN_FOLDER: String = gen_alpha_char(5);
        static ref GEN_GROUP_VARS: bool = gen_bool();
    }

    fn gen_dotenv_command(url: Option<String>) -> DotEnvCommand {
        DotEnvCommand {
            gitlab_project: GEN_GITLAB_PROJECT.clone(),
            environment: GEN_ENVIRONMENT.clone(),
            output_file: Some(GEN_OUTPUT_FILE.clone()),
            shell: *GEN_SHELL_TYPE,
            folder: GEN_FOLDER.clone(),
            per_page: *GEN_PER_PAGE,
            with_group_vars: *GEN_GROUP_VARS,
            parallel: num_cpus::get(),
            url: url.map_or_else(|| GEN_URL.clone(), |u| u),
            token: GEN_TOKEN.clone(),
        }
    }

    fn gen_request_config(url: Option<String>) -> RequestConfig {
        RequestConfig {
            url: url.map_or_else(|| GEN_URL.clone(), |u| u),
            token: GEN_TOKEN.clone(),
            gitlab_project: GEN_GITLAB_PROJECT.name.clone(),
            page: *GEN_PAGE,
            per_page: *GEN_PER_PAGE,
        }
    }

    #[test]
    fn get_request_config_from_dotenv() {
        assert_eq!(RequestConfig::from(&gen_dotenv_command(None), *GEN_PAGE), gen_request_config(None));
    }

    #[test]
    fn get_dotenv_from_cli_args() {
        app()
            .get_matches_from(vec![
                "gitlab-rescue",
                "dotenv",
                &GEN_GITLAB_PROJECT.name,
                &format!("-e={}", *GEN_ENVIRONMENT),
                &format!("-o={}", *GEN_OUTPUT_FILE),
                &format!("--folder={}", *GEN_FOLDER),
                &format!("--shell={}", *GEN_SHELL_TYPE),
                &format!("--per-page={}", *GEN_PER_PAGE),
                &format!("--parallel={}", num_cpus::get()),
                &format!("-u={}", *GEN_URL),
                &format!("-t={}", *GEN_TOKEN),
            ])
            .subcommand()
            .1
            .map_or_else(|| panic!(), |args| assert_eq!(DotEnvCommand::from(args), gen_dotenv_command(None)));
    }

    fn httpmock_list_variables() -> impl FnOnce(When, Then) {
        move |when, then| {
            when.method("GET").path(format!("/api/v4/projects/{}/variables", GEN_GITLAB_PROJECT.name.clone()));
            then.status(200)
                .header("Content-Type", "application/json")
                .header("x-total", &GEN_TOTAL.to_string())
                .json_body_obj(&gen_variable_list(if *GEN_PER_PAGE < *GEN_TOTAL { *GEN_PER_PAGE } else { *GEN_TOTAL }));
        }
    }

    #[test]
    fn test_get_list_of_variables() {
        let server = MockServer::start();
        let mock = server.mock(httpmock_list_variables());
        assert!(get_list_of_variables(&gen_dotenv_command(Some(server.base_url())))
            .map(|l| l.into_iter().all(|v| v.environment_scope == *GEN_ENVIRONMENT || v.environment_scope == DEFAULT_ENVIRONMENT))
            .unwrap());
        mock.assert_hits(if *GEN_PER_PAGE >= *GEN_TOTAL { 1 } else { num_requests(*GEN_TOTAL, *GEN_PER_PAGE) + 1 });
    }

    #[test]
    fn test_remaining_from_api() {
        let server = MockServer::start();
        let mock = server.mock(httpmock_list_variables());
        let num_requests = num_requests(*GEN_TOTAL, *GEN_PER_PAGE);
        remaining_from_api(gen_request_config(Some(server.base_url())), num_requests, num_cpus::get())
            .map_or_else(|_| panic!(), |list| assert_eq!(list.len(), num_requests * *GEN_PER_PAGE));
        mock.assert_hits(num_requests);
    }

    #[test]
    fn test_num_requests() {
        let (total, per_request) = (100, 10);
        assert_eq!(num_requests(total, per_request), 9);
        let (total, per_request) = (100, 90);
        assert_eq!(num_requests(total, per_request), 1);
    }

    #[test]
    fn test_get_files_to_create() {
        let variable = gen_variable(Some(GitLabVariableType::File));
        assert_eq!(
            get_files_to_create(&GEN_FOLDER, &[variable.clone(), gen_variable(Some(GitLabVariableType::EnvVar))]),
            vec![(format!("{}/{}.var", *GEN_FOLDER, variable.key), variable.value.as_bytes().to_vec())]
        );
    }

    #[test]
    fn test_generate_commands() {
        let env_variable = gen_variable(Some(GitLabVariableType::EnvVar));
        let file_variable = gen_variable(Some(GitLabVariableType::File));
        assert_eq!(
            generate_commands(*GEN_SHELL_TYPE, &[env_variable.clone(), file_variable.clone()], &GEN_FOLDER),
            vec![
                GEN_SHELL_TYPE.export_command(env_variable.key.clone(), env_variable.value),
                GEN_SHELL_TYPE.export_command(file_variable.key.clone(), format!("{}/{}.var", *GEN_FOLDER, file_variable.key))
            ]
        );
    }
}
