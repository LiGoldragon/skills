# Skill - edit coordination

## Rules

Before editing shared files or running commands that write them, inspect current ownership with Orchestrate and claim the exact path or repository. Use the registered session lane when one is supplied for this work; otherwise use the dispatcher-assigned unique, meaningful coordination name. This interim current-Orchestrate compatibility keeps same-role workers from releasing each other's claims while first-class session lanes are not deployed.

If no unique coordination name is assigned and the task needs a claim, pause and ask or report the missing name. Do not use generic role names such as `general-code-implementer`, `skill-editor`, or `rust-auditor` as claim owners. Release only claims you made under your assigned name.

```sh
orchestrate "(Observe Roles)"
orchestrate "(Claim (<assigned-name> [(Path <absolute-path>)] [reason]))"
orchestrate "(Release <assigned-name>)"
```

Do not edit projected lock files by hand. If a checkout is already claimed or visibly in use, do not share it; create an isolated `main`-based worktree or JJ workspace, claim that path, and file a BEADS/beads item naming the repository, branch, worktree, and required disposition: discard, partial merge, or full merge.

For bead-managed Git worktrees, use `bd worktree create <worktree> --branch <branch>`. For JJ workspaces, use `jj workspace add --revision main --message '<branch>' <worktree>` and move the feature bookmark to the completed commit with `jj bookmark set <branch> -r @-`.
