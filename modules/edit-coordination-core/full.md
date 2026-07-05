# Module - edit coordination core

## Edit Coordination

Before editing shared files or running a command that writes them, claim the
exact path or repository with Orchestrate. Use the registered session lane when
one is supplied for this work; otherwise use the dispatcher-assigned unique,
meaningful coordination name. This interim current-Orchestrate compatibility
keeps same-role workers from releasing each other's claims while first-class
session lanes are not deployed.

If no unique coordination name is assigned and the task needs a claim, pause and
ask or report the missing name. Do not use generic role names such as
`general-code-implementer`, `skill-editor`, or `rust-auditor` as claim owners.
Release only claims you made under your assigned name.

Do not edit projected lock files by hand.

```sh
orchestrate "(Observe Roles)"
orchestrate "(Claim (<assigned-name> [(Path /absolute/path)] [reason]))"
orchestrate "(Release <assigned-name>)"
```

If the local repository or worktree is already claimed or visibly in use, do
not share that checkout. Start from `main` in an isolated feature worktree,
claim that worktree path, and file a bead naming the repository, branch,
worktree, and required final disposition: discard, partial merge, or full
merge.

```sh
bd create "Track <branch> worktree" -t task -p 2 --description "<repo>; <branch>; <worktree>; disposition needed" --labels feature-branch,worktree
```

For Git worktrees managed by beads, create from a clean `main` checkout with
`bd worktree create <worktree> --branch <branch>`. In JJ workspaces, create
from `main` with `jj workspace add --revision main --message '<branch>'
<worktree>` and move the feature bookmark to the completed commit with
`jj bookmark set <branch> -r @-`.

When daemon worktree inventory is needed, the meta API shape is:

```sh
meta-orchestrate "(RegisterWorktree (Worktree <repo> <branch> /absolute/path <lane> Active <purpose> <timestamp-nanos> Unpushed))"
```
