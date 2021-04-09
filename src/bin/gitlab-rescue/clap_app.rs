use clap::App;
use clap::{crate_authors, crate_version, App as ClapApp, Arg, SubCommand};

fn environment_arg() -> Arg<'static, 'static> {
    Arg::with_name("environment")
        .long("environment")
        .short("e")
        .value_name("ENVIRONMENT")
        .long_help("Name of GitLab CI/CD environment.")
        .default_value("All")
}

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

fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("get")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Print variable in STDOUT")
        .args(&gitlab_instance_args())
        .arg(&environment_arg())
        .args(&[
            Arg::with_name("VARIABLE_NAME")
                .long_help("Name of GitLab CI/CD variable.")
                .required(true)
                .index(1),
            Arg::with_name("from-all-if-missing")
                .long("from-all-if-missing")
                .long_help("If variable(s) is(are) not found in defined environment (-e option), try searching in \"All\" environment."),
        ])
}

pub fn app() -> ClapApp<'static, 'static> {
    ClapApp::new("gitlab-rescue")
        .version(crate_version!())
        .author(crate_authors!())
        .about("CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)")
        .subcommand(
            // Project command
            SubCommand::with_name("project")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get variable from a GitLab project")
                .arg(Arg::with_name("GITLAB_PROJECT_ID").long_help("GitLab project ID.").required(true).index(1))
                // Project subcommands
                .subcommand(get_subcommand()),
        )
        .subcommand(
            // Group command
            SubCommand::with_name("group")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get variable from a GitLab group")
                .arg(Arg::with_name("GITLAB_GROUP_ID").long_help("GitLab group ID.").required(true).index(1))
                // Group subcommands
                .subcommand(get_subcommand()),
        )
        .subcommand(
            // Env command
            SubCommand::with_name("env")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Export project variables in the current shell (by default first 20 variables)")
                .arg(Arg::with_name("GITLAB_PROJECT_ID").long_help("GitLab project ID.").required(true).index(1))
                .args(&gitlab_instance_args())
                .arg(&environment_arg())
                .args(&[
                    Arg::with_name("folder")
                        .long("folder")
                        .value_name("PATH")
                        .long_help("Path where variables with type \"File\" will be stored. Files will be created with format <VARIABLE_NAME>.var. [default: $PWD/.env.<ENVIRONMENT>]"),
                    Arg::with_name("full-list")
                        .long("full-list")
                        .long_help("List all varibles (without this option, only 20 variables are showed). This option ovewrites --page and --per-page options."),
                    Arg::with_name("page")
                        .long("page")
                        .value_name("PAGE")
                        .long_help("Page number.\r\n(See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination).")
                        .default_value("1"),
                    Arg::with_name("per-page")
                        .long("per-page")
                        .value_name("PER_PAGE")
                        .long_help("Number of items to list per page.\r\n(See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination).")
                        .default_value("20"),
                    Arg::with_name("with-group-vars")
                        .long("with-group-vars")
                        .long_help("Export group variables if project belongs to a group"),
                ]),
        )
}
