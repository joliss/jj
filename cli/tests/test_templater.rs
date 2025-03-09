// Copyright 2022 The Jujutsu Authors
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

use std::path::Path;

use indoc::indoc;

use crate::common::CommandOutput;
use crate::common::TestEnvironment;

#[test]
fn test_templater_parse_error() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |template| get_template_output(&test_env, &repo_path, "@-", template);

    insta::assert_snapshot!(render(r#"description ()"#), @r"
    ------- stderr -------
    Error: Failed to parse template: Syntax error
    Caused by:  --> 1:13
      |
    1 | description ()
      |             ^---
      |
      = expected <EOI>, `++`, `||`, `&&`, `==`, `!=`, `>=`, `>`, `<=`, or `<`
    [exit status: 1]
    ");

    // Typo
    test_env.add_config(
        r###"
    [template-aliases]
    'conflicting' = ''
    'shorted()' = ''
    'socat(x)' = 'x'
    'format_id(id)' = 'id.sort()'
    "###,
    );
    insta::assert_snapshot!(render(r#"conflicts"#), @r"
    ------- stderr -------
    Error: Failed to parse template: Keyword `conflicts` doesn't exist
    Caused by:  --> 1:1
      |
    1 | conflicts
      | ^-------^
      |
      = Keyword `conflicts` doesn't exist
    Hint: Did you mean `conflict`, `conflicting`?
    [exit status: 1]
    ");
    insta::assert_snapshot!(render(r#"commit_id.shorter()"#), @r"
    ------- stderr -------
    Error: Failed to parse template: Method `shorter` doesn't exist for type `CommitOrChangeId`
    Caused by:  --> 1:11
      |
    1 | commit_id.shorter()
      |           ^-----^
      |
      = Method `shorter` doesn't exist for type `CommitOrChangeId`
    Hint: Did you mean `short`, `shortest`?
    [exit status: 1]
    ");
    insta::assert_snapshot!(render(r#"oncat()"#), @r"
    ------- stderr -------
    Error: Failed to parse template: Function `oncat` doesn't exist
    Caused by:  --> 1:1
      |
    1 | oncat()
      | ^---^
      |
      = Function `oncat` doesn't exist
    Hint: Did you mean `concat`, `socat`?
    [exit status: 1]
    ");
    insta::assert_snapshot!(render(r#""".lines().map(|s| se)"#), @r#"
    ------- stderr -------
    Error: Failed to parse template: Keyword `se` doesn't exist
    Caused by:  --> 1:20
      |
    1 | "".lines().map(|s| se)
      |                    ^^
      |
      = Keyword `se` doesn't exist
    Hint: Did you mean `s`, `self`?
    [exit status: 1]
    "#);
    insta::assert_snapshot!(render(r#"format_id(commit_id)"#), @r"
    ------- stderr -------
    Error: Failed to parse template: In alias `format_id(id)`
    Caused by:
    1:  --> 1:1
      |
    1 | format_id(commit_id)
      | ^------------------^
      |
      = In alias `format_id(id)`
    2:  --> 1:4
      |
    1 | id.sort()
      |    ^--^
      |
      = Method `sort` doesn't exist for type `CommitOrChangeId`
    Hint: Did you mean `short`, `shortest`?
    [exit status: 1]
    ");

    // "at least N arguments"
    insta::assert_snapshot!(render("separate()"), @r"
    ------- stderr -------
    Error: Failed to parse template: Function `separate`: Expected at least 1 arguments
    Caused by:  --> 1:10
      |
    1 | separate()
      |          ^
      |
      = Function `separate`: Expected at least 1 arguments
    [exit status: 1]
    ");

    // -Tbuiltin shows the predefined builtin_* aliases. This isn't 100%
    // guaranteed, but is nice.
    insta::assert_snapshot!(render(r#"builtin"#), @r"
    ------- stderr -------
    Error: Failed to parse template: Keyword `builtin` doesn't exist
    Caused by:  --> 1:1
      |
    1 | builtin
      | ^-----^
      |
      = Keyword `builtin` doesn't exist
    Hint: Did you mean `builtin_log_comfortable`, `builtin_log_compact`, `builtin_log_compact_full_description`, `builtin_log_detailed`, `builtin_log_node`, `builtin_log_node_ascii`, `builtin_log_oneline`, `builtin_op_log_comfortable`, `builtin_op_log_compact`, `builtin_op_log_node`, `builtin_op_log_node_ascii`, `builtin_op_log_oneline`?
    [exit status: 1]
    ");
}

#[test]
fn test_template_parse_warning() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let template = indoc! {r#"
        separate(' ',
          author.username(),
        )
    "#};
    let output = test_env.run_jj_in(&repo_path, ["log", "-r@", "-T", template]);
    insta::assert_snapshot!(output, @r"
    @  test.user
    │
    ~
    ------- stderr -------
    Warning: In template expression
     --> 2:10
      |
    2 |   author.username(),
      |          ^------^
      |
      = username() is deprecated; use email().local() instead
    ");
}

#[test]
fn test_templater_upper_lower() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |template| get_colored_template_output(&test_env, &repo_path, "@-", template);

    insta::assert_snapshot!(
        render(r#"change_id.shortest(4).upper() ++ change_id.shortest(4).upper().lower()"#),
        @"\u{1b}[1m\u{1b}[38;5;5mZ\u{1b}[0m\u{1b}[38;5;8mZZZ\u{1b}[1m\u{1b}[38;5;5mz\u{1b}[0m\u{1b}[38;5;8mzzz\u{1b}[39m[no newline]");
    insta::assert_snapshot!(
        render(r#""Hello".upper() ++ "Hello".lower()"#), @"HELLOhello[no newline]");
}

#[test]
fn test_templater_alias() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |template| get_template_output(&test_env, &repo_path, "@-", template);

    test_env.add_config(
        r###"
    [template-aliases]
    'my_commit_id' = 'commit_id.short()'
    'syntax_error' = 'foo.'
    'name_error' = 'unknown_id'
    'recurse' = 'recurse1'
    'recurse1' = 'recurse2()'
    'recurse2()' = 'recurse'
    'identity(x)' = 'x'
    'coalesce(x, y)' = 'if(x, x, y)'
    'deprecated()' = 'author.username()'
    'builtin_log_node' = '"#"'
    'builtin_op_log_node' = '"#"'
    "###,
    );

    insta::assert_snapshot!(render("my_commit_id"), @"000000000000[no newline]");
    insta::assert_snapshot!(render("identity(my_commit_id)"), @"000000000000[no newline]");

    insta::assert_snapshot!(render("commit_id ++ syntax_error"), @r"
    ------- stderr -------
    Error: Failed to parse template: In alias `syntax_error`
    Caused by:
    1:  --> 1:14
      |
    1 | commit_id ++ syntax_error
      |              ^----------^
      |
      = In alias `syntax_error`
    2:  --> 1:5
      |
    1 | foo.
      |     ^---
      |
      = expected <identifier>
    [exit status: 1]
    ");

    insta::assert_snapshot!(render("commit_id ++ name_error"), @r"
    ------- stderr -------
    Error: Failed to parse template: In alias `name_error`
    Caused by:
    1:  --> 1:14
      |
    1 | commit_id ++ name_error
      |              ^--------^
      |
      = In alias `name_error`
    2:  --> 1:1
      |
    1 | unknown_id
      | ^--------^
      |
      = Keyword `unknown_id` doesn't exist
    [exit status: 1]
    ");

    insta::assert_snapshot!(render(r#"identity(identity(commit_id.short("")))"#), @r#"
    ------- stderr -------
    Error: Failed to parse template: In alias `identity(x)`
    Caused by:
    1:  --> 1:1
      |
    1 | identity(identity(commit_id.short("")))
      | ^-------------------------------------^
      |
      = In alias `identity(x)`
    2:  --> 1:1
      |
    1 | x
      | ^
      |
      = In function parameter `x`
    3:  --> 1:10
      |
    1 | identity(identity(commit_id.short("")))
      |          ^---------------------------^
      |
      = In alias `identity(x)`
    4:  --> 1:1
      |
    1 | x
      | ^
      |
      = In function parameter `x`
    5:  --> 1:35
      |
    1 | identity(identity(commit_id.short("")))
      |                                   ^^
      |
      = Expected expression of type `Integer`, but actual type is `String`
    [exit status: 1]
    "#);

    insta::assert_snapshot!(render("commit_id ++ recurse"), @r"
    ------- stderr -------
    Error: Failed to parse template: In alias `recurse`
    Caused by:
    1:  --> 1:14
      |
    1 | commit_id ++ recurse
      |              ^-----^
      |
      = In alias `recurse`
    2:  --> 1:1
      |
    1 | recurse1
      | ^------^
      |
      = In alias `recurse1`
    3:  --> 1:1
      |
    1 | recurse2()
      | ^--------^
      |
      = In alias `recurse2()`
    4:  --> 1:1
      |
    1 | recurse
      | ^-----^
      |
      = Alias `recurse` expanded recursively
    [exit status: 1]
    ");

    insta::assert_snapshot!(render("identity()"), @r"
    ------- stderr -------
    Error: Failed to parse template: Function `identity`: Expected 1 arguments
    Caused by:  --> 1:10
      |
    1 | identity()
      |          ^
      |
      = Function `identity`: Expected 1 arguments
    [exit status: 1]
    ");
    insta::assert_snapshot!(render("identity(commit_id, commit_id)"), @r"
    ------- stderr -------
    Error: Failed to parse template: Function `identity`: Expected 1 arguments
    Caused by:  --> 1:10
      |
    1 | identity(commit_id, commit_id)
      |          ^------------------^
      |
      = Function `identity`: Expected 1 arguments
    [exit status: 1]
    ");

    insta::assert_snapshot!(render(r#"coalesce(label("x", "not boolean"), "")"#), @r#"
    ------- stderr -------
    Error: Failed to parse template: In alias `coalesce(x, y)`
    Caused by:
    1:  --> 1:1
      |
    1 | coalesce(label("x", "not boolean"), "")
      | ^-------------------------------------^
      |
      = In alias `coalesce(x, y)`
    2:  --> 1:4
      |
    1 | if(x, x, y)
      |    ^
      |
      = In function parameter `x`
    3:  --> 1:10
      |
    1 | coalesce(label("x", "not boolean"), "")
      |          ^-----------------------^
      |
      = Expected expression of type `Boolean`, but actual type is `Template`
    [exit status: 1]
    "#);

    insta::assert_snapshot!(render("(-my_commit_id)"), @r"
    ------- stderr -------
    Error: Failed to parse template: In alias `my_commit_id`
    Caused by:
    1:  --> 1:3
      |
    1 | (-my_commit_id)
      |   ^----------^
      |
      = In alias `my_commit_id`
    2:  --> 1:1
      |
    1 | commit_id.short()
      | ^---------------^
      |
      = Expected expression of type `Integer`, but actual type is `String`
    [exit status: 1]
    ");

    let output = test_env.run_jj_in(&repo_path, ["log", "-r@", "-Tdeprecated()"]);
    insta::assert_snapshot!(output, @r"
    #  test.user
    │
    ~
    ------- stderr -------
    Warning: In template expression
     --> 1:1
      |
    1 | deprecated()
      | ^----------^
      |
      = In alias `deprecated()`
     --> 1:8
      |
    1 | author.username()
      |        ^------^
      |
      = username() is deprecated; use email().local() instead
    ");
}

#[test]
fn test_templater_alias_override() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env.add_config(
        r#"
    [template-aliases]
    'f(x)' = '"user"'
    "#,
    );

    // 'f(x)' should be overridden by --config 'f(a)'. If aliases were sorted
    // purely by name, 'f(a)' would come first.
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "--no-graph",
            "-r@",
            "-T",
            r#"f(_)"#,
            r#"--config=template-aliases.'f(a)'='"arg"'"#,
        ],
    );
    insta::assert_snapshot!(output, @"arg[no newline]");
}

#[test]
fn test_templater_bad_alias_decl() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env.add_config(
        r###"
    [template-aliases]
    'badfn(a, a)' = 'a'
    'my_commit_id' = 'commit_id.short()'
    "###,
    );

    // Invalid declaration should be warned and ignored.
    let output = test_env.run_jj_in(&repo_path, ["log", "--no-graph", "-r@-", "-Tmy_commit_id"]);
    insta::assert_snapshot!(output, @r"
    000000000000[no newline]
    ------- stderr -------
    Warning: Failed to load `template-aliases.badfn(a, a)`:  --> 1:7
      |
    1 | badfn(a, a)
      |       ^--^
      |
      = Redefinition of function parameter
    ");
}

#[test]
fn test_templater_config_function() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |template| get_template_output(&test_env, &repo_path, "@-", template);

    insta::assert_snapshot!(
        render("config('user.name')"),
        @r#""Test User"[no newline]"#);
    insta::assert_snapshot!(
        render("config('user')"),
        @r#"{ email = "test.user@example.com", name = "Test User" }[no newline]"#);
    insta::assert_snapshot!(render("config('invalid name')"), @r"
    ------- stderr -------
    Error: Failed to parse template: Failed to parse config name
    Caused by:
    1:  --> 1:8
      |
    1 | config('invalid name')
      |        ^------------^
      |
      = Failed to parse config name
    2: TOML parse error at line 1, column 9
      |
    1 | invalid name
      |         ^


    [exit status: 1]
    ");
    insta::assert_snapshot!(render("config('unknown')"), @r"
    ------- stderr -------
    Error: Failed to parse template: Failed to get config value
    Caused by:
    1:  --> 1:1
      |
    1 | config('unknown')
      | ^----^
      |
      = Failed to get config value
    2: Value not found for unknown
    [exit status: 1]
    ");
}

#[must_use]
fn get_template_output(
    test_env: &TestEnvironment,
    repo_path: &Path,
    rev: &str,
    template: &str,
) -> CommandOutput {
    test_env.run_jj_in(repo_path, ["log", "--no-graph", "-r", rev, "-T", template])
}

#[must_use]
fn get_colored_template_output(
    test_env: &TestEnvironment,
    repo_path: &Path,
    rev: &str,
    template: &str,
) -> CommandOutput {
    test_env.run_jj_in(
        repo_path,
        [
            "log",
            "--color=always",
            "--no-graph",
            "-r",
            rev,
            "-T",
            template,
        ],
    )
}
