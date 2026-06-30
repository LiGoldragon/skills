# Skill - edit coordination

## Rules

Before editing shared files or running commands that write them, inspect current ownership with Orchestrate and claim the exact path or repository. Use the session lane when one exists; otherwise use your role name.

```sh
orchestrate "(Observe Roles)"
orchestrate "(Claim (<lane> [(Path <absolute-path>)] [reason]))"
orchestrate "(Release <lane>)"
```

Do not edit projected lock files by hand. If a checkout is already claimed or visibly in use, do not share it; create an isolated `main`-based worktree or JJ workspace, claim that path, and file a BEADS/beads item naming the repository, branch, worktree, and required disposition: discard, partial merge, or full merge.

For bead-managed Git worktrees, use `bd worktree create <worktree> --branch <branch>`. For JJ workspaces, use `jj workspace add --revision main --message '<branch>' <worktree>` and move the feature bookmark to the completed commit with `jj bookmark set <branch> -r @-`.
