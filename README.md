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
    list        List all variables in JSON format
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

## gitlab-rescue list

```text
$ gitlab-rescue list --help

gitlab-rescue-list 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
List all variables in JSON format

USAGE:
    gitlab-rescue list [OPTIONS]

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

## Usage

```bash
# Instead of using CLI flags, you can export GitLab instance variables
$ export GITLAB_PROJECT=<GITLAB_PROJECT>
$ export GITLAB_TOKEN=<GITLAB_TOKEN>
$ export GITLAB_API_URL=<GITLAB_API_URL>

# Get a variable
$ gitlab-rescue get -n MY_VARIABLE
hello-world

# Get a file
$ gitlab-rescue get -n MY_CREDENTIALS -e develop
{
    "a_super_secret_info": "a_super_secret_value"
}

# Export a variable
$ gitlab-rescue export -n MY_VARIABLE
$ echo $MY_VARIABLE
hello-world

# Export a file
$ gitlab-rescue export -n MY_CREDENTIALS -e develop
$ echo $MY_CREDENTIALS
$PWD/MY_CREDENTIALS.var
$ cat $MY_CREDENTIALS
{
    "a_super_secret_info": "a_super_secret_value"
}

# Export all
$ gitlab-rescue export-all -e develop
$ echo $MY_VARIABLE ## This variable is not in "develop" scope, so it was not exported.
$ echo $MY_CREDENTIALS
$PWD/.env.develop/MY_CREDENTIALS.var
$ cat $MY_CREDENTIALS
{
    "a_super_secret_info": "a_super_secret_value"
}

# Export all with fallback
$ gitlab-rescue export-all -e develop --from-all-if-missing
$ echo $MY_VARIABLE
hello-world
$ echo $MY_CREDENTIALS
$PWD/.env.develop/MY_CREDENTIALS.var
$ cat $MY_CREDENTIALS
{
    "a_super_secret_info": "a_super_secret_value"
}

# List variables
$ gitlab-rescue list -e develop --from-all-if-missing >output.json
$ cat output.json
{
    "MY_VARIABLE": "",
    "MY_CREDENTIALS": "{\\n\"a_super_secret_info\": \"a_super_secret_value\"\\n}"
}

# For example, you can get a JSON file using jq as follows:
$ jq '.MY_CREDENTIALS | fromjson' output.json >my_credentials.json
$ cat my_credentials.json
{
    "a_super_secret_info": "a_super_secret_value"
}
```
