use assert_cmd::cargo::CommandCargoExt;
use gitlab_rescue::shell_types::ShellType;
use httpmock::{MockServer, Then, When};
use std::fs;
use std::process::Command;

fn gitlab_rescue_command() -> Command {
    let mut cmd = Command::cargo_bin("gitlab-rescue").unwrap();
    cmd.current_dir("tests");
    cmd
}

fn gitlab_rescue() -> assert_cmd::Command {
    assert_cmd::Command::from_std(gitlab_rescue_command())
}

#[test]
fn test_should_get_a_variable_from_a_project() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method("GET").path("/api/v4/projects/a-project/variables/TEST_VARIABLE_1");
        then.status(200).body_from_file("tests/resources/response_show_env_var_envAll.json");
    });
    gitlab_rescue()
        .args(&["get", "TEST_VARIABLE_1", "-p", "a-project", "-t", "a-token", "-u", &server.base_url()])
        .assert()
        .success()
        .stdout("TEST_1");
    mock.assert();
}

#[test]
fn test_should_get_a_variable_from_a_group() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method("GET").path("/api/v4/groups/a-group/variables/TEST_VARIABLE_1");
        then.status(200).body_from_file("tests/resources/response_show_env_file_envAll.json");
    });
    gitlab_rescue()
        .args(&["get", "TEST_VARIABLE_1", "-g", "a-group", "-t", "a-token", "-u", &server.base_url()])
        .assert()
        .success()
        .stdout("{\"test_variable\":\"one\"}");
    mock.assert();
}

fn httpmock_list() -> impl FnOnce(When, Then) {
    |when, then| {
        when.method("GET").path("/api/v4/projects/a-project/variables");
        then.status(200).header("x-total", "8").body_from_file("tests/resources/response_list_variables.json");
    }
}

fn test_should_generate_dotenv_with_env(env: &str, shell: &str, folder: &str) {
    let server = MockServer::start();
    let mock = server.mock(httpmock_list());
    gitlab_rescue()
        .args(&["dotenv", "a-project", "-t", "a-token", "-u", &server.base_url(), "-e", env])
        .args(&["-s", if shell == "posix" { "bash" } else { shell }])
        .args(&["--folder", folder])
        .assert()
        .success()
        .stdout(fs::read_to_string(format!("tests/resources/dotenv_{}_with_{}_env.txt", shell, env)).unwrap());
    fs::remove_dir_all(format!("tests/{}", folder)).ok();
    fs::remove_file(format!("tests/output-{}-{}.txt", env, shell)).ok();
    gitlab_rescue()
        .args(&["dotenv", "a-project", "-t", "a-token", "-u", &server.base_url(), "-e", env])
        .args(&["-o", &format!("output-{}-{}.txt", env, shell)])
        .args(&["-s", if shell == "posix" { "bash" } else { shell }])
        .args(&["--folder", folder])
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(format!("tests/output-{}-{}.txt", env, shell)).unwrap(),
        fs::read_to_string(format!("tests/resources/dotenv_{}_with_{}_env.txt", shell, env)).unwrap()
    );
    mock.assert_hits(2);
}

#[test]
fn test_should_generate_posix_dotenv_without_env() {
    let path = &format!(".env.All.{}", ShellType::Posix);
    test_should_generate_dotenv_with_env("All", &format!("{}", ShellType::Posix), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
}

#[test]
fn test_should_generate_posix_dotenv_with_dev_env() {
    let path = &format!(".env.dev.{}", ShellType::Posix);
    test_should_generate_dotenv_with_env("dev", &format!("{}", ShellType::Posix), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_1.var", path)).unwrap(), "{\"test_variable\":\"one\"}");
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
}

#[test]
fn test_should_generate_posix_dotenv_with_qa_env() {
    let path = &format!(".env.qa.{}", ShellType::Posix);
    test_should_generate_dotenv_with_env("qa", &format!("{}", ShellType::Posix), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
}

#[test]
fn test_should_generate_posix_dotenv_with_prod_env() {
    let path = &format!(".env.prod.{}", ShellType::Posix);
    test_should_generate_dotenv_with_env("prod", &format!("{}", ShellType::Posix), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_7.var", path)).unwrap(), "{\"test_variable\":\"seven\"}");
}

#[test]
fn test_should_generate_fish_dotenv_without_env() {
    let path = &format!(".env.All.{}", ShellType::Fish);
    test_should_generate_dotenv_with_env("All", &format!("{}", ShellType::Fish), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
}

#[test]
fn test_should_generate_fish_dotenv_with_dev_env() {
    let path = &format!(".env.dev.{}", ShellType::Fish);
    test_should_generate_dotenv_with_env("dev", &format!("{}", ShellType::Fish), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_1.var", path)).unwrap(), "{\"test_variable\":\"one\"}");
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
}

#[test]
fn test_should_generate_fish_dotenv_with_qa_env() {
    let path = &format!(".env.qa.{}", ShellType::Fish);
    test_should_generate_dotenv_with_env("qa", &format!("{}", ShellType::Fish), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
}

#[test]
fn test_should_generate_fish_dotenv_with_prod_env() {
    let path = &format!(".env.prod.{}", ShellType::Fish);
    test_should_generate_dotenv_with_env("prod", &format!("{}", ShellType::Fish), path);
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_4.var", path)).unwrap(), "{\"test_variable\":\"four\"}");
    assert_eq!(fs::read_to_string(format!("tests/{}/TEST_VARIABLE_7.var", path)).unwrap(), "{\"test_variable\":\"seven\"}");
}

#[test]
fn test_should_response_an_error() {
    gitlab_rescue()
        .args(&["get", "TEST_VARIABLE_1", "-g", "a-group", "-t", "a-token", "-u", "a-url"])
        .assert()
        .failure();
}
