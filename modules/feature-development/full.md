# Skill — feature development

## Rules

Use feature branches or separate worktrees for code-repo feature work, experiments, rewrites, and prototypes that may ship. Do not do feature work directly on a shared integration line unless the repo instructions explicitly say so.

The parent or task owner names the branch and scope before work starts. A worker edits only that branch or worktree and returns the exact validation evidence.

Keep radical experiments in the existing repository. A branch may replace the whole tree for a prototype; that still does not justify a new repository.

Claim the worktree or task scope when the workspace uses claims. Different worktrees of the same repository are separate claim scopes; the same worktree is a conflict.

At merge or abandonment, remove stale worktrees and branches so later agents do not mistake them for live work.

Subagents that edit code or produce ship-ready prototypes use their assigned feature branch or worktree. Research-only workers that write only their assigned output need no worktree.
