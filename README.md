# gitlab-rescue (WIP)

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
    -g, --group-id <GITLAB_GROUP_ID>
            GitLab group ID

    -p, --project-id <GITLAB_PROJECT_ID>
            GitLab project ID

    -t, --token <GITLAB_API_TOKEN>
            A valid GitLab API token

    -u, --url <GITLAB_URL>
            URL of GitLab API. Default: https://gitlab.com

SUBCOMMANDS:
    help        Prints this message or the help of the given subcommand(s)
    get         Print variable in STDOUT
    list        List all variables in JSON format
    export      Export variable in current shell (if variable is File type, a file will be created and the path's file will be exported)
    local-env   Export all variables in current shell (file type variables will be stored in a folder)

Instead, you can set request parameters via environment variables:
export GITLAB_PROJECT_ID=<GITLAB_PROJECT_ID>
export GITLAB_API_TOKEN=<GITLAB_API_TOKEN>
export GITLAB_URL=<GITLAB_URL>
```

## gitlab-rescue get (Implemented)

```text
$ gitlab-rescue get --help

USAGE:
    gitlab-rescue get [FLAGS] [OPTIONS] <VARIABLE_NAME> [--group <GITLAB_GROUP>|--project <GITLAB_PROJECT>]

FLAGS:
        --from-all-if-missing    If variable(s) is(are) not found in defined environment (-e option), try searching in
                                 "All" environment.
    -h, --help                   Prints help information
    -V, --version                Prints version information

OPTIONS:
    -e, --environment <ENVIRONMENT>    Name of GitLab CI/CD environment. [default: All]
    -g, --group <GITLAB_GROUP>         The ID of a group or URL-encoded path of the group. This should not be used with
                                       --group option.
    -p, --project <GITLAB_PROJECT>     The ID of a project or URL-encoded NAMESPACE/PROJECT_NAME of the project. This
                                       should not be used with --group option.
    -t, --token <GITLAB_API_TOKEN>     A valid GitLab API token. Alternatively, you can export GITLAB_API_TOKEN
                                       variable.
    -u, --url <GITLAB_URL>             URL of GitLab API. [default: https://gitlab.com]. Alternatively, you can export
                                       GITLAB_URL variable.

ARGS:
    <VARIABLE_NAME>    Name of GitLab CI/CD variable.
```

## gitlab-rescue list (Not implemented)

```text
$ gitlab-rescue list --help

gitlab-rescue-list 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
List GitLab CI/CD variables in JSON format (by default first 20 variables).

USAGE:
    gitlab-rescue list [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information

    -a, --all
            List all varibles. By default, this command only load first 20 variables (https://docs.gitlab.com/ee/api/README.html#offset-based-pagination).

    --from-all-if-missing
            If variable is not found in defined environment (-e option), try searching in "All" environment.

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment (Default: All)
        
        --page <PAGE>
            Page number (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination). Default: 1.

        --per-page <PER_PAGE>
            Number of items to list per page (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination). Default: 20, Max. 100.
```

## gitlab-rescue export (Not implemented)

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
            If variable is not found in defined environment (-e option), try searching in "All" environment.

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment (Default: All)

    -n, --name <NAME>
            Name of GitLab CI/CD variable

    -o, --output-file <FILE>
            Path file when value will be stored (only for variables with type "File"). Default: $PWD/<NAME>.var
```

## gitlab-rescue local-env (WIP)

```text
$ gitlab-rescue local-env --help

USAGE:
    gitlab-rescue local-env [FLAGS] [OPTIONS] <GITLAB_PROJECT>

FLAGS:
    -a, --all                
            List all varibles (without this option, only 20 variables are showed). This option ovewrites --page and
            --per-page options.
    -h, --help               
            Prints help information

    -V, --version            
            Prints version information

        --with-group-vars    
            Export group variables if project belongs to a group

OPTIONS:
    -e, --environment <ENVIRONMENT>    
            Name of GitLab CI/CD environment. [default: All]

        --folder <PATH>                
            Path where variables with type "File" will be stored. Files will be created with format <VARIABLE_NAME>.var.
            [default: $PWD/.env.<ENVIRONMENT>]
        --page <PAGE>                  
            Page number.
            (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination). [default: 1]
        --per-page <PER_PAGE>          
            Number of items to list per page.
            (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination). [default: 20]
    -t, --token <GITLAB_API_TOKEN>     
            A valid GitLab API token. Alternatively, you can export GITLAB_API_TOKEN variable.

    -u, --url <GITLAB_URL>             
            URL of GitLab API. [default: https://gitlab.com]. Alternatively, you can export GITLAB_URL variable.

ARGS:
    <GITLAB_PROJECT>    
            The ID of a project or URL-encoded NAMESPACE/PROJECT_NAME of the project.
```
