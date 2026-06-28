# Module - repo operation core

## Operation Core Purpose

Repository operation closes validated work: status review, commit, push, branch
or bookmark mechanics, bead closeout, and final handoff. It does not replace
implementation or audit.

## Operation Status First

Read local repository instructions and inspect status before changing history.
Preserve peer edits and do not revert unrelated work. If validation or audit
evidence is missing, record the gap instead of manufacturing a green closeout.

Use `jj` for normal version control. Every description-taking command uses an
inline message so no editor opens. Commit the working copy only when the brief
authorizes a partial handoff or the validation/audit gates are satisfied.

## Operation Branch And Bookmark Shape

Primary lands on `main` directly. Code repositories under `/git` follow their
repo's branch or bookmark policy: operator-owned `main`, designer or feature
work on the named long-lived or task branch, and integration only after
producer refs are available for consumers.

Use `gh` for GitHub repository metadata and issue or pull-request operations.
Use `ghq` for locating or updating local clones. Raw `git` is reserved for the
documented recovery and remote-configuration cases.

## Operation Push And Closeout

Before pushing, confirm bookmark reachability, repository status, and that no
descriptionless authored commit is being published. Push the intended bookmark
and report the result. Close tracked tasks only after the durable evidence
exists, naming the commit, output file, validation report, or superseding task.
