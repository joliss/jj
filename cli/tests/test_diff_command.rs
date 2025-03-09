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

use indoc::indoc;
use itertools::Itertools;

use crate::common::create_commit;
use crate::common::fake_diff_editor_path;
use crate::common::to_toml_value;
use crate::common::CommandOutput;
use crate::common::TestEnvironment;

#[test]
fn test_diff_basic() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file2"), "1\n2\n3\n4\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1")).unwrap();
    std::fs::write(repo_path.join("file2"), "1\n5\n3\n").unwrap();
    std::fs::write(repo_path.join("file3"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file4"), "1\n2\n3\n4\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file2:
       1    1: 1
       2    2: 25
       3    3: 3
       4     : 4
    Modified regular file file3 (file1 => file3):
    Modified regular file file4 (file2 => file4):
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--context=0"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file2:
       1    1: 1
       2    2: 25
       3    3: 3
       4     : 4
    Modified regular file file3 (file1 => file3):
    Modified regular file file4 (file2 => file4):
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--color=debug"]);
    insta::assert_snapshot!(output, @r"
    [38;5;3m<<diff header::Modified regular file file2:>>[39m
    [38;5;1m<<diff removed line_number::   1>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   1>>[39m<<diff::: 1>>
    [38;5;1m<<diff removed line_number::   2>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   2>>[39m<<diff::: >>[4m[38;5;1m<<diff removed token::2>>[38;5;2m<<diff added token::5>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   3>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   3>>[39m<<diff::: 3>>
    [38;5;1m<<diff removed line_number::   4>>[39m<<diff::     : >>[4m[38;5;1m<<diff removed token::4>>[24m[39m
    [38;5;3m<<diff header::Modified regular file file3 (file1 => file3):>>[39m
    [38;5;3m<<diff header::Modified regular file file4 (file2 => file4):>>[39m
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "-s"]);
    insta::assert_snapshot!(output, @r"
    M file2
    R {file1 => file3}
    C {file2 => file4}
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--types"]);
    insta::assert_snapshot!(output, @r"
    FF file2
    FF {file1 => file3}
    FF {file2 => file4}
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--types", "glob:file[12]"]);
    insta::assert_snapshot!(output, @r"
    F- file1
    FF file2
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "file1"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    deleted file mode 100644
    index 257cc5642c..0000000000
    --- a/file1
    +++ /dev/null
    @@ -1,1 +0,0 @@
    -foo
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file2 b/file2
    index 94ebaf9001..1ffc51b472 100644
    --- a/file2
    +++ b/file2
    @@ -1,4 +1,3 @@
     1
    -2
    +5
     3
    -4
    diff --git a/file1 b/file3
    rename from file1
    rename to file3
    diff --git a/file2 b/file4
    copy from file2
    copy to file4
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--context=0"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file2 b/file2
    index 94ebaf9001..1ffc51b472 100644
    --- a/file2
    +++ b/file2
    @@ -2,1 +2,1 @@
    -2
    +5
    @@ -4,1 +3,0 @@
    -4
    diff --git a/file1 b/file3
    rename from file1
    rename to file3
    diff --git a/file2 b/file4
    copy from file2
    copy to file4
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--color=debug"]);
    insta::assert_snapshot!(output, @r"
    [1m<<diff file_header::diff --git a/file2 b/file2>>[0m
    [1m<<diff file_header::index 94ebaf9001..1ffc51b472 100644>>[0m
    [1m<<diff file_header::--- a/file2>>[0m
    [1m<<diff file_header::+++ b/file2>>[0m
    [38;5;6m<<diff hunk_header::@@ -1,4 +1,3 @@>>[39m
    <<diff context:: 1>>
    [38;5;1m<<diff removed::->>[4m<<diff removed token::2>>[24m<<diff removed::>>[39m
    [38;5;2m<<diff added::+>>[4m<<diff added token::5>>[24m<<diff added::>>[39m
    <<diff context:: 3>>
    [38;5;1m<<diff removed::->>[4m<<diff removed token::4>>[24m[39m
    [1m<<diff file_header::diff --git a/file1 b/file3>>[0m
    [1m<<diff file_header::rename from file1>>[0m
    [1m<<diff file_header::rename to file3>>[0m
    [1m<<diff file_header::diff --git a/file2 b/file4>>[0m
    [1m<<diff file_header::copy from file2>>[0m
    [1m<<diff file_header::copy to file4>>[0m
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "-s", "--git"]);
    insta::assert_snapshot!(output, @r"
    M file2
    R {file1 => file3}
    C {file2 => file4}
    diff --git a/file2 b/file2
    index 94ebaf9001..1ffc51b472 100644
    --- a/file2
    +++ b/file2
    @@ -1,4 +1,3 @@
     1
    -2
    +5
     3
    -4
    diff --git a/file1 b/file3
    rename from file1
    rename to file3
    diff --git a/file2 b/file4
    copy from file2
    copy to file4
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    file2            | 3 +--
    {file1 => file3} | 0
    {file2 => file4} | 0
    3 files changed, 1 insertion(+), 2 deletions(-)
    [EOF]
    ");

    // Filter by glob pattern
    let output = test_env.run_jj_in(&repo_path, ["diff", "-s", "glob:file[12]"]);
    insta::assert_snapshot!(output, @r"
    D file1
    M file2
    [EOF]
    ");

    // Unmatched paths should generate warnings
    let output = test_env.run_jj_in(
        ".",
        [
            "diff",
            "-Rrepo",
            "-s",
            "repo",       // matches directory
            "repo/file1", // deleted in to_tree, but exists in from_tree
            "repo/x",
            "repo/y/z",
        ],
    );
    insta::assert_snapshot!(output.normalize_backslash(), @r"
    M repo/file2
    R repo/{file1 => file3}
    C repo/{file2 => file4}
    [EOF]
    ------- stderr -------
    Warning: No matching entries for paths: repo/x, repo/y/z
    [EOF]
    ");

    // Unmodified paths shouldn't generate warnings
    let output = test_env.run_jj_in(&repo_path, ["diff", "-s", "--from=@", "file2"]);
    insta::assert_snapshot!(output, @"");
}

#[test]
fn test_diff_empty() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "").unwrap();
    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Added regular file file1:
        (empty)
    [EOF]
    ");

    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1")).unwrap();
    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Removed regular file file1:
        (empty)
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    file1 | 0
    1 file changed, 0 insertions(+), 0 deletions(-)
    [EOF]
    ");
}

#[test]
fn test_diff_file_mode() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    // Test content+mode/mode-only changes of empty/non-empty files:
    // - file1: ("",  x) -> ("2", n)  empty, content+mode
    // - file2: ("1", x) -> ("1", n)  non-empty, mode-only
    // - file3: ("1", n) -> ("2", x)  non-empty, content+mode
    // - file4: ("",  n) -> ("",  x)  empty, mode-only

    std::fs::write(repo_path.join("file1"), "").unwrap();
    std::fs::write(repo_path.join("file2"), "1\n").unwrap();
    std::fs::write(repo_path.join("file3"), "1\n").unwrap();
    std::fs::write(repo_path.join("file4"), "").unwrap();
    test_env
        .run_jj_in(&repo_path, ["file", "chmod", "x", "file1", "file2"])
        .success();

    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "2\n").unwrap();
    std::fs::write(repo_path.join("file3"), "2\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["file", "chmod", "n", "file1", "file2"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["file", "chmod", "x", "file3", "file4"])
        .success();

    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1")).unwrap();
    std::fs::remove_file(repo_path.join("file2")).unwrap();
    std::fs::remove_file(repo_path.join("file3")).unwrap();
    std::fs::remove_file(repo_path.join("file4")).unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff", "-r@--"]);
    insta::assert_snapshot!(output, @r"
    Added executable file file1:
        (empty)
    Added executable file file2:
            1: 1
    Added regular file file3:
            1: 1
    Added regular file file4:
        (empty)
    [EOF]
    ");
    let output = test_env.run_jj_in(&repo_path, ["diff", "-r@-"]);
    insta::assert_snapshot!(output, @r"
    Executable file became non-executable at file1:
            1: 2
    Executable file became non-executable at file2:
    Non-executable file became executable at file3:
       1    1: 12
    Non-executable file became executable at file4:
    [EOF]
    ");
    let output = test_env.run_jj_in(&repo_path, ["diff", "-r@"]);
    insta::assert_snapshot!(output, @r"
    Removed regular file file1:
       1     : 2
    Removed regular file file2:
       1     : 1
    Removed executable file file3:
       1     : 2
    Removed executable file file4:
        (empty)
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "-r@--", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    new file mode 100755
    index 0000000000..e69de29bb2
    diff --git a/file2 b/file2
    new file mode 100755
    index 0000000000..d00491fd7e
    --- /dev/null
    +++ b/file2
    @@ -0,0 +1,1 @@
    +1
    diff --git a/file3 b/file3
    new file mode 100644
    index 0000000000..d00491fd7e
    --- /dev/null
    +++ b/file3
    @@ -0,0 +1,1 @@
    +1
    diff --git a/file4 b/file4
    new file mode 100644
    index 0000000000..e69de29bb2
    [EOF]
    ");
    let output = test_env.run_jj_in(&repo_path, ["diff", "-r@-", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    old mode 100755
    new mode 100644
    index e69de29bb2..0cfbf08886
    --- a/file1
    +++ b/file1
    @@ -0,0 +1,1 @@
    +2
    diff --git a/file2 b/file2
    old mode 100755
    new mode 100644
    diff --git a/file3 b/file3
    old mode 100644
    new mode 100755
    index d00491fd7e..0cfbf08886
    --- a/file3
    +++ b/file3
    @@ -1,1 +1,1 @@
    -1
    +2
    diff --git a/file4 b/file4
    old mode 100644
    new mode 100755
    [EOF]
    ");
    let output = test_env.run_jj_in(&repo_path, ["diff", "-r@", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    deleted file mode 100644
    index 0cfbf08886..0000000000
    --- a/file1
    +++ /dev/null
    @@ -1,1 +0,0 @@
    -2
    diff --git a/file2 b/file2
    deleted file mode 100644
    index d00491fd7e..0000000000
    --- a/file2
    +++ /dev/null
    @@ -1,1 +0,0 @@
    -1
    diff --git a/file3 b/file3
    deleted file mode 100755
    index 0cfbf08886..0000000000
    --- a/file3
    +++ /dev/null
    @@ -1,1 +0,0 @@
    -2
    diff --git a/file4 b/file4
    deleted file mode 100755
    index e69de29bb2..0000000000
    [EOF]
    ");
}

#[test]
fn test_diff_types() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let file_path = repo_path.join("foo");

    // Missing
    test_env
        .run_jj_in(&repo_path, ["new", "root()", "-m=missing"])
        .success();

    // Normal file
    test_env
        .run_jj_in(&repo_path, ["new", "root()", "-m=file"])
        .success();
    std::fs::write(&file_path, "foo").unwrap();

    // Conflict (add/add)
    test_env
        .run_jj_in(&repo_path, ["new", "root()", "-m=conflict"])
        .success();
    std::fs::write(&file_path, "foo").unwrap();
    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    std::fs::write(&file_path, "bar").unwrap();
    test_env
        .run_jj_in(&repo_path, ["squash", r#"--into=description("conflict")"#])
        .success();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;

        // Executable
        test_env
            .run_jj_in(&repo_path, ["new", "root()", "-m=executable"])
            .success();
        std::fs::write(&file_path, "foo").unwrap();
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).unwrap();

        // Symlink
        test_env
            .run_jj_in(&repo_path, ["new", "root()", "-m=symlink"])
            .success();
        std::os::unix::fs::symlink(PathBuf::from("."), &file_path).unwrap();
    }

    let diff = |from: &str, to: &str| {
        test_env.run_jj_in(
            &repo_path,
            [
                "diff",
                "--types",
                &format!(r#"--from=description("{from}")"#),
                &format!(r#"--to=description("{to}")"#),
            ],
        )
    };
    insta::assert_snapshot!(diff("missing", "file"), @r"
    -F foo
    [EOF]
    ");
    insta::assert_snapshot!(diff("file", "conflict"), @r"
    FC foo
    [EOF]
    ");
    insta::assert_snapshot!(diff("conflict", "missing"), @r"
    C- foo
    [EOF]
    ");

    #[cfg(unix)]
    {
        insta::assert_snapshot!(diff("symlink", "file"), @r"
        LF foo
        [EOF]
        ");
        insta::assert_snapshot!(diff("missing", "executable"), @r"
        -F foo
        [EOF]
        ");
    }
}

#[test]
fn test_diff_name_only() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("deleted"), "d").unwrap();
    std::fs::write(repo_path.join("modified"), "m").unwrap();
    insta::assert_snapshot!(test_env.run_jj_in(&repo_path, ["diff", "--name-only"]), @r"
    deleted
    modified
    [EOF]
    ");
    test_env
        .run_jj_in(&repo_path, ["commit", "-mfirst"])
        .success();
    std::fs::remove_file(repo_path.join("deleted")).unwrap();
    std::fs::write(repo_path.join("modified"), "mod").unwrap();
    std::fs::write(repo_path.join("added"), "add").unwrap();
    std::fs::create_dir(repo_path.join("sub")).unwrap();
    std::fs::write(repo_path.join("sub/added"), "sub/add").unwrap();
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["diff", "--name-only"]).normalize_backslash(), @r"
    added
    deleted
    modified
    sub/added
    [EOF]
    ");
}

#[test]
fn test_diff_bad_args() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let output = test_env.run_jj_in(&repo_path, ["diff", "-s", "--types"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    error: the argument '--summary' cannot be used with '--types'

    Usage: jj diff --summary [FILESETS]...

    For more information, try '--help'.
    [EOF]
    [exit status: 2]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--color-words", "--git"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    error: the argument '--color-words' cannot be used with '--git'

    Usage: jj diff --color-words [FILESETS]...

    For more information, try '--help'.
    [EOF]
    [exit status: 2]
    ");
}

#[test]
fn test_diff_relative_paths() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::create_dir_all(repo_path.join("dir1").join("subdir1")).unwrap();
    std::fs::create_dir(repo_path.join("dir2")).unwrap();
    std::fs::write(repo_path.join("file1"), "foo1\n").unwrap();
    std::fs::write(repo_path.join("dir1").join("file2"), "foo2\n").unwrap();
    std::fs::write(
        repo_path.join("dir1").join("subdir1").join("file3"),
        "foo3\n",
    )
    .unwrap();
    std::fs::write(repo_path.join("dir2").join("file4"), "foo4\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "bar1\n").unwrap();
    std::fs::write(repo_path.join("dir1").join("file2"), "bar2\n").unwrap();
    std::fs::write(
        repo_path.join("dir1").join("subdir1").join("file3"),
        "bar3\n",
    )
    .unwrap();
    std::fs::write(repo_path.join("dir2").join("file4"), "bar4\n").unwrap();

    let output = test_env.run_jj_in(&repo_path.join("dir1"), ["diff"]);
    #[cfg(unix)]
    insta::assert_snapshot!(output, @r"
    Modified regular file file2:
       1    1: foo2bar2
    Modified regular file subdir1/file3:
       1    1: foo3bar3
    Modified regular file ../dir2/file4:
       1    1: foo4bar4
    Modified regular file ../file1:
       1    1: foo1bar1
    [EOF]
    ");
    #[cfg(windows)]
    insta::assert_snapshot!(output, @r"
    Modified regular file file2:
       1    1: foo2bar2
    Modified regular file subdir1\file3:
       1    1: foo3bar3
    Modified regular file ..\dir2\file4:
       1    1: foo4bar4
    Modified regular file ..\file1:
       1    1: foo1bar1
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path.join("dir1"), ["diff", "-s"]);
    #[cfg(unix)]
    insta::assert_snapshot!(output, @r"
    M file2
    M subdir1/file3
    M ../dir2/file4
    M ../file1
    [EOF]
    ");
    #[cfg(windows)]
    insta::assert_snapshot!(output, @r"
    M file2
    M subdir1\file3
    M ..\dir2\file4
    M ..\file1
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path.join("dir1"), ["diff", "--types"]);
    #[cfg(unix)]
    insta::assert_snapshot!(output, @r"
    FF file2
    FF subdir1/file3
    FF ../dir2/file4
    FF ../file1
    [EOF]
    ");
    #[cfg(windows)]
    insta::assert_snapshot!(output, @r"
    FF file2
    FF subdir1\file3
    FF ..\dir2\file4
    FF ..\file1
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path.join("dir1"), ["diff", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/dir1/file2 b/dir1/file2
    index 54b060eee9..1fe912cdd8 100644
    --- a/dir1/file2
    +++ b/dir1/file2
    @@ -1,1 +1,1 @@
    -foo2
    +bar2
    diff --git a/dir1/subdir1/file3 b/dir1/subdir1/file3
    index c1ec6c6f12..f3c8b75ec6 100644
    --- a/dir1/subdir1/file3
    +++ b/dir1/subdir1/file3
    @@ -1,1 +1,1 @@
    -foo3
    +bar3
    diff --git a/dir2/file4 b/dir2/file4
    index a0016dbc4c..17375f7a12 100644
    --- a/dir2/file4
    +++ b/dir2/file4
    @@ -1,1 +1,1 @@
    -foo4
    +bar4
    diff --git a/file1 b/file1
    index 1715acd6a5..05c4fe6772 100644
    --- a/file1
    +++ b/file1
    @@ -1,1 +1,1 @@
    -foo1
    +bar1
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path.join("dir1"), ["diff", "--stat"]);
    #[cfg(unix)]
    insta::assert_snapshot!(output, @r"
    file2         | 2 +-
    subdir1/file3 | 2 +-
    ../dir2/file4 | 2 +-
    ../file1      | 2 +-
    4 files changed, 4 insertions(+), 4 deletions(-)
    [EOF]
    ");
    #[cfg(windows)]
    insta::assert_snapshot!(output, @r"
    file2         | 2 +-
    subdir1\file3 | 2 +-
    ..\dir2\file4 | 2 +-
    ..\file1      | 2 +-
    4 files changed, 4 insertions(+), 4 deletions(-)
    [EOF]
    ");
}

#[test]
fn test_diff_hunks() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    // Test added, removed, inserted, and modified lines. The modified line
    // contains unchanged words.
    std::fs::write(repo_path.join("file1"), "").unwrap();
    std::fs::write(repo_path.join("file2"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file3"), "foo\nbaz qux blah blah\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file2"), "").unwrap();
    std::fs::write(repo_path.join("file3"), "foo\nbar\nbaz quux blah blah\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
            1: foo
    Modified regular file file2:
       1     : foo
    Modified regular file file3:
       1    1: foo
            2: bar
       2    3: baz quxquux blah blah
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--color=debug"]);
    insta::assert_snapshot!(output, @r"
    [38;5;3m<<diff header::Modified regular file file1:>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::   1>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::foo>>[24m[39m
    [38;5;3m<<diff header::Modified regular file file2:>>[39m
    [38;5;1m<<diff removed line_number::   1>>[39m<<diff::     : >>[4m[38;5;1m<<diff removed token::foo>>[24m[39m
    [38;5;3m<<diff header::Modified regular file file3:>>[39m
    [38;5;1m<<diff removed line_number::   1>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   1>>[39m<<diff::: foo>>
    <<diff::     >>[38;5;2m<<diff added line_number::   2>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::bar>>[24m[39m
    [38;5;1m<<diff removed line_number::   2>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   3>>[39m<<diff::: baz >>[4m[38;5;1m<<diff removed token::qux>>[38;5;2m<<diff added token::quux>>[24m[39m<<diff:: blah blah>>
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index e69de29bb2..257cc5642c 100644
    --- a/file1
    +++ b/file1
    @@ -0,0 +1,1 @@
    +foo
    diff --git a/file2 b/file2
    index 257cc5642c..e69de29bb2 100644
    --- a/file2
    +++ b/file2
    @@ -1,1 +0,0 @@
    -foo
    diff --git a/file3 b/file3
    index 221a95a095..a543ef3892 100644
    --- a/file3
    +++ b/file3
    @@ -1,2 +1,3 @@
     foo
    -baz qux blah blah
    +bar
    +baz quux blah blah
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--color=debug"]);
    insta::assert_snapshot!(output, @r"
    [1m<<diff file_header::diff --git a/file1 b/file1>>[0m
    [1m<<diff file_header::index e69de29bb2..257cc5642c 100644>>[0m
    [1m<<diff file_header::--- a/file1>>[0m
    [1m<<diff file_header::+++ b/file1>>[0m
    [38;5;6m<<diff hunk_header::@@ -0,0 +1,1 @@>>[39m
    [38;5;2m<<diff added::+>>[4m<<diff added token::foo>>[24m[39m
    [1m<<diff file_header::diff --git a/file2 b/file2>>[0m
    [1m<<diff file_header::index 257cc5642c..e69de29bb2 100644>>[0m
    [1m<<diff file_header::--- a/file2>>[0m
    [1m<<diff file_header::+++ b/file2>>[0m
    [38;5;6m<<diff hunk_header::@@ -1,1 +0,0 @@>>[39m
    [38;5;1m<<diff removed::->>[4m<<diff removed token::foo>>[24m[39m
    [1m<<diff file_header::diff --git a/file3 b/file3>>[0m
    [1m<<diff file_header::index 221a95a095..a543ef3892 100644>>[0m
    [1m<<diff file_header::--- a/file3>>[0m
    [1m<<diff file_header::+++ b/file3>>[0m
    [38;5;6m<<diff hunk_header::@@ -1,2 +1,3 @@>>[39m
    <<diff context:: foo>>
    [38;5;1m<<diff removed::-baz >>[4m<<diff removed token::qux>>[24m<<diff removed:: blah blah>>[39m
    [38;5;2m<<diff added::+>>[4m<<diff added token::bar>>[24m[39m
    [38;5;2m<<diff added::+baz >>[4m<<diff added token::quux>>[24m<<diff added:: blah blah>>[39m
    [EOF]
    ");
}

#[test]
fn test_diff_color_words_inlining_threshold() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let render_diff = |max_alternation: i32, args: &[&str]| {
        let config = format!("diff.color-words.max-inline-alternation={max_alternation}");
        test_env.run_jj_with(|cmd| {
            cmd.current_dir(&repo_path)
                .args(["diff", "--config", &config])
                .args(args)
        })
    };

    let file1_path = "file1-single-line";
    let file2_path = "file2-multiple-lines-in-single-hunk";
    let file3_path = "file3-changes-across-lines";
    std::fs::write(
        repo_path.join(file1_path),
        indoc! {"
            == adds ==
            a b c
            == removes ==
            a b c d e f g
            == adds + removes ==
            a b c d e
            == adds + removes + adds ==
            a b c d e
            == adds + removes + adds + removes ==
            a b c d e f g
        "},
    )
    .unwrap();
    std::fs::write(
        repo_path.join(file2_path),
        indoc! {"
            == adds; removes; adds + removes ==
            a b c
            a b c d e f g
            a b c d e
            == adds + removes + adds; adds + removes + adds + removes ==
            a b c d e
            a b c d e f g
        "},
    )
    .unwrap();
    std::fs::write(
        repo_path.join(file3_path),
        indoc! {"
            == adds ==
            a b c
            == removes ==
            a b c d
            e f g
            == adds + removes ==
            a b c
            d e
            == adds + removes + adds ==
            a b c
            d e
            == adds + removes + adds + removes ==
            a b
            c d e f g
        "},
    )
    .unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(
        repo_path.join(file1_path),
        indoc! {"
            == adds ==
            a X b Y Z c
            == removes ==
            a c f
            == adds + removes ==
            a X b d
            == adds + removes + adds ==
            a X b d Y
            == adds + removes + adds + removes ==
            X a Y b d Z e
        "},
    )
    .unwrap();
    std::fs::write(
        repo_path.join(file2_path),
        indoc! {"
            == adds; removes; adds + removes ==
            a X b Y Z c
            a c f
            a X b d
            == adds + removes + adds; adds + removes + adds + removes ==
            a X b d Y
            X a Y b d Z e
        "},
    )
    .unwrap();
    std::fs::write(
        repo_path.join(file3_path),
        indoc! {"
            == adds ==
            a X b
            Y Z c
            == removes ==
            a c f
            == adds + removes ==
            a
            X b d
            == adds + removes + adds ==
            a X b d
            Y
            == adds + removes + adds + removes ==
            X a Y b d
            Z e
        "},
    )
    .unwrap();

    // default
    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2    2: a X b Y Z c
       3    3: == removes ==
       4    4: a b c d e f g
       5    5: == adds + removes ==
       6    6: a X b c d e
       7    7: == adds + removes + adds ==
       8    8: a X b c d eY
       9    9: == adds + removes + adds + removes ==
      10     : a b c d e f g
           10: X a Y b d Z e
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2    2: a X b Y Z c
       3    3: a b c d e f g
       4    4: a X b c d e
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6     : a b c d e
       7     : a b c d e f g
            6: a X b d Y
            7: X a Y b d Z e
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2    2: a X b
       2    3: Y Z c
       3    4: == removes ==
       4    5: a b c d
       5    5: e f g
       6    6: == adds + removes ==
       7    7: a
       7    8: X b c
       8    8: d e
       9    9: == adds + removes + adds ==
      10   10: a X b c
      11   10: d e
      11   11: Y
      12   12: == adds + removes + adds + removes ==
      13     : a b
      14     : c d e f g
           13: X a Y b d
           14: Z e
    [EOF]
    ");

    // -1: inline all
    insta::assert_snapshot!(render_diff(-1, &[]), @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2    2: a X b Y Z c
       3    3: == removes ==
       4    4: a b c d e f g
       5    5: == adds + removes ==
       6    6: a X b c d e
       7    7: == adds + removes + adds ==
       8    8: a X b c d eY
       9    9: == adds + removes + adds + removes ==
      10   10: X a Y b c d Z e f g
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2    2: a X b Y Z c
       3    3: a b c d e f g
       4    4: a X b c d e
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6    6: a X b c d eY
       7    7: X a Y b c d Z e f g
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2    2: a X b
       2    3: Y Z c
       3    4: == removes ==
       4    5: a b c d
       5    5: e f g
       6    6: == adds + removes ==
       7    7: a
       7    8: X b c
       8    8: d e
       9    9: == adds + removes + adds ==
      10   10: a X b c
      11   10: d e
      11   11: Y
      12   12: == adds + removes + adds + removes ==
      13   13: X a Y b
      14   13: c d
      14   14: Z e f g
    [EOF]
    ");

    // 0: no inlining
    insta::assert_snapshot!(render_diff(0, &[]), @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2     : a b c
            2: a X b Y Z c
       3    3: == removes ==
       4     : a b c d e f g
            4: a c f
       5    5: == adds + removes ==
       6     : a b c d e
            6: a X b d
       7    7: == adds + removes + adds ==
       8     : a b c d e
            8: a X b d Y
       9    9: == adds + removes + adds + removes ==
      10     : a b c d e f g
           10: X a Y b d Z e
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2     : a b c
       3     : a b c d e f g
       4     : a b c d e
            2: a X b Y Z c
            3: a c f
            4: a X b d
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6     : a b c d e
       7     : a b c d e f g
            6: a X b d Y
            7: X a Y b d Z e
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2     : a b c
            2: a X b
            3: Y Z c
       3    4: == removes ==
       4     : a b c d
       5     : e f g
            5: a c f
       6    6: == adds + removes ==
       7     : a b c
       8     : d e
            7: a
            8: X b d
       9    9: == adds + removes + adds ==
      10     : a b c
      11     : d e
           10: a X b d
           11: Y
      12   12: == adds + removes + adds + removes ==
      13     : a b
      14     : c d e f g
           13: X a Y b d
           14: Z e
    [EOF]
    ");

    // 1: inline adds-only or removes-only lines
    insta::assert_snapshot!(render_diff(1, &[]), @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2    2: a X b Y Z c
       3    3: == removes ==
       4    4: a b c d e f g
       5    5: == adds + removes ==
       6     : a b c d e
            6: a X b d
       7    7: == adds + removes + adds ==
       8     : a b c d e
            8: a X b d Y
       9    9: == adds + removes + adds + removes ==
      10     : a b c d e f g
           10: X a Y b d Z e
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2     : a b c
       3     : a b c d e f g
       4     : a b c d e
            2: a X b Y Z c
            3: a c f
            4: a X b d
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6     : a b c d e
       7     : a b c d e f g
            6: a X b d Y
            7: X a Y b d Z e
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2    2: a X b
       2    3: Y Z c
       3    4: == removes ==
       4    5: a b c d
       5    5: e f g
       6    6: == adds + removes ==
       7     : a b c
       8     : d e
            7: a
            8: X b d
       9    9: == adds + removes + adds ==
      10     : a b c
      11     : d e
           10: a X b d
           11: Y
      12   12: == adds + removes + adds + removes ==
      13     : a b
      14     : c d e f g
           13: X a Y b d
           14: Z e
    [EOF]
    ");

    // 2: inline up to adds + removes lines
    insta::assert_snapshot!(render_diff(2, &[]), @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2    2: a X b Y Z c
       3    3: == removes ==
       4    4: a b c d e f g
       5    5: == adds + removes ==
       6    6: a X b c d e
       7    7: == adds + removes + adds ==
       8     : a b c d e
            8: a X b d Y
       9    9: == adds + removes + adds + removes ==
      10     : a b c d e f g
           10: X a Y b d Z e
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2    2: a X b Y Z c
       3    3: a b c d e f g
       4    4: a X b c d e
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6     : a b c d e
       7     : a b c d e f g
            6: a X b d Y
            7: X a Y b d Z e
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2    2: a X b
       2    3: Y Z c
       3    4: == removes ==
       4    5: a b c d
       5    5: e f g
       6    6: == adds + removes ==
       7    7: a
       7    8: X b c
       8    8: d e
       9    9: == adds + removes + adds ==
      10     : a b c
      11     : d e
           10: a X b d
           11: Y
      12   12: == adds + removes + adds + removes ==
      13     : a b
      14     : c d e f g
           13: X a Y b d
           14: Z e
    [EOF]
    ");

    // 3: inline up to adds + removes + adds lines
    insta::assert_snapshot!(render_diff(3, &[]), @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2    2: a X b Y Z c
       3    3: == removes ==
       4    4: a b c d e f g
       5    5: == adds + removes ==
       6    6: a X b c d e
       7    7: == adds + removes + adds ==
       8    8: a X b c d eY
       9    9: == adds + removes + adds + removes ==
      10     : a b c d e f g
           10: X a Y b d Z e
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2    2: a X b Y Z c
       3    3: a b c d e f g
       4    4: a X b c d e
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6     : a b c d e
       7     : a b c d e f g
            6: a X b d Y
            7: X a Y b d Z e
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2    2: a X b
       2    3: Y Z c
       3    4: == removes ==
       4    5: a b c d
       5    5: e f g
       6    6: == adds + removes ==
       7    7: a
       7    8: X b c
       8    8: d e
       9    9: == adds + removes + adds ==
      10   10: a X b c
      11   10: d e
      11   11: Y
      12   12: == adds + removes + adds + removes ==
      13     : a b
      14     : c d e f g
           13: X a Y b d
           14: Z e
    [EOF]
    ");

    // 4: inline up to adds + removes + adds + removes lines
    insta::assert_snapshot!(render_diff(4, &[]), @r"
    Modified regular file file1-single-line:
       1    1: == adds ==
       2    2: a X b Y Z c
       3    3: == removes ==
       4    4: a b c d e f g
       5    5: == adds + removes ==
       6    6: a X b c d e
       7    7: == adds + removes + adds ==
       8    8: a X b c d eY
       9    9: == adds + removes + adds + removes ==
      10   10: X a Y b c d Z e f g
    Modified regular file file2-multiple-lines-in-single-hunk:
       1    1: == adds; removes; adds + removes ==
       2    2: a X b Y Z c
       3    3: a b c d e f g
       4    4: a X b c d e
       5    5: == adds + removes + adds; adds + removes + adds + removes ==
       6    6: a X b c d eY
       7    7: X a Y b c d Z e f g
    Modified regular file file3-changes-across-lines:
       1    1: == adds ==
       2    2: a X b
       2    3: Y Z c
       3    4: == removes ==
       4    5: a b c d
       5    5: e f g
       6    6: == adds + removes ==
       7    7: a
       7    8: X b c
       8    8: d e
       9    9: == adds + removes + adds ==
      10   10: a X b c
      11   10: d e
      11   11: Y
      12   12: == adds + removes + adds + removes ==
      13   13: X a Y b
      14   13: c d
      14   14: Z e f g
    [EOF]
    ");

    // context words in added/removed lines should be labeled as such
    insta::assert_snapshot!(render_diff(2, &["--color=always"]), @r"
    [38;5;3mModified regular file file1-single-line:[39m
    [38;5;1m   1[39m [38;5;2m   1[39m: == adds ==
    [38;5;1m   2[39m [38;5;2m   2[39m: a [4m[38;5;2mX [24m[39mb [4m[38;5;2mY Z [24m[39mc
    [38;5;1m   3[39m [38;5;2m   3[39m: == removes ==
    [38;5;1m   4[39m [38;5;2m   4[39m: a [4m[38;5;1mb [24m[39mc [4m[38;5;1md e [24m[39mf[4m[38;5;1m g[24m[39m
    [38;5;1m   5[39m [38;5;2m   5[39m: == adds + removes ==
    [38;5;1m   6[39m [38;5;2m   6[39m: a [4m[38;5;2mX [24m[39mb [4m[38;5;1mc [24m[39md[4m[38;5;1m e[24m[39m
    [38;5;1m   7[39m [38;5;2m   7[39m: == adds + removes + adds ==
    [38;5;1m   8[39m     : [38;5;1ma b [4mc [24md [4me[24m[39m
         [38;5;2m   8[39m: [38;5;2ma [4mX [24mb d [4mY[24m[39m
    [38;5;1m   9[39m [38;5;2m   9[39m: == adds + removes + adds + removes ==
    [38;5;1m  10[39m     : [38;5;1ma b [4mc [24md e[4m f g[24m[39m
         [38;5;2m  10[39m: [4m[38;5;2mX [24ma [4mY [24mb d [4mZ [24me[39m
    [38;5;3mModified regular file file2-multiple-lines-in-single-hunk:[39m
    [38;5;1m   1[39m [38;5;2m   1[39m: == adds; removes; adds + removes ==
    [38;5;1m   2[39m [38;5;2m   2[39m: a [4m[38;5;2mX [24m[39mb [4m[38;5;2mY Z [24m[39mc
    [38;5;1m   3[39m [38;5;2m   3[39m: a [4m[38;5;1mb [24m[39mc [4m[38;5;1md e [24m[39mf[4m[38;5;1m g[24m[39m
    [38;5;1m   4[39m [38;5;2m   4[39m: a [4m[38;5;2mX [24m[39mb [4m[38;5;1mc [24m[39md[4m[38;5;1m e[24m[39m
    [38;5;1m   5[39m [38;5;2m   5[39m: == adds + removes + adds; adds + removes + adds + removes ==
    [38;5;1m   6[39m     : [38;5;1ma b [4mc [24md [4me[24m[39m
    [38;5;1m   7[39m     : [38;5;1ma b [4mc [24md e[4m f g[24m[39m
         [38;5;2m   6[39m: [38;5;2ma [4mX [24mb d [4mY[24m[39m
         [38;5;2m   7[39m: [4m[38;5;2mX [24ma [4mY [24mb d [4mZ [24me[39m
    [38;5;3mModified regular file file3-changes-across-lines:[39m
    [38;5;1m   1[39m [38;5;2m   1[39m: == adds ==
    [38;5;1m   2[39m [38;5;2m   2[39m: a [4m[38;5;2mX [24m[39mb[4m[38;5;2m[24m[39m
    [38;5;1m   2[39m [38;5;2m   3[39m: [4m[38;5;2mY Z[24m[39m c
    [38;5;1m   3[39m [38;5;2m   4[39m: == removes ==
    [38;5;1m   4[39m [38;5;2m   5[39m: a [4m[38;5;1mb [24m[39mc [4m[38;5;1md[24m[39m
    [38;5;1m   5[39m [38;5;2m   5[39m: [4m[38;5;1me [24m[39mf[4m[38;5;1m g[24m[39m
    [38;5;1m   6[39m [38;5;2m   6[39m: == adds + removes ==
    [38;5;1m   7[39m [38;5;2m   7[39m: a[4m[38;5;2m[24m[39m
    [38;5;1m   7[39m [38;5;2m   8[39m: [4m[38;5;2mX[24m[39m b [4m[38;5;1mc[24m[39m
    [38;5;1m   8[39m [38;5;2m   8[39m: d[4m[38;5;1m e[24m[39m
    [38;5;1m   9[39m [38;5;2m   9[39m: == adds + removes + adds ==
    [38;5;1m  10[39m     : [38;5;1ma b [4mc[24m[39m
    [38;5;1m  11[39m     : [38;5;1md[4m e[24m[39m
         [38;5;2m  10[39m: [38;5;2ma [4mX [24mb d[4m[24m[39m
         [38;5;2m  11[39m: [4m[38;5;2mY[24m[39m
    [38;5;1m  12[39m [38;5;2m  12[39m: == adds + removes + adds + removes ==
    [38;5;1m  13[39m     : [38;5;1ma b[4m[24m[39m
    [38;5;1m  14[39m     : [4m[38;5;1mc[24m d e[4m f g[24m[39m
         [38;5;2m  13[39m: [4m[38;5;2mX [24ma [4mY [24mb d[4m[24m[39m
         [38;5;2m  14[39m: [4m[38;5;2mZ[24m e[39m
    [EOF]
    ");
    insta::assert_snapshot!(render_diff(2, &["--color=debug"]), @r"
    [38;5;3m<<diff header::Modified regular file file1-single-line:>>[39m
    [38;5;1m<<diff removed line_number::   1>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   1>>[39m<<diff::: == adds ==>>
    [38;5;1m<<diff removed line_number::   2>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   2>>[39m<<diff::: a >>[4m[38;5;2m<<diff added token::X >>[24m[39m<<diff::b >>[4m[38;5;2m<<diff added token::Y Z >>[24m[39m<<diff::c>>
    [38;5;1m<<diff removed line_number::   3>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   3>>[39m<<diff::: == removes ==>>
    [38;5;1m<<diff removed line_number::   4>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   4>>[39m<<diff::: a >>[4m[38;5;1m<<diff removed token::b >>[24m[39m<<diff::c >>[4m[38;5;1m<<diff removed token::d e >>[24m[39m<<diff::f>>[4m[38;5;1m<<diff removed token:: g>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   5>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   5>>[39m<<diff::: == adds + removes ==>>
    [38;5;1m<<diff removed line_number::   6>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   6>>[39m<<diff::: a >>[4m[38;5;2m<<diff added token::X >>[24m[39m<<diff::b >>[4m[38;5;1m<<diff removed token::c >>[24m[39m<<diff::d>>[4m[38;5;1m<<diff removed token:: e>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   7>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   7>>[39m<<diff::: == adds + removes + adds ==>>
    [38;5;1m<<diff removed line_number::   8>>[39m<<diff::     : >>[38;5;1m<<diff removed::a b >>[4m<<diff removed token::c >>[24m<<diff removed::d >>[4m<<diff removed token::e>>[24m<<diff removed::>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::   8>>[39m<<diff::: >>[38;5;2m<<diff added::a >>[4m<<diff added token::X >>[24m<<diff added::b d >>[4m<<diff added token::Y>>[24m<<diff added::>>[39m
    [38;5;1m<<diff removed line_number::   9>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   9>>[39m<<diff::: == adds + removes + adds + removes ==>>
    [38;5;1m<<diff removed line_number::  10>>[39m<<diff::     : >>[38;5;1m<<diff removed::a b >>[4m<<diff removed token::c >>[24m<<diff removed::d e>>[4m<<diff removed token:: f g>>[24m<<diff removed::>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::  10>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::X >>[24m<<diff added::a >>[4m<<diff added token::Y >>[24m<<diff added::b d >>[4m<<diff added token::Z >>[24m<<diff added::e>>[39m
    [38;5;3m<<diff header::Modified regular file file2-multiple-lines-in-single-hunk:>>[39m
    [38;5;1m<<diff removed line_number::   1>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   1>>[39m<<diff::: == adds; removes; adds + removes ==>>
    [38;5;1m<<diff removed line_number::   2>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   2>>[39m<<diff::: a >>[4m[38;5;2m<<diff added token::X >>[24m[39m<<diff::b >>[4m[38;5;2m<<diff added token::Y Z >>[24m[39m<<diff::c>>
    [38;5;1m<<diff removed line_number::   3>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   3>>[39m<<diff::: a >>[4m[38;5;1m<<diff removed token::b >>[24m[39m<<diff::c >>[4m[38;5;1m<<diff removed token::d e >>[24m[39m<<diff::f>>[4m[38;5;1m<<diff removed token:: g>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   4>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   4>>[39m<<diff::: a >>[4m[38;5;2m<<diff added token::X >>[24m[39m<<diff::b >>[4m[38;5;1m<<diff removed token::c >>[24m[39m<<diff::d>>[4m[38;5;1m<<diff removed token:: e>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   5>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   5>>[39m<<diff::: == adds + removes + adds; adds + removes + adds + removes ==>>
    [38;5;1m<<diff removed line_number::   6>>[39m<<diff::     : >>[38;5;1m<<diff removed::a b >>[4m<<diff removed token::c >>[24m<<diff removed::d >>[4m<<diff removed token::e>>[24m<<diff removed::>>[39m
    [38;5;1m<<diff removed line_number::   7>>[39m<<diff::     : >>[38;5;1m<<diff removed::a b >>[4m<<diff removed token::c >>[24m<<diff removed::d e>>[4m<<diff removed token:: f g>>[24m<<diff removed::>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::   6>>[39m<<diff::: >>[38;5;2m<<diff added::a >>[4m<<diff added token::X >>[24m<<diff added::b d >>[4m<<diff added token::Y>>[24m<<diff added::>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::   7>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::X >>[24m<<diff added::a >>[4m<<diff added token::Y >>[24m<<diff added::b d >>[4m<<diff added token::Z >>[24m<<diff added::e>>[39m
    [38;5;3m<<diff header::Modified regular file file3-changes-across-lines:>>[39m
    [38;5;1m<<diff removed line_number::   1>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   1>>[39m<<diff::: == adds ==>>
    [38;5;1m<<diff removed line_number::   2>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   2>>[39m<<diff::: a >>[4m[38;5;2m<<diff added token::X >>[24m[39m<<diff::b>>[4m[38;5;2m<<diff added token::>>[24m[39m
    [38;5;1m<<diff removed line_number::   2>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   3>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::Y Z>>[24m[39m<<diff:: c>>
    [38;5;1m<<diff removed line_number::   3>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   4>>[39m<<diff::: == removes ==>>
    [38;5;1m<<diff removed line_number::   4>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   5>>[39m<<diff::: a >>[4m[38;5;1m<<diff removed token::b >>[24m[39m<<diff::c >>[4m[38;5;1m<<diff removed token::d>>[24m[39m
    [38;5;1m<<diff removed line_number::   5>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   5>>[39m<<diff::: >>[4m[38;5;1m<<diff removed token::e >>[24m[39m<<diff::f>>[4m[38;5;1m<<diff removed token:: g>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   6>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   6>>[39m<<diff::: == adds + removes ==>>
    [38;5;1m<<diff removed line_number::   7>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   7>>[39m<<diff::: a>>[4m[38;5;2m<<diff added token::>>[24m[39m
    [38;5;1m<<diff removed line_number::   7>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   8>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::X>>[24m[39m<<diff:: b >>[4m[38;5;1m<<diff removed token::c>>[24m[39m
    [38;5;1m<<diff removed line_number::   8>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   8>>[39m<<diff::: d>>[4m[38;5;1m<<diff removed token:: e>>[24m[39m<<diff::>>
    [38;5;1m<<diff removed line_number::   9>>[39m<<diff:: >>[38;5;2m<<diff added line_number::   9>>[39m<<diff::: == adds + removes + adds ==>>
    [38;5;1m<<diff removed line_number::  10>>[39m<<diff::     : >>[38;5;1m<<diff removed::a b >>[4m<<diff removed token::c>>[24m[39m
    [38;5;1m<<diff removed line_number::  11>>[39m<<diff::     : >>[38;5;1m<<diff removed::d>>[4m<<diff removed token:: e>>[24m<<diff removed::>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::  10>>[39m<<diff::: >>[38;5;2m<<diff added::a >>[4m<<diff added token::X >>[24m<<diff added::b d>>[4m<<diff added token::>>[24m[39m
    <<diff::     >>[38;5;2m<<diff added line_number::  11>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::Y>>[24m<<diff added::>>[39m
    [38;5;1m<<diff removed line_number::  12>>[39m<<diff:: >>[38;5;2m<<diff added line_number::  12>>[39m<<diff::: == adds + removes + adds + removes ==>>
    [38;5;1m<<diff removed line_number::  13>>[39m<<diff::     : >>[38;5;1m<<diff removed::a b>>[4m<<diff removed token::>>[24m[39m
    [38;5;1m<<diff removed line_number::  14>>[39m<<diff::     : >>[4m[38;5;1m<<diff removed token::c>>[24m<<diff removed:: d e>>[4m<<diff removed token:: f g>>[24m<<diff removed::>>[39m
    <<diff::     >>[38;5;2m<<diff added line_number::  13>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::X >>[24m<<diff added::a >>[4m<<diff added token::Y >>[24m<<diff added::b d>>[4m<<diff added token::>>[24m[39m
    <<diff::     >>[38;5;2m<<diff added line_number::  14>>[39m<<diff::: >>[4m[38;5;2m<<diff added token::Z>>[24m<<diff added:: e>>[39m
    [EOF]
    ");
}

#[test]
fn test_diff_missing_newline() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo").unwrap();
    std::fs::write(repo_path.join("file2"), "foo\nbar").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "foo\nbar").unwrap();
    std::fs::write(repo_path.join("file2"), "foo").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
       1    1: foo
            2: bar
    Modified regular file file2:
       1    1: foo
       2     : bar
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index 1910281566..a907ec3f43 100644
    --- a/file1
    +++ b/file1
    @@ -1,1 +1,2 @@
    -foo
    \ No newline at end of file
    +foo
    +bar
    \ No newline at end of file
    diff --git a/file2 b/file2
    index a907ec3f43..1910281566 100644
    --- a/file2
    +++ b/file2
    @@ -1,2 +1,1 @@
    -foo
    -bar
    \ No newline at end of file
    +foo
    \ No newline at end of file
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    file1 | 3 ++-
    file2 | 3 +--
    2 files changed, 3 insertions(+), 3 deletions(-)
    [EOF]
    ");
}

#[test]
fn test_color_words_diff_missing_newline() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Empty"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd\ne\nf\ng\nh\ni").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Add no newline"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\ne\nf\ng\nh\ni").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Modify first line"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\nE\nf\ng\nh\ni").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Modify middle line"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\nE\nf\ng\nh\nI").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Modify last line"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\nE\nf\ng\nh\nI\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Append newline"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\nE\nf\ng\nh\nI").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Remove newline"])
        .success();
    std::fs::write(repo_path.join("file1"), "").unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "=== Empty"])
        .success();

    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-Tdescription",
            "-pr::@-",
            "--no-graph",
            "--reversed",
        ],
    );
    insta::assert_snapshot!(output, @r"
    === Empty
    Added regular file file1:
        (empty)
    === Add no newline
    Modified regular file file1:
            1: a
            2: b
            3: c
            4: d
            5: e
            6: f
            7: g
            8: h
            9: i
    === Modify first line
    Modified regular file file1:
       1    1: aA
       2    2: b
       3    3: c
       4    4: d
        ...
    === Modify middle line
    Modified regular file file1:
       1    1: A
       2    2: b
       3    3: c
       4    4: d
       5    5: eE
       6    6: f
       7    7: g
       8    8: h
       9    9: i
    === Modify last line
    Modified regular file file1:
        ...
       6    6: f
       7    7: g
       8    8: h
       9    9: iI
    === Append newline
    Modified regular file file1:
        ...
       6    6: f
       7    7: g
       8    8: h
       9    9: I
    === Remove newline
    Modified regular file file1:
        ...
       6    6: f
       7    7: g
       8    8: h
       9    9: I
    === Empty
    Modified regular file file1:
       1     : A
       2     : b
       3     : c
       4     : d
       5     : E
       6     : f
       7     : g
       8     : h
       9     : I
    [EOF]
    ");

    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "--config=diff.color-words.max-inline-alternation=0",
            "-Tdescription",
            "-pr::@-",
            "--no-graph",
            "--reversed",
        ],
    );
    insta::assert_snapshot!(output, @r"
    === Empty
    Added regular file file1:
        (empty)
    === Add no newline
    Modified regular file file1:
            1: a
            2: b
            3: c
            4: d
            5: e
            6: f
            7: g
            8: h
            9: i
    === Modify first line
    Modified regular file file1:
       1     : a
            1: A
       2    2: b
       3    3: c
       4    4: d
        ...
    === Modify middle line
    Modified regular file file1:
       1    1: A
       2    2: b
       3    3: c
       4    4: d
       5     : e
            5: E
       6    6: f
       7    7: g
       8    8: h
       9    9: i
    === Modify last line
    Modified regular file file1:
        ...
       6    6: f
       7    7: g
       8    8: h
       9     : i
            9: I
    === Append newline
    Modified regular file file1:
        ...
       6    6: f
       7    7: g
       8    8: h
       9     : I
            9: I
    === Remove newline
    Modified regular file file1:
        ...
       6    6: f
       7    7: g
       8    8: h
       9     : I
            9: I
    === Empty
    Modified regular file file1:
       1     : A
       2     : b
       3     : c
       4     : d
       5     : E
       6     : f
       7     : g
       8     : h
       9     : I
    [EOF]
    ");
}

#[test]
fn test_diff_ignore_whitespace() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(
        repo_path.join("file1"),
        indoc! {"
            foo {
                bar;
            }
            baz {}
        "},
    )
    .unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "-mindent + whitespace insertion"])
        .success();
    std::fs::write(
        repo_path.join("file1"),
        indoc! {"
            {
                foo {
                    bar;
                }
            }
            baz {  }
        "},
    )
    .unwrap();
    test_env.run_jj_in(&repo_path, ["status"]).success();

    // Git diff as reference output
    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--ignore-all-space"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index f532aa68ad..033c4a6168 100644
    --- a/file1
    +++ b/file1
    @@ -1,4 +1,6 @@
    +{
         foo {
             bar;
         }
    +}
     baz {  }
    [EOF]
    ");
    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--ignore-space-change"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index f532aa68ad..033c4a6168 100644
    --- a/file1
    +++ b/file1
    @@ -1,4 +1,6 @@
    -foo {
    +{
    +    foo {
             bar;
    +    }
     }
    -baz {}
    +baz {  }
    [EOF]
    ");

    // Diff-stat should respects the whitespace options
    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat", "--ignore-all-space"]);
    insta::assert_snapshot!(output, @r"
    file1 | 2 ++
    1 file changed, 2 insertions(+), 0 deletions(-)
    [EOF]
    ");
    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat", "--ignore-space-change"]);
    insta::assert_snapshot!(output, @r"
    file1 | 6 ++++--
    1 file changed, 4 insertions(+), 2 deletions(-)
    [EOF]
    ");

    // Word-level changes are still highlighted
    let output = test_env.run_jj_in(&repo_path, ["diff", "--color=always", "--ignore-all-space"]);
    insta::assert_snapshot!(output, @r"
    [38;5;3mModified regular file file1:[39m
         [38;5;2m   1[39m: [4m[38;5;2m{[24m[39m
    [38;5;1m   1[39m [38;5;2m   2[39m: [4m[38;5;2m    [24m[39mfoo {
    [38;5;1m   2[39m [38;5;2m   3[39m:     [4m[38;5;2m    [24m[39mbar;
    [38;5;1m   3[39m [38;5;2m   4[39m: [4m[38;5;2m    [24m[39m}
         [38;5;2m   5[39m: [4m[38;5;2m}[24m[39m
    [38;5;1m   4[39m [38;5;2m   6[39m: baz {[4m[38;5;2m  [24m[39m}
    [EOF]
    ");
    let output = test_env.run_jj_in(
        &repo_path,
        ["diff", "--color=always", "--ignore-space-change"],
    );
    insta::assert_snapshot!(output, @r"
    [38;5;3mModified regular file file1:[39m
         [38;5;2m   1[39m: [4m[38;5;2m{[24m[39m
    [38;5;1m   1[39m [38;5;2m   2[39m: [4m[38;5;2m    [24m[39mfoo {
    [38;5;1m   2[39m [38;5;2m   3[39m:     [4m[38;5;2m    [24m[39mbar;
         [38;5;2m   4[39m: [4m[38;5;2m    }[24m[39m
    [38;5;1m   3[39m [38;5;2m   5[39m: }
    [38;5;1m   4[39m [38;5;2m   6[39m: baz {[4m[38;5;2m  [24m[39m}
    [EOF]
    ");
}

#[test]
fn test_diff_skipped_context() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd\ne\nf\ng\nh\ni\nj").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "=== Left side of diffs"])
        .success();

    test_env
        .run_jj_in(&repo_path, ["new", "@", "-m", "=== Must skip 2 lines"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\ne\nf\ng\nh\ni\nJ").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== Don't skip 1 line"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nd\ne\nf\ng\nh\nI\nj").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== No gap to skip"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nB\nc\nd\ne\nf\ng\nh\nI\nj").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== No gap to skip"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nC\nd\ne\nf\ng\nh\nI\nj").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== 1 line at start"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd\nE\nf\ng\nh\ni\nj").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== 1 line at end"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd\ne\nF\ng\nh\ni\nj").unwrap();

    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-Tdescription", "-p", "--no-graph", "--reversed"],
    );
    insta::assert_snapshot!(output, @r"
    === Left side of diffs
    Added regular file file1:
            1: a
            2: b
            3: c
            4: d
            5: e
            6: f
            7: g
            8: h
            9: i
           10: j
    === Must skip 2 lines
    Modified regular file file1:
       1    1: aA
       2    2: b
       3    3: c
       4    4: d
        ...
       7    7: g
       8    8: h
       9    9: i
      10   10: jJ
    === Don't skip 1 line
    Modified regular file file1:
       1    1: aA
       2    2: b
       3    3: c
       4    4: d
       5    5: e
       6    6: f
       7    7: g
       8    8: h
       9    9: iI
      10   10: j
    === No gap to skip
    Modified regular file file1:
       1    1: a
       2    2: bB
       3    3: c
       4    4: d
       5    5: e
       6    6: f
       7    7: g
       8    8: h
       9    9: iI
      10   10: j
    === No gap to skip
    Modified regular file file1:
       1    1: a
       2    2: b
       3    3: cC
       4    4: d
       5    5: e
       6    6: f
       7    7: g
       8    8: h
       9    9: iI
      10   10: j
    === 1 line at start
    Modified regular file file1:
       1    1: a
       2    2: b
       3    3: c
       4    4: d
       5    5: eE
       6    6: f
       7    7: g
       8    8: h
        ...
    === 1 line at end
    Modified regular file file1:
        ...
       3    3: c
       4    4: d
       5    5: e
       6    6: fF
       7    7: g
       8    8: h
       9    9: i
      10   10: j
    [EOF]
    ");
}

#[test]
fn test_diff_skipped_context_from_settings_color_words() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env.add_config(
        r#"
[diff.color-words]
context = 0
        "#,
    );

    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd\ne").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "=== First commit"])
        .success();

    test_env
        .run_jj_in(&repo_path, ["new", "@", "-m", "=== Must show 0 context"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nC\nd\ne").unwrap();

    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-Tdescription", "-p", "--no-graph", "--reversed"],
    );
    insta::assert_snapshot!(output, @r"
    === First commit
    Added regular file file1:
            1: a
            2: b
            3: c
            4: d
            5: e
    === Must show 0 context
    Modified regular file file1:
        ...
       3    3: cC
        ...
    [EOF]
    ");
}

#[test]
fn test_diff_skipped_context_from_settings_git() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env.add_config(
        r#"
[diff.git]
context = 0
        "#,
    );

    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd\ne").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "=== First commit"])
        .success();

    test_env
        .run_jj_in(&repo_path, ["new", "@", "-m", "=== Must show 0 context"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nC\nd\ne").unwrap();

    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-Tdescription",
            "-p",
            "--git",
            "--no-graph",
            "--reversed",
        ],
    );
    insta::assert_snapshot!(output, @r"
    === First commit
    diff --git a/file1 b/file1
    new file mode 100644
    index 0000000000..0fec236860
    --- /dev/null
    +++ b/file1
    @@ -0,0 +1,5 @@
    +a
    +b
    +c
    +d
    +e
    \ No newline at end of file
    === Must show 0 context
    diff --git a/file1 b/file1
    index 0fec236860..b7615dae52 100644
    --- a/file1
    +++ b/file1
    @@ -3,1 +3,1 @@
    -c
    +C
    [EOF]
    ");
}

#[test]
fn test_diff_skipped_context_nondefault() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "a\nb\nc\nd").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "=== Left side of diffs"])
        .success();

    test_env
        .run_jj_in(&repo_path, ["new", "@", "-m", "=== Must skip 2 lines"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nc\nD").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== Don't skip 1 line"])
        .success();
    std::fs::write(repo_path.join("file1"), "A\nb\nC\nd").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== No gap to skip"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nB\nC\nd").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== 1 line at start"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nB\nc\nd").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "@-", "-m", "=== 1 line at end"])
        .success();
    std::fs::write(repo_path.join("file1"), "a\nb\nC\nd").unwrap();

    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-Tdescription",
            "-p",
            "--no-graph",
            "--reversed",
            "--context=0",
        ],
    );
    insta::assert_snapshot!(output, @r"
    === Left side of diffs
    Added regular file file1:
            1: a
            2: b
            3: c
            4: d
    === Must skip 2 lines
    Modified regular file file1:
       1    1: aA
        ...
       4    4: dD
    === Don't skip 1 line
    Modified regular file file1:
       1    1: aA
       2    2: b
       3    3: cC
       4    4: d
    === No gap to skip
    Modified regular file file1:
       1    1: a
       2    2: bB
       3    3: cC
       4    4: d
    === 1 line at start
    Modified regular file file1:
       1    1: a
       2    2: bB
        ...
    === 1 line at end
    Modified regular file file1:
        ...
       3    3: cC
       4    4: d
    [EOF]
    ");
}

#[test]
fn test_diff_leading_trailing_context() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    // N=5 context lines at start/end of the file
    std::fs::write(
        repo_path.join("file1"),
        "1\n2\n3\n4\n5\nL\n6\n7\n8\n9\n10\n11\n",
    )
    .unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(
        repo_path.join("file1"),
        "1\n2\n3\n4\n5\n6\nR\n7\n8\n9\n10\n11\n",
    )
    .unwrap();

    // N=5 <= num_context_lines + 1: No room to skip.
    let output = test_env.run_jj_in(&repo_path, ["diff", "--context=4"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
       1    1: 1
       2    2: 2
       3    3: 3
       4    4: 4
       5    5: 5
       6     : L
       7    6: 6
            7: R
       8    8: 7
       9    9: 8
      10   10: 9
      11   11: 10
      12   12: 11
    [EOF]
    ");

    // N=5 <= 2 * num_context_lines + 1: The last hunk wouldn't be split if
    // trailing diff existed.
    let output = test_env.run_jj_in(&repo_path, ["diff", "--context=3"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
        ...
       3    3: 3
       4    4: 4
       5    5: 5
       6     : L
       7    6: 6
            7: R
       8    8: 7
       9    9: 8
      10   10: 9
        ...
    [EOF]
    ");

    // N=5 > 2 * num_context_lines + 1: The last hunk should be split no matter
    // if trailing diff existed.
    let output = test_env.run_jj_in(&repo_path, ["diff", "--context=1"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
        ...
       5    5: 5
       6     : L
       7    6: 6
            7: R
       8    8: 7
        ...
    [EOF]
    ");

    // N=5 <= num_context_lines: No room to skip.
    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--context=5"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index 1bf57dee4a..69b3e1865c 100644
    --- a/file1
    +++ b/file1
    @@ -1,12 +1,12 @@
     1
     2
     3
     4
     5
    -L
     6
    +R
     7
     8
     9
     10
     11
    [EOF]
    ");

    // N=5 <= 2 * num_context_lines: The last hunk wouldn't be split if
    // trailing diff existed.
    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--context=3"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index 1bf57dee4a..69b3e1865c 100644
    --- a/file1
    +++ b/file1
    @@ -3,8 +3,8 @@
     3
     4
     5
    -L
     6
    +R
     7
     8
     9
    [EOF]
    ");

    // N=5 > 2 * num_context_lines: The last hunk should be split no matter
    // if trailing diff existed.
    let output = test_env.run_jj_in(&repo_path, ["diff", "--git", "--context=2"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1 b/file1
    index 1bf57dee4a..69b3e1865c 100644
    --- a/file1
    +++ b/file1
    @@ -4,6 +4,6 @@
     4
     5
    -L
     6
    +R
     7
     8
    [EOF]
    ");
}

#[test]
fn test_diff_external_tool() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file2"), "foo\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1")).unwrap();
    std::fs::write(repo_path.join("file2"), "foo\nbar\n").unwrap();
    std::fs::write(repo_path.join("file3"), "foo\n").unwrap();

    let edit_script = test_env.set_up_fake_diff_editor();

    // nonzero exit codes should print a warning
    std::fs::write(&edit_script, "fail").unwrap();
    let output = test_env.run_jj_in(
        &repo_path,
        ["diff", "--config=ui.diff.tool=fake-diff-editor"],
    );
    let mut insta_settings = insta::Settings::clone_current();
    insta_settings.add_filter("exit (status|code)", "<exit status>");
    insta_settings.bind(|| {
        insta::assert_snapshot!(output, @r"
        ------- stderr -------
        Warning: Tool exited with <exit status>: 1 (run with --debug to see the exact invocation)
        [EOF]
        ");
    });

    // nonzero exit codes should not print a warning if it's an expected exit code
    std::fs::write(&edit_script, "fail").unwrap();
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "diff",
            "--tool",
            "fake-diff-editor",
            "--config=merge-tools.fake-diff-editor.diff-expected-exit-codes=[1]",
        ],
    );
    insta::assert_snapshot!(output, @"");

    std::fs::write(
        &edit_script,
        "print-files-before\0print --\0print-files-after",
    )
    .unwrap();

    // diff without file patterns
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["diff", "--tool=fake-diff-editor"]), @r"
    file1
    file2
    --
    file2
    file3
    [EOF]
    ");

    // diff with file patterns
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["diff", "--tool=fake-diff-editor", "file1"]), @r"
    file1
    --
    [EOF]
    ");

    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-p", "--tool=fake-diff-editor"]), @r"
    @  rlvkpnrz test.user@example.com 2001-02-03 08:05:09 39d9055d
    │  (no description set)
    │  file1
    │  file2
    │  --
    │  file2
    │  file3
    ○  qpvuntsm test.user@example.com 2001-02-03 08:05:08 0ad4ef22
    │  (no description set)
    │  --
    │  file1
    │  file2
    ◆  zzzzzzzz root() 00000000
       --
    [EOF]
    ");

    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["show", "--tool=fake-diff-editor"]), @r"
    Commit ID: 39d9055d70873099fd924b9af218289d5663eac8
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    file1
    file2
    --
    file2
    file3
    [EOF]
    ");

    // Enabled by default, looks up the merge-tools table
    let config = "--config=ui.diff.tool=fake-diff-editor";
    insta::assert_snapshot!(test_env.run_jj_in(&repo_path, ["diff", config]), @r"
    file1
    file2
    --
    file2
    file3
    [EOF]
    ");

    // Inlined command arguments
    let command_toml = to_toml_value(fake_diff_editor_path());
    let config = format!("--config=ui.diff.tool=[{command_toml}, '$right', '$left']");
    insta::assert_snapshot!(test_env.run_jj_in(&repo_path, ["diff", &config]), @r"
    file2
    file3
    --
    file1
    file2
    [EOF]
    ");

    // Output of external diff tool shouldn't be escaped
    std::fs::write(&edit_script, "print \x1b[1;31mred").unwrap();
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["diff", "--color=always", "--tool=fake-diff-editor"]),
        @r"
    [1;31mred
    [EOF]
    ");

    // Non-zero exit code isn't an error
    std::fs::write(&edit_script, "print diff\0fail").unwrap();
    let output = test_env.run_jj_in(&repo_path, ["show", "--tool=fake-diff-editor"]);
    insta::assert_snapshot!(output.normalize_stderr_exit_status(), @r"
    Commit ID: 39d9055d70873099fd924b9af218289d5663eac8
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    diff
    [EOF]
    ------- stderr -------
    Warning: Tool exited with exit status: 1 (run with --debug to see the exact invocation)
    [EOF]
    ");

    // --tool=:builtin shouldn't be ignored
    let output = test_env.run_jj_in(&repo_path, ["diff", "--tool=:builtin"]);
    insta::assert_snapshot!(output.strip_stderr_last_line(), @r"
    ------- stderr -------
    Error: Failed to generate diff
    Caused by:
    1: Error executing ':builtin' (run with --debug to see the exact invocation)
    [EOF]
    [exit status: 1]
    ");
}

#[test]
fn test_diff_external_file_by_file_tool() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "file1\n").unwrap();
    std::fs::write(repo_path.join("file2"), "file2\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1")).unwrap();
    std::fs::write(repo_path.join("file2"), "file2\nfile2\n").unwrap();
    std::fs::write(repo_path.join("file3"), "file3\n").unwrap();
    std::fs::write(repo_path.join("file4"), "file1\n").unwrap();

    let edit_script = test_env.set_up_fake_diff_editor();
    std::fs::write(
        edit_script,
        "print ==\0print-files-before\0print --\0print-files-after",
    )
    .unwrap();

    // Enabled by default, looks up the merge-tools table
    let configs: &[_] = &[
        "--config=ui.diff.tool=fake-diff-editor",
        "--config=merge-tools.fake-diff-editor.diff-invocation-mode=file-by-file",
    ];

    // diff without file patterns
    insta::assert_snapshot!(
        test_env.run_jj_with(|cmd| cmd.current_dir(&repo_path).arg("diff").args(configs)), @r"
    ==
    file2
    --
    file2
    ==
    file3
    --
    file3
    ==
    file1
    --
    file4
    [EOF]
    ");

    // diff with file patterns
    insta::assert_snapshot!(
        test_env.run_jj_with(|cmd| {
            cmd.current_dir(&repo_path)
                .args(["diff", "file1"])
                .args(configs)
        }), @r"
    ==
    file1
    --
    file1
    [EOF]
    ");
    insta::assert_snapshot!(
        test_env.run_jj_with(|cmd| {
            cmd.current_dir(&repo_path)
                .args(["log", "-p"])
                .args(configs)
        }), @r"
    @  rlvkpnrz test.user@example.com 2001-02-03 08:05:09 7b01704a
    │  (no description set)
    │  ==
    │  file2
    │  --
    │  file2
    │  ==
    │  file3
    │  --
    │  file3
    │  ==
    │  file1
    │  --
    │  file4
    ○  qpvuntsm test.user@example.com 2001-02-03 08:05:08 6e485984
    │  (no description set)
    │  ==
    │  file1
    │  --
    │  file1
    │  ==
    │  file2
    │  --
    │  file2
    ◆  zzzzzzzz root() 00000000
    [EOF]
    ");

    insta::assert_snapshot!(
        test_env.run_jj_with(|cmd| cmd.current_dir(&repo_path).arg("show").args(configs)), @r"
    Commit ID: 7b01704a670bc77d11ed117d362855cff1d4513b
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    ==
    file2
    --
    file2
    ==
    file3
    --
    file3
    ==
    file1
    --
    file4
    [EOF]
    ");
}

#[cfg(unix)]
#[test]
fn test_diff_external_tool_symlink() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let external_file_path = test_env.env_root().join("external-file");
    std::fs::write(&external_file_path, "").unwrap();
    let external_file_permissions = external_file_path.symlink_metadata().unwrap().permissions();

    std::os::unix::fs::symlink("non-existent1", repo_path.join("dead")).unwrap();
    std::os::unix::fs::symlink(&external_file_path, repo_path.join("file")).unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("dead")).unwrap();
    std::os::unix::fs::symlink("non-existent2", repo_path.join("dead")).unwrap();
    std::fs::remove_file(repo_path.join("file")).unwrap();
    std::fs::write(repo_path.join("file"), "").unwrap();

    let edit_script = test_env.set_up_fake_diff_editor();
    std::fs::write(
        edit_script,
        "print-files-before\0print --\0print-files-after",
    )
    .unwrap();

    // Shouldn't try to change permission of symlinks
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["diff", "--tool=fake-diff-editor"]), @r"
    dead
    file
    --
    dead
    file
    [EOF]
    ");

    // External file should be intact
    assert_eq!(
        external_file_path.symlink_metadata().unwrap().permissions(),
        external_file_permissions
    );
}

#[test]
fn test_diff_external_tool_conflict_marker_style() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let file_path = repo_path.join("file");

    // Create a conflict
    std::fs::write(
        &file_path,
        indoc! {"
        line 1
        line 2
        line 3
        line 4
        line 5
    "},
    )
    .unwrap();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "base"])
        .success();
    std::fs::write(
        &file_path,
        indoc! {"
        line 1
        line 2.1
        line 2.2
        line 3
        line 4.1
        line 5
    "},
    )
    .unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "side-a"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "description(base)", "-m", "side-b"])
        .success();
    std::fs::write(
        &file_path,
        indoc! {"
        line 1
        line 2.3
        line 3
        line 4.2
        line 4.3
        line 5
    "},
    )
    .unwrap();

    // Resolve one of the conflicts in the working copy
    test_env
        .run_jj_in(
            &repo_path,
            ["new", "description(side-a)", "description(side-b)"],
        )
        .success();
    std::fs::write(
        &file_path,
        indoc! {"
        line 1
        line 2.1
        line 2.2
        line 2.3
        line 3
        <<<<<<<
        %%%%%%%
        -line 4
        +line 4.1
        +++++++
        line 4.2
        line 4.3
        >>>>>>>
        line 5
    "},
    )
    .unwrap();

    // Set up diff editor to use "snapshot" conflict markers
    let edit_script = test_env.set_up_fake_diff_editor();
    test_env.add_config(r#"merge-tools.fake-diff-editor.conflict-marker-style = "snapshot""#);

    // We want to see whether the diff is using the correct conflict markers
    std::fs::write(
        &edit_script,
        ["files-before file", "files-after file", "dump file file"].join("\0"),
    )
    .unwrap();
    let output = test_env.run_jj_in(&repo_path, ["diff", "--tool", "fake-diff-editor"]);
    insta::assert_snapshot!(output, @"");
    // Conflicts should render using "snapshot" format
    insta::assert_snapshot!(
        std::fs::read_to_string(test_env.env_root().join("file")).unwrap(), @r"
    line 1
    line 2.1
    line 2.2
    line 2.3
    line 3
    <<<<<<< Conflict 1 of 1
    +++++++ Contents of side #1
    line 4.1
    ------- Contents of base
    line 4
    +++++++ Contents of side #2
    line 4.2
    line 4.3
    >>>>>>> Conflict 1 of 1 ends
    line 5
    ");
}

#[test]
fn test_diff_stat() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    file1 | 1 +
    1 file changed, 1 insertion(+), 0 deletions(-)
    [EOF]
    ");

    test_env.run_jj_in(&repo_path, ["new"]).success();

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    0 files changed, 0 insertions(+), 0 deletions(-)
    [EOF]
    ");

    std::fs::write(repo_path.join("file1"), "foo\nbar\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "bar\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    file1 | 1 -
    1 file changed, 0 insertions(+), 1 deletion(-)
    [EOF]
    ");
}

#[test]
fn test_diff_stat_long_name_or_stat() {
    let mut test_env = TestEnvironment::default();
    test_env.add_env_var("COLUMNS", "30");
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let get_stat = |test_env: &TestEnvironment, path_length: usize, stat_size: usize| {
        test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
        let ascii_name = "1234567890".chars().cycle().take(path_length).join("");
        let han_name = "一二三四五六七八九十"
            .chars()
            .cycle()
            .take(path_length)
            .join("");
        let content = "content line\n".repeat(stat_size);
        std::fs::write(repo_path.join(ascii_name), &content).unwrap();
        std::fs::write(repo_path.join(han_name), &content).unwrap();
        test_env.run_jj_in(&repo_path, ["diff", "--stat"])
    };

    insta::assert_snapshot!(get_stat(&test_env, 1, 1), @r"
    1   | 1 +
    一  | 1 +
    2 files changed, 2 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 1, 10), @r"
    1   | 10 ++++++++++
    一  | 10 ++++++++++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 1, 100), @r"
    1   | 100 +++++++++++++++++
    一  | 100 +++++++++++++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 10, 1), @r"
    1234567890      | 1 +
    ...五六七八九十 | 1 +
    2 files changed, 2 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 10, 10), @r"
    1234567890     | 10 +++++++
    ...六七八九十  | 10 +++++++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 10, 100), @r"
    1234567890     | 100 ++++++
    ...六七八九十  | 100 ++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 50, 1), @r"
    ...901234567890 | 1 +
    ...五六七八九十 | 1 +
    2 files changed, 2 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 50, 10), @r"
    ...01234567890 | 10 +++++++
    ...六七八九十  | 10 +++++++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 50, 100), @r"
    ...01234567890 | 100 ++++++
    ...六七八九十  | 100 ++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");

    // Lengths around where we introduce the ellipsis
    insta::assert_snapshot!(get_stat(&test_env, 13, 100), @r"
    1234567890123  | 100 ++++++
    ...九十一二三  | 100 ++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 14, 100), @r"
    12345678901234 | 100 ++++++
    ...十一二三四  | 100 ++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 15, 100), @r"
    ...56789012345 | 100 ++++++
    ...一二三四五  | 100 ++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 16, 100), @r"
    ...67890123456 | 100 ++++++
    ...二三四五六  | 100 ++++++
    2 files changed, 200 insertions(+), 0 deletions(-)
    [EOF]
    ");

    // Very narrow terminal (doesn't have to fit, just don't crash)
    test_env.add_env_var("COLUMNS", "10");
    insta::assert_snapshot!(get_stat(&test_env, 10, 10), @r"
    ... | 10 ++
    ... | 10 ++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
    test_env.add_env_var("COLUMNS", "3");
    insta::assert_snapshot!(get_stat(&test_env, 10, 10), @r"
    ... | 10 ++
    ... | 10 ++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 3, 10), @r"
    123 | 10 ++
    ... | 10 ++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
    insta::assert_snapshot!(get_stat(&test_env, 1, 10), @r"
    1   | 10 ++
    一  | 10 ++
    2 files changed, 20 insertions(+), 0 deletions(-)
    [EOF]
    ");
}

#[test]
fn test_diff_binary() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1.png"), b"\x89PNG\r\n\x1a\nabcdefg\0").unwrap();
    std::fs::write(repo_path.join("file2.png"), b"\x89PNG\r\n\x1a\n0123456\0").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1.png")).unwrap();
    std::fs::write(repo_path.join("file2.png"), "foo\nbar\n").unwrap();
    std::fs::write(repo_path.join("file3.png"), b"\x89PNG\r\n\x1a\nxyz\0").unwrap();
    // try a file that's valid UTF-8 but contains control characters
    std::fs::write(repo_path.join("file4.png"), b"\0\0\0").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["diff"]);
    insta::assert_snapshot!(output, @r"
    Removed regular file file1.png:
        (binary)
    Modified regular file file2.png:
        (binary)
    Added regular file file3.png:
        (binary)
    Added regular file file4.png:
        (binary)
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--git"]);
    insta::assert_snapshot!(output, @r"
    diff --git a/file1.png b/file1.png
    deleted file mode 100644
    index 2b65b23c22..0000000000
    Binary files a/file1.png and /dev/null differ
    diff --git a/file2.png b/file2.png
    index 7f036ce788..3bd1f0e297 100644
    Binary files a/file2.png and b/file2.png differ
    diff --git a/file3.png b/file3.png
    new file mode 100644
    index 0000000000..deacfbc286
    Binary files /dev/null and b/file3.png differ
    diff --git a/file4.png b/file4.png
    new file mode 100644
    index 0000000000..4227ca4e87
    Binary files /dev/null and b/file4.png differ
    [EOF]
    ");

    let output = test_env.run_jj_in(&repo_path, ["diff", "--stat"]);
    insta::assert_snapshot!(output, @r"
    file1.png | 3 ---
    file2.png | 5 ++---
    file3.png | 3 +++
    file4.png | 1 +
    4 files changed, 6 insertions(+), 6 deletions(-)
    [EOF]
    ");
}

#[test]
fn test_diff_revisions() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    //
    //
    //
    // E
    // |\
    // C D
    // |/
    // B
    // |
    // A
    create_commit(&test_env, &repo_path, "A", &[]);
    create_commit(&test_env, &repo_path, "B", &["A"]);
    create_commit(&test_env, &repo_path, "C", &["B"]);
    create_commit(&test_env, &repo_path, "D", &["B"]);
    create_commit(&test_env, &repo_path, "E", &["C", "D"]);

    let diff_revisions = |expression: &str| -> CommandOutput {
        test_env.run_jj_in(&repo_path, ["diff", "-s", "-r", expression])
    };
    // Can diff a single revision
    insta::assert_snapshot!(diff_revisions("B"), @r"
    A B
    [EOF]
    ");

    // Can diff a merge
    insta::assert_snapshot!(diff_revisions("E"), @r"
    A E
    [EOF]
    ");

    // A gap in the range is not allowed (yet at least)
    insta::assert_snapshot!(diff_revisions("A|C"), @r"
    ------- stderr -------
    Error: Cannot diff revsets with gaps in.
    Hint: Revision 50c75fd767bf would need to be in the set.
    [EOF]
    [exit status: 1]
    ");

    // Can diff a linear chain
    insta::assert_snapshot!(diff_revisions("A::C"), @r"
    A A
    A B
    A C
    [EOF]
    ");

    // Can diff a chain with an internal merge
    insta::assert_snapshot!(diff_revisions("B::E"), @r"
    A B
    A C
    A D
    A E
    [EOF]
    ");

    // Can diff a set with multiple roots
    insta::assert_snapshot!(diff_revisions("C|D|E"), @r"
    A C
    A D
    A E
    [EOF]
    ");

    // Can diff a set with multiple heads
    insta::assert_snapshot!(diff_revisions("B|C|D"), @r"
    A B
    A C
    A D
    [EOF]
    ");

    // Can diff a set with multiple root and multiple heads
    insta::assert_snapshot!(diff_revisions("B|C"), @r"
    A B
    A C
    [EOF]
    ");
}
