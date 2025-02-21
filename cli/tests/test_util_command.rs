// Copyright 2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use insta::assert_snapshot;

use crate::common::TestEnvironment;

#[test]
fn test_util_config_schema() {
    let test_env = TestEnvironment::default();
    let stdout = test_env.jj_cmd_success(test_env.env_root(), &["util", "config-schema"]);
    // Validate partial snapshot, redacting any lines nested 2+ indent levels.
    insta::with_settings!({filters => vec![(r"(?m)(^        .*$\r?\n)+", "        [...]\n")]}, {
        assert_snapshot!(stdout, @r#"
        {
            "$schema": "http://json-schema.org/draft-04/schema",
            "$comment": "`taplo` and the corresponding VS Code plugins only support draft-04 verstion of JSON Schema, see <https://taplo.tamasfe.dev/configuration/developing-schemas.html>. draft-07 is mostly compatible with it, newer versions may not be.",
            "title": "Jujutsu config",
            "type": "object",
            "description": "User configuration for Jujutsu VCS. See https://jj-vcs.github.io/jj/latest/config/ for details",
            "properties": {
                [...]
            }
        }
        [EOF]
        "#);
    });
}

#[test]
fn test_gc_args() {
    let test_env = TestEnvironment::default();
    // Use the local backend because GitBackend::gc() depends on the git CLI.
    test_env.jj_cmd_ok(
        test_env.env_root(),
        &["init", "repo", "--config=ui.allow-init-native=true"],
    );
    let repo_path = test_env.env_root().join("repo");

    let (_stdout, stderr) = test_env.jj_cmd_ok(&repo_path, &["util", "gc"]);
    insta::assert_snapshot!(stderr, @"");

    let stderr = test_env.jj_cmd_failure(&repo_path, &["util", "gc", "--at-op=@-"]);
    insta::assert_snapshot!(stderr, @r"
    Error: Cannot garbage collect from a non-head operation
    [EOF]
    ");

    let stderr = test_env.jj_cmd_failure(&repo_path, &["util", "gc", "--expire=foobar"]);
    insta::assert_snapshot!(stderr, @r"
    Error: --expire only accepts 'now'
    [EOF]
    ");
}

#[test]
fn test_gc_operation_log() {
    let test_env = TestEnvironment::default();
    // Use the local backend because GitBackend::gc() depends on the git CLI.
    test_env.jj_cmd_ok(
        test_env.env_root(),
        &["init", "repo", "--config=ui.allow-init-native=true"],
    );
    let repo_path = test_env.env_root().join("repo");

    // Create an operation.
    std::fs::write(repo_path.join("file"), "a change\n").unwrap();
    test_env.jj_cmd_ok(&repo_path, &["commit", "-m", "a change"]);
    let op_to_remove = test_env.current_operation_id(&repo_path);

    // Make another operation the head.
    std::fs::write(repo_path.join("file"), "another change\n").unwrap();
    test_env.jj_cmd_ok(&repo_path, &["commit", "-m", "another change"]);

    // This works before the operation is removed.
    test_env.jj_cmd_ok(&repo_path, &["debug", "operation", &op_to_remove]);

    // Remove some operations.
    test_env.jj_cmd_ok(&repo_path, &["operation", "abandon", "..@-"]);
    test_env.jj_cmd_ok(&repo_path, &["util", "gc", "--expire=now"]);

    // Now this doesn't work.
    let stderr = test_env.jj_cmd_failure(&repo_path, &["debug", "operation", &op_to_remove]);
    insta::assert_snapshot!(stderr, @r#"
    Error: No operation ID matching "82d97d16a736e4cc050cb4152664bd0a0b9f33a295c86956f7b09f02b7b127c93241a4841926bbe8fd229a2863ccad974604fbd7d0d2358a073a37e550b663b5"
    [EOF]
    "#);
}

#[test]
fn test_shell_completions() {
    #[track_caller]
    fn test(shell: &str) {
        let test_env = TestEnvironment::default();
        // Use the local backend because GitBackend::gc() depends on the git CLI.
        let (out, err) = test_env.jj_cmd_ok(test_env.env_root(), &["util", "completion", shell]);
        // Ensures only stdout contains text
        assert!(!out.is_empty());
        assert!(err.is_empty());
    }

    test("bash");
    test("fish");
    test("nushell");
    test("zsh");
}

#[test]
fn test_util_exec() {
    let test_env = TestEnvironment::default();
    let formatter_path = assert_cmd::cargo::cargo_bin("fake-formatter");
    let (out, err) = test_env.jj_cmd_ok(
        test_env.env_root(),
        &[
            "util",
            "exec",
            "--",
            formatter_path.to_str().unwrap(),
            "--append",
            "hello",
        ],
    );
    insta::assert_snapshot!(out, @"hello[EOF]");
    // Ensures only stdout contains text
    assert!(err.is_empty());
}

#[test]
fn test_util_exec_fail() {
    let test_env = TestEnvironment::default();
    let output = test_env.run_jj_in(
        test_env.env_root(),
        ["util", "exec", "--", "jj-test-missing-program"],
    );
    insta::assert_snapshot!(output.strip_stderr_last_line(), @r"
    ------- stderr -------
    Error: Failed to execute external command 'jj-test-missing-program'
    [EOF]
    [exit status: 1]
    ");
}
