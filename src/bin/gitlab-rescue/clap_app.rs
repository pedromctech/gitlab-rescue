use clap::{crate_authors, crate_version, App as ClapApp, Arg, SubCommand};

/// Returns an arg object with `--environment` flag configuration
fn environment_arg() -> Arg<'static, 'static> {
    Arg::with_name("environment")
        .long("environment")
        .short("e")
        .value_name("ENVIRONMENT")
        .long_help("Name of GitLab CI/CD environment.")
        .default_value("All")
}

/// Returns an array with `--token` and `--url` flags configuration
fn gitlab_instance_args() -> [Arg<'static, 'static>; 2] {
    [
        Arg::with_name("token")
            .long("token")
            .short("t")
            .value_name("GITLAB_API_TOKEN")
            .long_help("A valid GitLab API token. Alternatively, you can export GITLAB_API_TOKEN variable."),
        Arg::with_name("url")
            .long("url")
            .short("u")
            .value_name("GITLAB_URL")
            .long_help("URL of GitLab API. [default: https://gitlab.com]. Alternatively, you can export GITLAB_URL variable."),
    ]
}

/// Returns an array with `--project` and `--group` flags configuration
fn project_and_group_args() -> [Arg<'static, 'static>; 2] {
    [
        Arg::with_name("project")
            .long("project")
            .short("p")
            .value_name("GITLAB_PROJECT")
            .long_help("The ID of a project or URL-encoded NAMESPACE/PROJECT_NAME of the project. This should not be used with --group option.")
            .conflicts_with("group")
            .required(true),
        Arg::with_name("group")
            .long("group")
            .short("g")
            .value_name("GITLAB_GROUP")
            .long_help("The ID of a group or URL-encoded path of the group. This should not be used with --project option.")
            .conflicts_with("project")
            .required(true),
    ]
}

/// Returns the `ClapApp` object with all CLI structure
pub fn app() -> ClapApp<'static, 'static> {
    ClapApp::new("gitlab-rescue")
        .version(crate_version!())
        .author(crate_authors!())
        .about("CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)")
        .subcommand(
            // Get command
            SubCommand::with_name("get")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Print variable in STDOUT")
                .args(&gitlab_instance_args())
                .arg(&environment_arg())
                .args(&project_and_group_args())
                .args(&[
                    Arg::with_name("VARIABLE_NAME").long_help("Name of GitLab CI/CD variable.").required(true).index(1),
                    Arg::with_name("from-all-if-missing")
                        .long("from-all-if-missing")
                        .long_help("If variable(s) is(are) not found in defined environment (-e option), try searching in \"All\" environment."),
                ]),
        )
        .subcommand(
            // Local Env command
            SubCommand::with_name("dotenv")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Export project variables in the current shell (by default first 20 variables)")
                .arg(
                    Arg::with_name("GITLAB_PROJECT")
                        .long_help("The ID of a project or URL-encoded NAMESPACE/PROJECT_NAME of the project.")
                        .required(true)
                        .index(1),
                )
                .args(&gitlab_instance_args())
                .arg(&environment_arg())
                .args(&[
                    Arg::with_name("output")
                        .long("output")
                        .short("o")
                        .value_name("OUTPUT_FILE")
                        .long_help("Write dotenv to a file instead of stdout."),
                    Arg::with_name("shell")
                        .long("shell")
                        .short("s")
                        .value_name("SHELL")
                        .possible_values(&["bash", "zsh", "fish"])
                        .default_value("bash")
                        .long_help("Generate dotenv for this shell type. Supported shells are: bash, zsh and fish."),
                    Arg::with_name("folder")
                        .long("folder")
                        .value_name("PATH")
                        .long_help("Path where variables with type \"File\" will be stored. Files will be created with format <VARIABLE_NAME>.var. [default: $PWD/.env.<ENVIRONMENT>]"),
                    Arg::with_name("per-page")
                        .long("per-page")
                        .value_name("PER_PAGE")
                        .long_help("Number of items to bring per request.\r\n(See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination).")
                        .default_value("100"),
                    Arg::with_name("parallel")
                        .long("parallel")
                        .value_name("PARALLEL")
                        .long_help("Number of threads for GitLab API requests."),
                ]),
        )
}
