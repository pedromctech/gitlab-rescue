# gitlab-rescue

CLI tool for getting and importing GitLab CI/CD variables from a project (Read only).

```text
$ gitlab-rescue --help

gitlab-rescue 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
CLI tool for getting and importing GitLab CI/CD variables from a project (Read only).

USAGE:
    gitlab-rescue [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --project <GITLAB_PROJECT>
            GitLab project ID

    -t, --token <GITLAB_TOKEN>
            A valid GitLab API token

    -u, --api-url <GITLAB_API_URL>
            URL of GitLab API. Default: https://gitlab.com/api/v4

SUBCOMMANDS:
    help        Prints this message or the help of the given subcommand(s)
    get         Print variable in STDOUT
    export      Export variable in current shell (if variable is File type, a file will be created and the path's file will be exported)
    export-all  Export all variables in current shell (file type variables will be stored in a folder)

Instead, you can set request parameters via environment variables:
export GITLAB_PROJECT=<GITLAB_PROJECT>
export GITLAB_TOKEN=<GITLAB_TOKEN>
export GITLAB_API_URL=<GITLAB_API_URL>
```

## gitlab-rescue get

```text
$ gitlab-rescue get --help

gitlab-rescue-get 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
Print variable in STDOUT

USAGE:
    gitlab-rescue get --name <NAME> [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information

    --from-all-if-missing
            If variable is not found in defined environment (-e option), try with "All" environment.

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment (Default: All)

    -n, --name <NAME>
            Name of GitLab CI/CD variable
```

## gitlab-rescue export

```text
$ gitlab-rescue export --help

gitlab-rescue-export 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
Export variable in current shell (if variable is File type, a file will be created and the path's file will be exported)

USAGE:
    gitlab-rescue export --name <NAME> [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information
    
    --from-all-if-missing
            If variable is not found in defined environment (-e option), try with "All" environment.

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment (Default: All)

    -n, --name <NAME>
            Name of GitLab CI/CD variable

    -o, --output-file <FILE>
            Path file when value will be stored (only for variables with type "File"). Default: $PWD/<NAME>.var
```

## gitlab-rescue export-all

```text
$ gitlab-rescue export-all --help

gitlab-rescue-export-all 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
Export all variables in current shell.

USAGE:
    gitlab-rescue export-all [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information

        --from-all-if-missing
            If variables are not found in defined environment (-e option), try with "All" environment.

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment. (Default: None)

        --folder <PATH>
            Path where variables with type "File" will be stored. Files will be created with format <VARIABLE_NAME>.var. Default: $PWD/.env.<ENVIRONMENT>.
```
