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
use std::path::Path;

use test_case::test_case;
use testutils::git;

use crate::common::create_commit;
use crate::common::CommandOutput;
use crate::common::TestEnvironment;

fn add_commit_to_branch(git_repo: &gix::Repository, branch: &str) -> gix::ObjectId {
    git::add_commit(
        git_repo,
        &format!("refs/heads/{branch}"),
        branch,            // filename
        branch.as_bytes(), // content
        "message",
        &[],
    )
    .commit_id
}

/// Creates a remote Git repo containing a bookmark with the same name
fn init_git_remote(test_env: &TestEnvironment, remote: &str) -> gix::Repository {
    let git_repo_path = test_env.env_root().join(remote);
    let git_repo = git::init(git_repo_path);
    add_commit_to_branch(&git_repo, remote);

    git_repo
}

/// Add a remote containing a bookmark with the same name
fn add_git_remote(test_env: &TestEnvironment, repo_path: &Path, remote: &str) -> gix::Repository {
    let repo = init_git_remote(test_env, remote);
    test_env
        .run_jj_in(
            repo_path,
            ["git", "remote", "add", remote, &format!("../{remote}")],
        )
        .success();

    repo
}

#[must_use]
fn get_bookmark_output(test_env: &TestEnvironment, repo_path: &Path) -> CommandOutput {
    // --quiet to suppress deleted bookmarks hint
    test_env.run_jj_in(repo_path, ["bookmark", "list", "--all-remotes", "--quiet"])
}

#[must_use]
fn get_log_output(test_env: &TestEnvironment, workspace_root: &Path) -> CommandOutput {
    let template =
        r#"commit_id.short() ++ " \"" ++ description.first_line() ++ "\" " ++ bookmarks"#;
    test_env.run_jj_in(workspace_root, ["log", "-T", template, "-r", "all()"])
}

fn clone_git_remote_into(
    test_env: &TestEnvironment,
    upstream: &str,
    fork: &str,
) -> gix::Repository {
    let upstream_path = test_env.env_root().join(upstream);
    let fork_path = test_env.env_root().join(fork);
    let fork_repo = git::clone(&fork_path, upstream_path.to_str().unwrap(), Some(upstream));

    // create local branch mirroring the upstream
    let upstream_head = fork_repo
        .find_reference(&format!("refs/remotes/{upstream}/{upstream}"))
        .unwrap()
        .peel_to_id_in_place()
        .unwrap()
        .detach();

    fork_repo
        .reference(
            format!("refs/heads/{upstream}"),
            upstream_head,
            gix::refs::transaction::PreviousValue::MustNotExist,
            "create tracking head",
        )
        .unwrap();

    fork_repo
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_with_default_config(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "origin");

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin@origin: qmyrypzk ab8b299e message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_default_remote(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "origin");

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qmyrypzk ab8b299e message
      @origin: qmyrypzk ab8b299e message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_single_remote(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");

    let output = test_env.run_jj_in(&repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Hint: Fetching from the only existing remote: rem1
    bookmark: rem1@rem1 [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_single_remote_all_remotes_flag(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");

    test_env
        .run_jj_in(&repo_path, ["git", "fetch", "--all-remotes"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_single_remote_from_arg(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");

    test_env
        .run_jj_in(&repo_path, ["git", "fetch", "--remote", "rem1"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_single_remote_from_config(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    test_env.add_config(r#"git.fetch = "rem1""#);

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_multiple_remotes(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");

    test_env
        .run_jj_in(
            &repo_path,
            ["git", "fetch", "--remote", "rem1", "--remote", "rem2"],
        )
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    rem2: pzqqpnpo 44c57802 message
      @rem2: pzqqpnpo 44c57802 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_with_glob(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");

    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--remote", "glob:*"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: rem1@rem1 [new] untracked
    bookmark: rem2@rem2 [new] untracked
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_with_glob_and_exact_match(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");
    add_git_remote(&test_env, &repo_path, "upstream1");
    add_git_remote(&test_env, &repo_path, "upstream2");
    add_git_remote(&test_env, &repo_path, "origin");

    let output = test_env.run_jj_in(
        &repo_path,
        ["git", "fetch", "--remote=glob:rem*", "--remote=origin"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: origin@origin [new] untracked
    bookmark: rem1@rem1     [new] untracked
    bookmark: rem2@rem2     [new] untracked
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_with_glob_from_config(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config(r#"git.fetch = "glob:rem*""#);
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");
    add_git_remote(&test_env, &repo_path, "upstream");

    let output = test_env.run_jj_in(&repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: rem1@rem1 [new] untracked
    bookmark: rem2@rem2 [new] untracked
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_with_glob_with_no_matching_remotes(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "upstream");

    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--remote=glob:rem*"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: No matching git remotes for patterns: rem*
    [EOF]
    [exit status: 1]
    ");
    }
    // No remote should have been fetched as part of the failing transaction
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_all_remotes(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");

    // add empty [remote "rem3"] section to .git/config, which should be ignored
    test_env
        .run_jj_in(&repo_path, ["git", "remote", "add", "rem3", "../unknown"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["git", "remote", "remove", "rem3"])
        .success();

    test_env
        .run_jj_in(&repo_path, ["git", "fetch", "--all-remotes"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    rem2: pzqqpnpo 44c57802 message
      @rem2: pzqqpnpo 44c57802 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_multiple_remotes_from_config(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");
    test_env.add_config(r#"git.fetch = ["rem1", "rem2"]"#);

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    rem2: pzqqpnpo 44c57802 message
      @rem2: pzqqpnpo 44c57802 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_nonexistent_remote(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");

    let output = test_env.run_jj_in(
        &repo_path,
        ["git", "fetch", "--remote", "rem1", "--remote", "rem2"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: No git remote named 'rem2'
    [EOF]
    [exit status: 1]
    ");
    }
    insta::allow_duplicates! {
    // No remote should have been fetched as part of the failing transaction
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_nonexistent_remote_from_config(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");
    test_env.add_config(r#"git.fetch = ["rem1", "rem2"]"#);

    let output = test_env.run_jj_in(&repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: No git remote named 'rem2'
    [EOF]
    [exit status: 1]
    ");
    }
    // No remote should have been fetched as part of the failing transaction
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_from_remote_named_git(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let repo_path = test_env.env_root().join("repo");
    init_git_remote(&test_env, "git");

    git::init(&repo_path);
    git::add_remote(&repo_path, "git", "../git");

    // Existing remote named 'git' shouldn't block the repo initialization.
    test_env
        .run_jj_in(&repo_path, ["git", "init", "--git-repo=."])
        .success();

    // Try fetching from the remote named 'git'.
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--remote=git"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: Git remote named 'git' is reserved for local Git repository
    Hint: Run `jj git remote rename` to give a different name.
    [EOF]
    [exit status: 1]
    ");
    }

    // Fetch remote refs by using the git CLI.
    git::fetch(&repo_path, "git");

    // Implicit import shouldn't fail because of the remote ref.
    let output = test_env.run_jj_in(&repo_path, ["bookmark", "list", "--all-remotes"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @"");
    }

    // Explicit import is an error.
    // (This could be warning if we add mechanism to report ignored refs.)
    insta::allow_duplicates! {
    insta::assert_snapshot!(test_env.run_jj_in(&repo_path, ["git", "import"]), @r"
    ------- stderr -------
    Error: Failed to import refs from underlying Git repo
    Caused by: Git remote named 'git' is reserved for local Git repository
    Hint: Run `jj git remote rename` to give different name.
    [EOF]
    [exit status: 1]
    ");
    }

    // The remote can be renamed, and the ref can be imported.
    test_env
        .run_jj_in(&repo_path, ["git", "remote", "rename", "git", "bar"])
        .success();
    let output = test_env.run_jj_in(&repo_path, ["bookmark", "list", "--all-remotes"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    git: vkponlun 400c483d message
      @bar: vkponlun 400c483d message
      @git: vkponlun 400c483d message
    [EOF]
    ------- stderr -------
    Done importing changes from the underlying Git repo.
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_from_remote_with_slashes(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let repo_path = test_env.env_root().join("repo");
    init_git_remote(&test_env, "source");

    git::init(&repo_path);
    git::add_remote(&repo_path, "slash/origin", "../source");

    // Existing remote with slash shouldn't block the repo initialization.
    test_env
        .run_jj_in(&repo_path, ["git", "init", "--git-repo=."])
        .success();

    // Try fetching from the remote named 'git'.
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--remote=slash/origin"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: Git remotes with slashes are incompatible with jj: slash/origin
    Hint: Run `jj git remote rename` to give a different name.
    [EOF]
    [exit status: 1]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_prune_before_updating_tips(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let git_repo = add_git_remote(&test_env, &repo_path, "origin");
    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qmyrypzk ab8b299e message
      @origin: qmyrypzk ab8b299e message
    [EOF]
    ");
    }

    // Remove origin bookmark in git repo and create origin/subname
    let mut origin_reference = git_repo.find_reference("refs/heads/origin").unwrap();
    let commit_id = origin_reference.peel_to_commit().unwrap().id().detach();
    origin_reference.delete().unwrap();
    git_repo
        .reference(
            "refs/heads/origin/subname",
            commit_id,
            gix::refs::transaction::PreviousValue::MustNotExist,
            "create new reference",
        )
        .unwrap();

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin/subname: qmyrypzk ab8b299e message
      @origin: qmyrypzk ab8b299e message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_conflicting_bookmarks(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "rem1");

    // Create a rem1 bookmark locally
    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "rem1"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: kkmpptxz fcdbbd73 (empty) (no description set)
    [EOF]
    ");
    }

    test_env
        .run_jj_in(
            &repo_path,
            ["git", "fetch", "--remote", "rem1", "--branch", "glob:*"],
        )
        .success();
    // This should result in a CONFLICTED bookmark
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1 (conflicted):
      + kkmpptxz fcdbbd73 (empty) (no description set)
      + ppspxspk 4acd0343 message
      @rem1 (behind by 1 commits): ppspxspk 4acd0343 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_conflicting_bookmarks_colocated(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let repo_path = test_env.env_root().join("repo");
    git::init(&repo_path);
    // create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &repo_path);
    test_env
        .run_jj_in(&repo_path, ["git", "init", "--git-repo", "."])
        .success();
    add_git_remote(&test_env, &repo_path, "rem1");
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }

    // Create a rem1 bookmark locally
    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "rem1"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1: zsuskuln f652c321 (empty) (no description set)
      @git: zsuskuln f652c321 (empty) (no description set)
    [EOF]
    ");
    }

    test_env
        .run_jj_in(
            &repo_path,
            ["git", "fetch", "--remote", "rem1", "--branch", "rem1"],
        )
        .success();
    // This should result in a CONFLICTED bookmark
    // See https://github.com/jj-vcs/jj/pull/1146#discussion_r1112372340 for the bug this tests for.
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    rem1 (conflicted):
      + zsuskuln f652c321 (empty) (no description set)
      + ppspxspk 4acd0343 message
      @git (behind by 1 commits): zsuskuln f652c321 (empty) (no description set)
      @rem1 (behind by 1 commits): ppspxspk 4acd0343 message
    [EOF]
    ");
    }
}

// Helper functions to test obtaining multiple bookmarks at once and changed
// bookmarks
fn create_colocated_repo_and_bookmarks_from_trunk1(
    test_env: &TestEnvironment,
    repo_path: &Path,
) -> String {
    // Create a colocated repo in `source` to populate it more easily
    test_env
        .run_jj_in(repo_path, ["git", "init", "--git-repo", "."])
        .success();
    create_commit(test_env, repo_path, "trunk1", &[]);
    create_commit(test_env, repo_path, "a1", &["trunk1"]);
    create_commit(test_env, repo_path, "a2", &["trunk1"]);
    create_commit(test_env, repo_path, "b", &["trunk1"]);
    format!(
        "   ===== Source git repo contents =====\n{}",
        get_log_output(test_env, repo_path)
    )
}

fn create_trunk2_and_rebase_bookmarks(test_env: &TestEnvironment, repo_path: &Path) -> String {
    create_commit(test_env, repo_path, "trunk2", &["trunk1"]);
    for br in ["a1", "a2", "b"] {
        test_env
            .run_jj_in(repo_path, ["rebase", "-b", br, "-d", "trunk2"])
            .success();
    }
    format!(
        "   ===== Source git repo contents =====\n{}",
        get_log_output(test_env, repo_path)
    )
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_all(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "none()""#);
    let source_git_repo_path = test_env.env_root().join("source");
    git::init(source_git_repo_path.clone());

    // Clone an empty repo. The target repo is a normal `jj` repo, *not* colocated
    let output = test_env.run_jj_in(".", ["git", "clone", "source", "target"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Fetching into new repo in "$TEST_ENV/target"
    Nothing changed.
    [EOF]
    "#);
    }
    let target_jj_repo_path = test_env.env_root().join("target");

    let source_log =
        create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    @  8cc4df9dd488 "b" b
    │ ○  e2a95b19b745 "a2" a2
    ├─╯
    │ ○  dd42071fe1ad "a1" a1
    ├─╯
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Nothing in our repo before the fetch
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @"");
    }
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin     [new] tracked
    bookmark: a2@origin     [new] tracked
    bookmark: b@origin      [new] tracked
    bookmark: trunk1@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @r"
    a1: spvnozwy dd42071f a1
      @origin: spvnozwy dd42071f a1
    a2: qnxtrkvv e2a95b19 a2
      @origin: qnxtrkvv e2a95b19 a2
    b: lnxrmsmo 8cc4df9d b
      @origin: lnxrmsmo 8cc4df9d b
    trunk1: qzywppkx 9929b494 trunk1
      @origin: qzywppkx 9929b494 trunk1
    [EOF]
    ");
        }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // ==== Change both repos ====
    // First, change the target repo:
    let source_log = create_trunk2_and_rebase_bookmarks(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    ○  7c277a6aa3c3 "b" b
    │ ○  698fed8731d8 "a2" a2
    ├─╯
    │ ○  a3f2410627ff "a1" a1
    ├─╯
    @  e7525a4649e3 "trunk2" trunk2
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    // Change a bookmark in the source repo as well, so that it becomes conflicted.
    test_env
        .run_jj_in(
            &target_jj_repo_path,
            ["describe", "b", "-m=new_descr_for_b_to_create_conflict"],
        )
        .success();

    // Our repo before and after fetch
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  5b3bc9c99bb3 "new_descr_for_b_to_create_conflict" b*
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @r"
    a1: spvnozwy dd42071f a1
      @origin: spvnozwy dd42071f a1
    a2: qnxtrkvv e2a95b19 a2
      @origin: qnxtrkvv e2a95b19 a2
    b: lnxrmsmo 5b3bc9c9 new_descr_for_b_to_create_conflict
      @origin (ahead by 1 commits, behind by 1 commits): lnxrmsmo hidden 8cc4df9d b
    trunk1: qzywppkx 9929b494 trunk1
      @origin: qzywppkx 9929b494 trunk1
    [EOF]
    ");
    }
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin     [updated] tracked
    bookmark: a2@origin     [updated] tracked
    bookmark: b@origin      [updated] tracked
    bookmark: trunk2@origin [new] tracked
    Abandoned 2 commits that are no longer reachable.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @r"
    a1: vrnsrlyk a3f24106 a1
      @origin: vrnsrlyk a3f24106 a1
    a2: vlowznwy 698fed87 a2
      @origin: vlowznwy 698fed87 a2
    b (conflicted):
      - lnxrmsmo hidden 8cc4df9d b
      + lnxrmsmo 5b3bc9c9 new_descr_for_b_to_create_conflict
      + uulqyxll 7c277a6a b
      @origin (behind by 1 commits): uulqyxll 7c277a6a b
    trunk1: qzywppkx 9929b494 trunk1
      @origin: qzywppkx 9929b494 trunk1
    trunk2: lzqpwqnx e7525a46 trunk2
      @origin: lzqpwqnx e7525a46 trunk2
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  7c277a6aa3c3 "b" b?? b@origin
    │ │ ○  698fed8731d8 "a2" a2
    │ ├─╯
    │ │ ○  a3f2410627ff "a1" a1
    │ ├─╯
    │ ○  e7525a4649e3 "trunk2" trunk2
    │ │ ○  5b3bc9c99bb3 "new_descr_for_b_to_create_conflict" b??
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_some_of_many_bookmarks(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "none()""#);
    let source_git_repo_path = test_env.env_root().join("source");
    git::init(source_git_repo_path.clone());

    // Clone an empty repo. The target repo is a normal `jj` repo, *not* colocated
    let output = test_env.run_jj_in(".", ["git", "clone", "source", "target"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Fetching into new repo in "$TEST_ENV/target"
    Nothing changed.
    [EOF]
    "#);
    }
    let target_jj_repo_path = test_env.env_root().join("target");

    let source_log =
        create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    @  8cc4df9dd488 "b" b
    │ ○  e2a95b19b745 "a2" a2
    ├─╯
    │ ○  dd42071fe1ad "a1" a1
    ├─╯
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Test an error message
    let output = test_env.run_jj_in(
        &target_jj_repo_path,
        ["git", "fetch", "--branch", "glob:^:a*"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: Invalid branch pattern provided. When fetching, branch names and globs may not contain the characters `:`, `^`, `?`, `[`, `]`
    [EOF]
    [exit status: 1]
    ");
    }
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch", "--branch", "a*"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: Branch names may not include `*`.
    Hint: Prefix the pattern with `glob:` to expand `*` as a glob
    [EOF]
    [exit status: 1]
    ");
    }

    // Nothing in our repo before the fetch
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    // Fetch one bookmark...
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch", "--branch", "b"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: b@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    // ...check what the intermediate state looks like...
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @r"
    b: lnxrmsmo 8cc4df9d b
      @origin: lnxrmsmo 8cc4df9d b
    [EOF]
    ");
    }
    // ...then fetch two others with a glob.
    let output = test_env.run_jj_in(
        &target_jj_repo_path,
        ["git", "fetch", "--branch", "glob:a*"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin [new] tracked
    bookmark: a2@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  e2a95b19b745 "a2" a2
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ │ ○  8cc4df9dd488 "b" b
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    // Fetching the same bookmark again
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch", "--branch", "a1"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Nothing changed.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  e2a95b19b745 "a2" a2
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ │ ○  8cc4df9dd488 "b" b
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // ==== Change both repos ====
    // First, change the target repo:
    let source_log = create_trunk2_and_rebase_bookmarks(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    ○  96c8ad25ad36 "b" b
    │ ○  9ccd9f75fc0c "a2" a2
    ├─╯
    │ ○  527d2e46a87f "a1" a1
    ├─╯
    @  2f34a0e70741 "trunk2" trunk2
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    // Change a bookmark in the source repo as well, so that it becomes conflicted.
    test_env
        .run_jj_in(
            &target_jj_repo_path,
            ["describe", "b", "-m=new_descr_for_b_to_create_conflict"],
        )
        .success();

    // Our repo before and after fetch of two bookmarks
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  e9445259b932 "new_descr_for_b_to_create_conflict" b*
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    let output = test_env.run_jj_in(
        &target_jj_repo_path,
        ["git", "fetch", "--branch", "b", "--branch", "a1"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin [updated] tracked
    bookmark: b@origin  [updated] tracked
    Abandoned 1 commits that are no longer reachable.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  96c8ad25ad36 "b" b?? b@origin
    │ │ ○  527d2e46a87f "a1" a1
    │ ├─╯
    │ ○  2f34a0e70741 "trunk2"
    │ │ ○  e9445259b932 "new_descr_for_b_to_create_conflict" b??
    │ ├─╯
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // We left a2 where it was before, let's see how `jj bookmark list` sees this.
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @r"
    a1: qptloxlm 527d2e46 a1
      @origin: qptloxlm 527d2e46 a1
    a2: qnxtrkvv e2a95b19 a2
      @origin: qnxtrkvv e2a95b19 a2
    b (conflicted):
      - lnxrmsmo hidden 8cc4df9d b
      + lnxrmsmo e9445259 new_descr_for_b_to_create_conflict
      + rruvkzpm 96c8ad25 b
      @origin (behind by 1 commits): rruvkzpm 96c8ad25 b
    [EOF]
    ");
    }
    // Now, let's fetch a2 and double-check that fetching a1 and b again doesn't do
    // anything.
    let output = test_env.run_jj_in(
        &target_jj_repo_path,
        ["git", "fetch", "--branch", "b", "--branch", "glob:a*"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a2@origin [updated] tracked
    Abandoned 1 commits that are no longer reachable.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  9ccd9f75fc0c "a2" a2
    │ │ ○  96c8ad25ad36 "b" b?? b@origin
    │ ├─╯
    │ │ ○  527d2e46a87f "a1" a1
    │ ├─╯
    │ ○  2f34a0e70741 "trunk2"
    │ │ ○  e9445259b932 "new_descr_for_b_to_create_conflict" b??
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &target_jj_repo_path), @r"
    a1: qptloxlm 527d2e46 a1
      @origin: qptloxlm 527d2e46 a1
    a2: ltuqxttq 9ccd9f75 a2
      @origin: ltuqxttq 9ccd9f75 a2
    b (conflicted):
      - lnxrmsmo hidden 8cc4df9d b
      + lnxrmsmo e9445259 new_descr_for_b_to_create_conflict
      + rruvkzpm 96c8ad25 b
      @origin (behind by 1 commits): rruvkzpm 96c8ad25 b
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_bookmarks_some_missing(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "origin");
    add_git_remote(&test_env, &repo_path, "rem1");
    add_git_remote(&test_env, &repo_path, "rem2");
    add_git_remote(&test_env, &repo_path, "rem3");

    // single missing bookmark, implicit remotes (@origin)
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--branch", "noexist"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Warning: No branch matching `noexist` found on any specified/configured remote
    Nothing changed.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }

    // multiple missing bookmarks, implicit remotes (@origin)
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "git", "fetch", "--branch", "noexist1", "--branch", "noexist2",
        ],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Warning: No branch matching `noexist1` found on any specified/configured remote
    Warning: No branch matching `noexist2` found on any specified/configured remote
    Nothing changed.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }

    // single existing bookmark, implicit remotes (@origin)
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--branch", "origin"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: origin@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qmyrypzk ab8b299e message
      @origin: qmyrypzk ab8b299e message
    [EOF]
    ");
    }

    // multiple existing bookmark, explicit remotes, each bookmark is only in one
    // remote.
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "git", "fetch", "--branch", "rem1", "--branch", "rem2", "--branch", "rem3", "--remote",
            "rem1", "--remote", "rem2", "--remote", "rem3",
        ],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: rem1@rem1 [new] tracked
    bookmark: rem2@rem2 [new] tracked
    bookmark: rem3@rem3 [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qmyrypzk ab8b299e message
      @origin: qmyrypzk ab8b299e message
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    rem2: pzqqpnpo 44c57802 message
      @rem2: pzqqpnpo 44c57802 message
    rem3: wrzwlmys 45a3faef message
      @rem3: wrzwlmys 45a3faef message
    [EOF]
    ")
    }

    // multiple bookmarks, one exists, one doesn't
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "git", "fetch", "--branch", "rem1", "--branch", "notexist", "--remote", "rem1",
        ],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Warning: No branch matching `notexist` found on any specified/configured remote
    Nothing changed.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qmyrypzk ab8b299e message
      @origin: qmyrypzk ab8b299e message
    rem1: ppspxspk 4acd0343 message
      @rem1: ppspxspk 4acd0343 message
    rem2: pzqqpnpo 44c57802 message
      @rem2: pzqqpnpo 44c57802 message
    rem3: wrzwlmys 45a3faef message
      @rem3: wrzwlmys 45a3faef message
    [EOF]
    ");
    }
}

#[test]
fn test_git_fetch_bookmarks_missing_with_subprocess_localized_message() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "origin");

    // "fatal: couldn't find remote ref %s" shouldn't be localized.
    let output = test_env.run_jj_with(|cmd| {
        cmd.current_dir(&repo_path)
            .args(["git", "fetch", "--branch=unknown"])
            // Initialize locale as "en_US" which is the most common.
            .env("LC_ALL", "en_US.UTF-8")
            // Set some other locale variables for testing.
            .env("LC_MESSAGES", "en_US.UTF-8")
            .env("LANG", "en_US.UTF-8")
            // GNU gettext prioritizes LANGUAGE if translation is enabled. It works
            // no matter if system locale exists or not.
            .env("LANGUAGE", "zh_TW")
    });
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Warning: No branch matching `unknown` found on any specified/configured remote
    Nothing changed.
    [EOF]
    ");
}

// See `test_undo_restore_commands.rs` for fetch-undo-push and fetch-undo-fetch
// of the same bookmarks for various kinds of undo.
#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_undo(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let source_git_repo_path = test_env.env_root().join("source");
    git::init(source_git_repo_path.clone());

    // Clone an empty repo. The target repo is a normal `jj` repo, *not* colocated
    let output = test_env.run_jj_in(".", ["git", "clone", "source", "target"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Fetching into new repo in "$TEST_ENV/target"
    Nothing changed.
    [EOF]
    "#);
    }
    let target_jj_repo_path = test_env.env_root().join("target");

    let source_log =
        create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    @  8cc4df9dd488 "b" b
    │ ○  e2a95b19b745 "a2" a2
    ├─╯
    │ ○  dd42071fe1ad "a1" a1
    ├─╯
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Fetch 2 bookmarks
    let output = test_env.run_jj_in(
        &target_jj_repo_path,
        ["git", "fetch", "--branch", "b", "--branch", "a1"],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin [new] tracked
    bookmark: b@origin  [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    let output = test_env.run_jj_in(&target_jj_repo_path, ["undo"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Undid operation: 4bd67fb242bc (2001-02-03 08:05:18) fetch from git remote(s) origin
    [EOF]
    ");
    }
    // The undo works as expected
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    // Now try to fetch just one bookmark
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch", "--branch", "b"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: b@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
}

// Compare to `test_git_import_undo` in test_git_import_export
// TODO: Explain why these behaviors are useful
#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_fetch_undo_what(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let source_git_repo_path = test_env.env_root().join("source");
    git::init(source_git_repo_path.clone());

    // Clone an empty repo. The target repo is a normal `jj` repo, *not* colocated
    let output = test_env.run_jj_in(".", ["git", "clone", "source", "target"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Fetching into new repo in "$TEST_ENV/target"
    Nothing changed.
    [EOF]
    "#);
    }
    let repo_path = test_env.env_root().join("target");

    let source_log =
        create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    @  8cc4df9dd488 "b" b
    │ ○  e2a95b19b745 "a2" a2
    ├─╯
    │ ○  dd42071fe1ad "a1" a1
    ├─╯
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Initial state we will try to return to after `op restore`. There are no
    // bookmarks.
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @"");
    }
    let base_operation_id = test_env.current_operation_id(&repo_path);

    // Fetch a bookmark
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--branch", "b"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: b@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    b: lnxrmsmo 8cc4df9d b
      @origin: lnxrmsmo 8cc4df9d b
    [EOF]
    ");
    }

    // We can undo the change in the repo without moving the remote-tracking
    // bookmark
    let output = test_env.run_jj_in(
        &repo_path,
        ["op", "restore", "--what", "repo", &base_operation_id],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Restored to operation: eac759b9ab75 (2001-02-03 08:05:07) add workspace 'default'
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    b (deleted)
      @origin: lnxrmsmo hidden 8cc4df9d b
    [EOF]
    ");
    }

    // Now, let's demo restoring just the remote-tracking bookmark. First, let's
    // change our local repo state...
    test_env
        .run_jj_in(&repo_path, ["bookmark", "c", "-r@", "newbookmark"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    b (deleted)
      @origin: lnxrmsmo hidden 8cc4df9d b
    newbookmark: qpvuntsm 230dd059 (empty) (no description set)
    [EOF]
    ");
    }
    // Restoring just the remote-tracking state will not affect `newbookmark`, but
    // will eliminate `b@origin`.
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "op",
            "restore",
            "--what",
            "remote-tracking",
            &base_operation_id,
        ],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Restored to operation: eac759b9ab75 (2001-02-03 08:05:07) add workspace 'default'
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    newbookmark: qpvuntsm 230dd059 (empty) (no description set)
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_remove_fetch(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "origin");

    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "origin"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qpvuntsm 230dd059 (empty) (no description set)
    [EOF]
    ");
    }

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin (conflicted):
      + qpvuntsm 230dd059 (empty) (no description set)
      + qmyrypzk ab8b299e message
      @origin (behind by 1 commits): qmyrypzk ab8b299e message
    [EOF]
    ");
    }

    test_env
        .run_jj_in(&repo_path, ["git", "remote", "remove", "origin"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin (conflicted):
      + qpvuntsm 230dd059 (empty) (no description set)
      + qmyrypzk ab8b299e message
    [EOF]
    ");
    }

    test_env
        .run_jj_in(&repo_path, ["git", "remote", "add", "origin", "../origin"])
        .success();

    // Check that origin@origin is properly recreated
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: origin@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin (conflicted):
      + qpvuntsm 230dd059 (empty) (no description set)
      + qmyrypzk ab8b299e message
      @origin (behind by 1 commits): qmyrypzk ab8b299e message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_rename_fetch(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    add_git_remote(&test_env, &repo_path, "origin");

    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "origin"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin: qpvuntsm 230dd059 (empty) (no description set)
    [EOF]
    ");
    }

    test_env.run_jj_in(&repo_path, ["git", "fetch"]).success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin (conflicted):
      + qpvuntsm 230dd059 (empty) (no description set)
      + qmyrypzk ab8b299e message
      @origin (behind by 1 commits): qmyrypzk ab8b299e message
    [EOF]
    ");
    }

    test_env
        .run_jj_in(
            &repo_path,
            ["git", "remote", "rename", "origin", "upstream"],
        )
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    origin (conflicted):
      + qpvuntsm 230dd059 (empty) (no description set)
      + qmyrypzk ab8b299e message
      @upstream (behind by 1 commits): qmyrypzk ab8b299e message
    [EOF]
    ");
    }

    // Check that jj indicates that nothing has changed
    let output = test_env.run_jj_in(&repo_path, ["git", "fetch", "--remote", "upstream"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Nothing changed.
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_removed_bookmark(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let source_git_repo_path = test_env.env_root().join("source");
    git::init(source_git_repo_path.clone());

    // Clone an empty repo. The target repo is a normal `jj` repo, *not* colocated
    let output = test_env.run_jj_in(".", ["git", "clone", "source", "target"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Fetching into new repo in "$TEST_ENV/target"
    Nothing changed.
    [EOF]
    "#);
    }
    let target_jj_repo_path = test_env.env_root().join("target");

    let source_log =
        create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    @  8cc4df9dd488 "b" b
    │ ○  e2a95b19b745 "a2" a2
    ├─╯
    │ ○  dd42071fe1ad "a1" a1
    ├─╯
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Fetch all bookmarks
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin     [new] tracked
    bookmark: a2@origin     [new] tracked
    bookmark: b@origin      [new] tracked
    bookmark: trunk1@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Remove a2 bookmark in origin
    test_env
        .run_jj_in(
            &source_git_repo_path,
            ["bookmark", "forget", "--include-remotes", "a2"],
        )
        .success();

    // Fetch bookmark a1 from origin and check that a2 is still there
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch", "--branch", "a1"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Nothing changed.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Fetch bookmarks a2 from origin, and check that it has been removed locally
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch", "--branch", "a2"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a2@origin [deleted] untracked
    Abandoned 1 commits that are no longer reachable.
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_removed_parent_bookmark(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    let source_git_repo_path = test_env.env_root().join("source");
    git::init(source_git_repo_path.clone());

    // Clone an empty repo. The target repo is a normal `jj` repo, *not* colocated
    let output = test_env.run_jj_in(".", ["git", "clone", "source", "target"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Fetching into new repo in "$TEST_ENV/target"
    Nothing changed.
    [EOF]
    "#);
    }
    let target_jj_repo_path = test_env.env_root().join("target");

    let source_log =
        create_colocated_repo_and_bookmarks_from_trunk1(&test_env, &source_git_repo_path);
    insta::allow_duplicates! {
    insta::assert_snapshot!(source_log, @r#"
       ===== Source git repo contents =====
    @  8cc4df9dd488 "b" b
    │ ○  e2a95b19b745 "a2" a2
    ├─╯
    │ ○  dd42071fe1ad "a1" a1
    ├─╯
    ○  9929b494c411 "trunk1" trunk1
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Fetch all bookmarks
    let output = test_env.run_jj_in(&target_jj_repo_path, ["git", "fetch"]);
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin     [new] tracked
    bookmark: a2@origin     [new] tracked
    bookmark: b@origin      [new] tracked
    bookmark: trunk1@origin [new] tracked
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ │ ○  dd42071fe1ad "a1" a1
    │ ├─╯
    │ ○  9929b494c411 "trunk1" trunk1
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }

    // Remove all bookmarks in origin.
    test_env
        .run_jj_in(
            &source_git_repo_path,
            ["bookmark", "forget", "--include-remotes", "glob:*"],
        )
        .success();

    // Fetch bookmarks master, trunk1 and a1 from origin and check that only those
    // bookmarks have been removed and that others were not rebased because of
    // abandoned commits.
    let output = test_env.run_jj_in(
        &target_jj_repo_path,
        [
            "git", "fetch", "--branch", "master", "--branch", "trunk1", "--branch", "a1",
        ],
    );
    insta::allow_duplicates! {
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    bookmark: a1@origin     [deleted] untracked
    bookmark: trunk1@origin [deleted] untracked
    Abandoned 1 commits that are no longer reachable.
    Warning: No branch matching `master` found on any specified/configured remote
    [EOF]
    ");
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &target_jj_repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  8cc4df9dd488 "b" b
    │ │ ○  e2a95b19b745 "a2" a2
    │ ├─╯
    │ ○  9929b494c411 "trunk1"
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_remote_only_bookmark(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    // Create non-empty git repo to add as a remote
    let git_repo_path = test_env.env_root().join("git-repo");
    let git_repo = git::init(git_repo_path);
    test_env
        .run_jj_in(
            &repo_path,
            ["git", "remote", "add", "origin", "../git-repo"],
        )
        .success();

    // Create a commit and a bookmark in the git repo
    let commit_result = git::add_commit(
        &git_repo,
        "refs/heads/feature1",
        "file",
        b"content",
        "message",
        &[],
    );

    // Fetch using git.auto_local_bookmark = true
    test_env.add_config("git.auto-local-bookmark = true");
    test_env
        .run_jj_in(&repo_path, ["git", "fetch", "--remote=origin"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    feature1: qomsplrm ebeb70d8 message
      @origin: qomsplrm ebeb70d8 message
    [EOF]
    ");
    }

    git::write_commit(
        &git_repo,
        "refs/heads/feature2",
        commit_result.tree_id,
        "message",
        &[],
    );

    // Fetch using git.auto_local_bookmark = false
    test_env.add_config("git.auto-local-bookmark = false");
    test_env
        .run_jj_in(&repo_path, ["git", "fetch", "--remote=origin"])
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &repo_path), @r#"
    @  230dd059e1b0 ""
    │ ◆  ebeb70d8c5f9 "message" feature1 feature2@origin
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    feature1: qomsplrm ebeb70d8 message
      @origin: qomsplrm ebeb70d8 message
    feature2@origin: qomsplrm ebeb70d8 message
    [EOF]
    ");
    }
}

#[cfg_attr(feature = "git2", test_case(false; "use git2 for remote calls"))]
#[test_case(true; "spawn a git subprocess for remote calls")]
fn test_git_fetch_preserve_commits_across_repos(subprocess: bool) {
    let test_env = TestEnvironment::default();
    if !subprocess {
        test_env.add_config("git.subprocess = false");
    }
    test_env.add_config("git.auto-local-bookmark = true");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let upstream_repo = add_git_remote(&test_env, &repo_path, "upstream");

    let fork_path = test_env.env_root().join("fork");
    let fork_repo = clone_git_remote_into(&test_env, "upstream", "fork");
    test_env
        .run_jj_in(&repo_path, ["git", "remote", "add", "fork", "../fork"])
        .success();

    // add commit to fork remote in another branch
    add_commit_to_branch(&fork_repo, "feature");

    // fetch remote bookmarks
    test_env
        .run_jj_in(
            &repo_path,
            ["git", "fetch", "--remote=fork", "--remote=upstream"],
        )
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○  bcd7cd779791 "message" upstream
    ├─╯
    │ ○  16ec9ef2877a "message" feature
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    feature: srwrtuky 16ec9ef2 message
      @fork: srwrtuky 16ec9ef2 message
    upstream: zkvzklqn bcd7cd77 message
      @fork: zkvzklqn bcd7cd77 message
      @upstream: zkvzklqn bcd7cd77 message
    [EOF]
    ");
    }

    // merge fork/feature into the upstream/upstream
    git::add_remote(upstream_repo.git_dir(), "fork", fork_path.to_str().unwrap());
    git::fetch(upstream_repo.git_dir(), "fork");

    let base_id = upstream_repo
        .find_reference("refs/heads/upstream")
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id()
        .detach();

    let fork_id = upstream_repo
        .find_reference("refs/remotes/fork/feature")
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id()
        .detach();

    git::write_commit(
        &upstream_repo,
        "refs/heads/upstream",
        upstream_repo.empty_tree().id().detach(),
        "merge",
        &[base_id, fork_id],
    );

    // remove branch on the fork
    fork_repo
        .find_reference("refs/heads/feature")
        .unwrap()
        .delete()
        .unwrap();

    // fetch again on the jj repo, first looking at fork and then at upstream
    test_env
        .run_jj_in(
            &repo_path,
            ["git", "fetch", "--remote=fork", "--remote=upstream"],
        )
        .success();
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_log_output(&test_env, &repo_path), @r#"
    @  230dd059e1b0 ""
    │ ○    f3e9250bd003 "merge" upstream*
    │ ├─╮
    │ │ ○  16ec9ef2877a "message"
    ├───╯
    │ ○  bcd7cd779791 "message" upstream@fork
    ├─╯
    ◆  000000000000 ""
    [EOF]
    "#);
    }
    insta::allow_duplicates! {
    insta::assert_snapshot!(get_bookmark_output(&test_env, &repo_path), @r"
    upstream: trrkvuqr f3e9250b merge
      @fork (behind by 2 commits): zkvzklqn bcd7cd77 message
      @upstream: trrkvuqr f3e9250b merge
    [EOF]
    ");
    }
}
