# Skill — Jujutsu discipline

## Use jj, not raw git

Use `jj` for version control. Raw `git` is only an escape hatch named in this skill. Every description-taking command uses an inline message or equivalent headless flag; never let a command open an editor.

## Primary workspace stays on main

In primary, work directly on `main`. Do not create branches. Commit the complete working copy, then move `main` to the committed parent and push.

```sh
jj status --no-pager
jj commit -m 'short imperative message'
jj bookmark set main -r @-
jj git push --bookmark main
```

If unrelated dirty files exist, name them and avoid staging concepts; jj commits the working copy as a whole unless a repo-specific instruction explicitly permits path-scoped surgery.

## Code repos use logical commits

Outside primary, keep one logical change per commit. Inspect status before and after edits. Use concise imperative commit messages that name the behavior changed.

```sh
jj status --no-pager
jj diff --stat
jj commit -m 'component: change behavior'
jj bookmark set main -r @-
jj git push --bookmark main
```

Push each completed logical commit. Do not accumulate a local stack that is ready but unpushed.

## Descriptions are explicit

Do not run `jj describe @` as a finalization step. Do not leave a real commit descriptionless. If a command would open an editor, cancel and rerun with `-m`, `--message`, or the command's headless equivalent.

## Routine checks

Before committing, run the narrow validation that proves the change. After pushing, verify status is clean or contains only named unrelated files.

Useful reads:

```sh
jj status --no-pager
jj log -r 'main..@' --no-pager
jj diff --stat
jj show --stat --no-pager
```

## Fix uninitialized repos

If a repository lacks jj metadata, initialize colocated jj and track the existing default bookmark before editing.

```sh
jj git init --colocate
jj bookmark track main@origin
jj status --no-pager
```

## Raw-git escape hatches

If a remote URL blocks push mechanics, use raw git only to inspect or change the remote configuration, then return to jj.

If push is rejected because the remote advanced, stop normal work. Fetch with jj, inspect divergence, and ask before rebasing or force-moving shared history unless the task explicitly authorizes that repair.

## Restore carefully

`jj restore` discards working-copy content. Use it only when the exact path and loss are understood. Prefer reading diffs and making a forward edit.

## Forbidden shortcuts

- Do not use `jj git push -c @` for routine commits.
- Do not create anonymous descriptionless checkpoints to satisfy process.
- Do not path-scope commits in primary.
- Do not use raw git for ordinary add, commit, branch, merge, or push work.
