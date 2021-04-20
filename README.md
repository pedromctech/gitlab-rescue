# gitlab-rescue (WIP)

CLI tool for getting and importing GitLab CI/CD variables from a project (Read only).

```text
gitlab-rescue 0.1.0
CLI tool for getting and importing GitLab CI/CD variables from a project (Read only)

USAGE:
    gitlab-rescue [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    dotenv    Export project variables in the current shell (by default first 20 variables)
    get       Print variable in STDOUT
    help      Prints this message or the help of the given subcommand(s)
```

## gitlab-rescue get

```text
gitlab-rescue-get 0.1.0
Pedro Miranda <pedrodotmc@gmail.com>
Print variable in STDOUT

USAGE:
    gitlab-rescue get [FLAGS] [OPTIONS] <VARIABLE_NAME> --group <GITLAB_GROUP> --project <GITLAB_PROJECT>

FLAGS:
        --from-all-if-missing
            If variable(s) is(are) not found in defined environment (-e option), try searching in "All" environment.

    -h, --help
            Prints help information

    -V, --version
            Prints version information

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment. [default: All]

    -g, --group <GITLAB_GROUP>
            The ID of a group or URL-encoded path of the group. This should not be used with --project option.

    -p, --project <GITLAB_PROJECT>
            The ID of a project or URL-encoded NAMESPACE/PROJECT_NAME of the project. This should not be used with
            --group option.
    -t, --token <GITLAB_API_TOKEN>
            A valid GitLab API token. Alternatively, you can export GITLAB_API_TOKEN variable.

    -u, --url <GITLAB_URL>
            URL of GitLab API. [default: https://gitlab.com]. Alternatively, you can export GITLAB_URL variable.

ARGS:
    <VARIABLE_NAME>
            Name of GitLab CI/CD variable.
```

## gitlab-rescue dotenv

```text
gitlab-rescue-dotenv 0.1.0
Export project variables in the current shell (by default first 20 variables)

USAGE:
    gitlab-rescue dotenv [OPTIONS] <GITLAB_PROJECT>

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information

OPTIONS:
    -e, --environment <ENVIRONMENT>
            Name of GitLab CI/CD environment. [default: All]

        --folder <PATH>
            Path where variables with type "File" will be stored. Files will be created with format <VARIABLE_NAME>.var.
            [default: $PWD/.env.<ENVIRONMENT>]
    -o, --output <OUTPUT_FILE>
            Write dotenv to a file instead of stdout.

        --parallel <PARALLEL>
            Number of threads for GitLab API requests.

        --per-page <PER_PAGE>
            Number of items to bring per request.
            (See https://docs.gitlab.com/ee/api/README.html#offset-based-pagination). [default: 100]
    -s, --shell <SHELL>
            Generate dotenv for this shell type. Supported shells are: bash, zsh and fish. [default: bash]  [possible
            values: bash, zsh, fish]
    -t, --token <GITLAB_API_TOKEN>
            A valid GitLab API token. Alternatively, you can export GITLAB_API_TOKEN variable.

    -u, --url <GITLAB_URL>
            URL of GitLab API. [default: https://gitlab.com]. Alternatively, you can export GITLAB_URL variable.

ARGS:
    <GITLAB_PROJECT>
            The ID of a project or URL-encoded NAMESPACE/PROJECT_NAME of the project.
```

## Examples

### Get a variable
```bash
$ gitlab-rescue get MY_VARIABLE -p my-project
[INFO] Getting variable from project my-project...
[SUCCESS] Variable MY_VARIABLE obtained successfully
my-value

# Or export variable
$ export VARIABLE=$(gitlab-rescue get MY_VARIABLE -p my-project)
[INFO] Getting variable from project my-project...
[SUCCESS] Variable MY_VARIABLE obtained successfully
$ echo $VARIABLE
my-value
```

### Creating a dotenv file
```bash
$ gitlab-rescue dotenv my-project -o .env
[INFO] Getting variables from project my-project...
[INFO] Creating files for variables of type File...
[INFO] Creating dotenv command list...
[INFO] File .env created successfully
$ cat .env
export MY_VARIABLE_1="a-value"
export MY_VARIABLE_2="another-value"
export MY_FILE_VARIABLE=".env.All/MY_FILE_VARIABLE.var"
$ ls .env.All
MY_FILE_VARIABLE.var
```
