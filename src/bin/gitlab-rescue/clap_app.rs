use clap::{crate_authors, crate_version, App as ClapApp, Arg, SubCommand};

pub fn app() -> ClapApp<'static, 'static> {
    let gitlab_instance_args = [
        Arg::with_name("project-id")
            .long("project-id")
            .short("p")
            .value_name("GITLAB_PROJECT_ID")
            .long_help("GitLab project ID."),
        Arg::with_name("token")
            .long("token")
            .short("t")
            .value_name("GITLAB_API_TOKEN")
            .long_help("A valid GitLab API token."),
        Arg::with_name("url")
            .long("url")
            .short("u")
            .value_name("GITLAB_API_URL")
            .long_help("URL of GitLab API. [default: https://gitlab.com/api/v4]"),
    ];
    let common_args = [
        Arg::with_name("environment")
            .long("environment")
            .short("e")
            .value_name("ENVIRONMENT")
            .long_help("Name of GitLab CI/CD environment.")
            .default_value("All"),
        Arg::with_name("from-all-if-missing")
            .long("from-all-if-missing")
            .long_help("If variable(s) is(are) not found in defined environment (-e option), try searching in \"All\" environment."),
    ];
    let after_help = "Instead, you can set request parameters via environment variables:
    \rexport GITLAB_PROJECT_ID=<GITLAB_PROJECT_ID>
    \rexport GITLAB_API_TOKEN=<GITLAB_API_TOKEN>
    \rexport GITLAB_API_URL=<GITLAB_API_URL>";
    ClapApp::new("gitlab-rescue")
        .version(crate_version!())
        .author(crate_authors!())
        .about(
            "CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)",
        )
        .subcommand(
            SubCommand::with_name("get")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Print variable in STDOUT")
                .args(&gitlab_instance_args)
                .args(&common_args)
                .arg(Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("VARIABLE_NAME")
                    .long_help("Name of GitLab CI/CD variable.")
                    .required(true)
                )
                .after_help(after_help)
        )
        .subcommand(
            SubCommand::with_name("env")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Export variables in current shell (by default first 20 variables)")
                .args(&gitlab_instance_args)
                .args(&common_args)
                .args(&[
                    Arg::with_name("folder")
                        .long("folder")
                        .value_name("PATH")
                        .long_help("Path where variables with type \"File\" will be stored. Files will be created with format <VARIABLE_NAME>.var. [default: $PWD/.env.<ENVIRONMENT>]"),
                    Arg::with_name("all")
                        .long("all")
                        .long_help("List all varibles (without this option, only 20 variables are showed). This option ovewrites --page and --per-page options."),
                    Arg::with_name("page")
                        .long("page")
                        .value_name("PAGE")
                        .long_help("Page number (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination).")
                        .default_value("1"),
                    Arg::with_name("per-page")
                        .long("per-page")
                        .value_name("PER_PAGE")
                        .long_help("Number of items to list per page (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination).")
                        .default_value("20")
                ])
                .after_help(after_help),
        )
}
