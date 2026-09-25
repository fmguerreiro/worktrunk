---
title: "wt prune"
description: "Remove worktrees and branches merged into the default branch."
sidebar:
  order: 14
---
<!-- ⚠️ AUTO-GENERATED from `wt prune --help-page` — edit src/cli/mod.rs to update -->

Remove worktrees and branches merged into the default branch.

Bulk-removes worktrees and branches that are integrated into the default branch, using the same criteria as `wt remove`'s branch cleanup. Stale worktree entries are cleaned up too, except one whose git metadata holds staged changes or an operation in progress; `git worktree repair` can still restore those.

In `wt list`, candidates show `_` (same commit) or `⊂` (content integrated). Run `--dry-run` to preview. See `wt remove --help` for the full integration criteria.

Locked worktrees, worktrees with uncommitted changes, and the main worktree are always skipped. The current worktree is removed last, triggering cd to the primary worktree. Pre-remove and post-remove hooks run for each removal; a candidate whose hooks include an unapproved project command is skipped with `(approval required)` (pre-approve with `wt config approvals add`, or pass `--yes`).

## Min-age guard

Candidates younger than `--min-age` (default: 1 day) are skipped. A worktree's age comes from its creation time. A branch with no worktree takes its age from its oldest reflog entry, or, when it has none (common in bare repositories), from when git last wrote its ref. Operations such as `git gc` or deleting a branch can rewrite many refs at once, so afterwards older branches without a reflog are skipped until `--min-age` has passed. This prevents removing a worktree just created from the default branch: it looks "merged" because its branch points at the same commit.

```console
$ wt prune --min-age=0s     # no age guard
$ wt prune --min-age=2d     # skip candidates younger than 2 days
```

## JSON output

`--format=json` prints one object per candidate to stdout. A live run reports `branch_outcome`, as [`wt remove`](/remove/#json-output) does. `--dry-run` reports `branch_deleted` (whether the removal would delete the branch), `reason` (why the candidate qualifies), and `target` (what it was measured against).

## Examples

Preview what would be removed:

```console
$ wt prune --dry-run
```

Remove all merged worktrees:

```console
$ wt prune
```

## Command reference

```text wt-command-reference
wt prune - Remove worktrees and branches merged into the default branch

Usage: wt prune [OPTIONS]

Options:
      --dry-run
          Show what would be removed

      --min-age <MIN_AGE>
          Skip worktrees and branches younger than this

          [default: 1d]

      --foreground
          Run removal in foreground (block until complete)

      --format <FORMAT>
          Output format

          [default: text]
          [possible values: text, json]

  -h, --help
          Print help (see a summary with '-h')

Global Options:
  -C <path>
          Working directory for this command

      --config <path>
          User config file path

      --config-set <toml>
          Override config with inline TOML, e.g. --config-set list.full=true (repeatable)

  -v, --verbose...
          Verbose output (-v: info logs + hook/alias template variables on stderr; -vv: also debug
          logs and raw subprocess output written to .git/wt/logs/). Set WORKTRUNK_VERBOSE=0|1|2 to
          apply the same level everywhere — including shell completion, which no flag can reach

  -y, --yes
          Skip approval prompts
```

<!-- END AUTO-GENERATED -->
