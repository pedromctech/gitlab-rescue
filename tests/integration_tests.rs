use assert_cmd::cargo::CommandCargoExt;
use httpmock::MockServer;
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
fn test_get_variable_from_project() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method("GET")
            .path("/api/v4/projects/a-project/variables/SAMPLE_VARIABLE")
            .query_param("filter[environment_scope]", "*")
            .header("PRIVATE-TOKEN", "a-gitlab-token");
        then.status(200).body_from_file("tests/resources/response_show_env_var_envAll.json");
    });
    gitlab_rescue()
        .args(&["get", "SAMPLE_VARIABLE", "-p", "a-project", "-t", "a-gitlab-token", "-u", &server.base_url()])
        .assert()
        .success()
        .stdout("TEST_1");
    mock.assert();
}
