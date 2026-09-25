//! Integration tests for the top-level `wt prune`; `step_prune.rs` covers the
//! operation itself.

use crate::common::{TestRepo, repo, wt_command};
use ansi_str::AnsiStr as _;
use rstest::rstest;

fn branch_exists(repo: &TestRepo, branch: &str) -> bool {
    repo.git_command()
        .args(["show-ref", "--verify", "--quiet"])
        .arg(format!("refs/heads/{branch}"))
        .run()
        .unwrap()
        .status
        .success()
}

/// Prune finishes its removals before returning, so the branch is gone the
/// instant the command exits.
#[rstest]
fn test_prune_removes_merged_worktree_and_branch(mut repo: TestRepo) {
    repo.commit("initial");
    repo.add_worktree("merged-branch");

    let output = repo
        .wt_command()
        .args(["prune", "--yes", "--min-age=0s"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .ansi_strip()
        .into_owned();
    assert!(
        output.status.success(),
        "wt prune should succeed:\n{stderr}"
    );

    let worktree_path = repo
        .root_path()
        .parent()
        .unwrap()
        .join("repo.merged-branch");
    assert!(
        !worktree_path.exists(),
        "wt prune should remove the integrated worktree:\n{stderr}"
    );
    assert!(
        !branch_exists(&repo, "merged-branch"),
        "wt prune should delete the integrated branch:\n{stderr}"
    );
}

#[rstest]
fn test_prune_dry_run_previews_without_removing(mut repo: TestRepo) {
    repo.commit("initial");
    repo.add_worktree("merged-a");
    repo.add_worktree("merged-b");

    let output = repo
        .wt_command()
        .args(["prune", "--dry-run", "--min-age=0s"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout)
        .ansi_strip()
        .into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .ansi_strip()
        .into_owned();
    assert!(
        output.status.success(),
        "wt prune --dry-run should succeed:\n{stderr}"
    );

    assert!(
        stdout.contains("merged-a") && stdout.contains("merged-b"),
        "the preview should name both candidates:\n{stdout}"
    );

    let parent = repo.root_path().parent().unwrap();
    for branch in ["merged-a", "merged-b"] {
        assert!(
            parent.join(format!("repo.{branch}")).exists(),
            "--dry-run must not remove {branch}'s worktree:\n{stdout}"
        );
        assert!(
            branch_exists(&repo, branch),
            "--dry-run must not delete branch {branch}:\n{stdout}"
        );
    }
}

/// Under the test epoch a fresh worktree reads as younger than the 1d default,
/// so the guard skips it until `--min-age=0s`.
#[rstest]
fn test_prune_min_age_zero_drops_the_age_guard(mut repo: TestRepo) {
    repo.commit("initial");
    repo.add_worktree("young-branch");
    let worktree_path = repo.root_path().parent().unwrap().join("repo.young-branch");

    let guarded = repo.wt_command().args(["prune", "--yes"]).output().unwrap();
    let guarded_stderr = String::from_utf8_lossy(&guarded.stderr)
        .ansi_strip()
        .into_owned();
    assert!(
        guarded.status.success(),
        "wt prune should succeed:\n{guarded_stderr}"
    );
    assert!(
        worktree_path.exists(),
        "the default --min-age should hold back a worktree this young:\n{guarded_stderr}"
    );

    let dropped = repo
        .wt_command()
        .args(["prune", "--yes", "--min-age=0s"])
        .output()
        .unwrap();
    let dropped_stderr = String::from_utf8_lossy(&dropped.stderr)
        .ansi_strip()
        .into_owned();
    assert!(
        dropped.status.success(),
        "wt prune --min-age=0s should succeed:\n{dropped_stderr}"
    );
    assert!(
        !worktree_path.exists(),
        "--min-age=0s should drop the age guard and remove the worktree:\n{dropped_stderr}"
    );
}

/// `--dry-run` mutates nothing, so both spellings see the same repository, and
/// `RAYON_NUM_THREADS=1` fixes the scan's completion order.
#[rstest]
fn test_prune_and_step_prune_are_the_same_operation(mut repo: TestRepo) {
    repo.commit("initial");
    repo.add_worktree("merged-a");
    repo.add_worktree("merged-b");
    repo.add_worktree_with_commit("unmerged", "u.txt", "content", "unmerged commit");

    let step = repo
        .wt_command()
        .args(["step", "prune", "--dry-run", "--min-age=0s"])
        .env("RAYON_NUM_THREADS", "1")
        .output()
        .unwrap();
    let top = repo
        .wt_command()
        .args(["prune", "--dry-run", "--min-age=0s"])
        .env("RAYON_NUM_THREADS", "1")
        .output()
        .unwrap();

    let step_stdout = String::from_utf8_lossy(&step.stdout).into_owned();
    let top_stdout = String::from_utf8_lossy(&top.stdout).into_owned();
    let step_stderr = String::from_utf8_lossy(&step.stderr).into_owned();
    let top_stderr = String::from_utf8_lossy(&top.stderr).into_owned();

    assert!(
        step.status.success() && top.status.success(),
        "both spellings should succeed:\nstep: {step_stderr}\nprune: {top_stderr}"
    );
    assert!(
        step_stdout.contains("merged-a") && step_stdout.contains("merged-b"),
        "fixture should offer candidates to compare:\n{step_stdout}"
    );
    assert_eq!(
        step_stdout, top_stdout,
        "wt prune and wt step prune must preview the same removals"
    );
    assert_eq!(
        step_stderr, top_stderr,
        "wt prune and wt step prune must narrate identically"
    );
}

#[test]
fn test_prune_is_listed_in_top_level_help() {
    let output = wt_command().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout)
        .ansi_strip()
        .into_owned();

    let listed = stdout.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("prune") && line.contains("merged into the default branch")
    });
    assert!(
        listed,
        "`wt --help` must list prune with its description:\n{stdout}"
    );
}

#[test]
fn test_remove_help_points_at_prune() {
    let output = wt_command().args(["remove", "--help"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout)
        .ansi_strip()
        .into_owned();

    assert!(
        stdout.contains("run wt prune"),
        "`wt remove --help` must tell the reader how to remove every integrated \
         worktree at once:\n{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.trim_start().starts_with("- wt prune")),
        "`wt remove --help` should list wt prune under See also:\n{stdout}"
    );
}
