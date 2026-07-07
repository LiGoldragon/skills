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
inline message so no editor opens. Commit the working copy when the brief
authorizes a partial handoff or the validation and audit gates are satisfied.
For file edits made by the current agent, the editing closeout rule still
requires commit and push before final output; missing validation is evidence to
report, not permission to leave edits unpublished.

Raw `git` is reserved for remote inspection or configuration, and for recovery
only when the repo guidance or push rejection requires it.

## Operation Branch And Bookmark Shape

Primary lands on `main` directly:

```sh
jj status --no-pager
jj commit -m 'short imperative message'
jj bookmark set main -r @-
jj git push --bookmark main
```

Code repositories keep one logical change per commit. Follow the repo's branch
or bookmark policy: integration-owned `main`, design or feature work on the
named long-lived or task branch, and integration only after producer refs are
available for consumers.

For main feature integration, start from current `main`, work on a named
integration bookmark while the feature is not green, test the affected branch
family together, rebase on moved `main` before landing, then land producers
before consumers. Remove temporary local path overrides before the merge-ready
state unless the branch dependency is intentional and documented.
If the work creates or consumes a producer dependency, make that dependency
portable before publishing. Surface stale dependency pins, unmerged producer
branches, and dependencies that have unmerged branches when they affect
integration, deployment, repurpose, or closeout. If portable closeout is not
possible, report it as a hard blocker.

If a local repository or worktree is already claimed, do not share it. Create an
isolated main-based feature worktree or workspace, claim that path, and file a
tracker item naming the repository, branch, worktree, and needed final
disposition: discard, partial merge, or full merge.

## Operation Work Tracking

Use tracked work items when work must survive the session or coordinate
with other work. Before working an item, inspect its state and dependencies, then
claim only the item actively being worked.

Create executable item text: desired outcome, owning repository or component,
likely files or surfaces, acceptance criteria, dependencies, blockers, and
expected verification. Wire producer-before-consumer dependencies explicitly.

Close an item only after the acceptance criteria pass or the item is explicitly
invalidated. Closing notes name durable evidence: commit, output file,
validation artifact, or superseding task. If blocked, leave it open and name
the blocker.

## Operation Push And Closeout

Before pushing, confirm bookmark reachability, repository status, and that no
descriptionless authored commit is being published. Push the intended bookmark
and return the result.

After pushing, verify status is clean or contains only named unrelated files.
Report basis commit, branch bookmark, temporary overrides used for testing,
commands run, push result, and any remaining disposition or follow-up.
