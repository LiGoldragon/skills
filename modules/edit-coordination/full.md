# Skill - edit coordination

## Rules

Before editing shared files or running commands that write them, register the assigned Session/Lane with `meta-orchestrate`, then claim the exact path or repository with ordinary Orchestrate under that lane. The ordinary claim field is role-shaped, but it carries the lane identity.

If the task needs editing and no session name, lane name, or Fresh/Recovery mode is assigned, pause and report the missing coordination identity. Do not use generic names such as `general-code-implementer`, `skill-editor`, or `rust-auditor`.

Lane registration is the atomic check. Do not pre-observe before registration. Treat Fresh duplicate registration as a conflict/blocker. Treat orchestrator-declared Recovery duplicate as inherited only when the active lane clearly matches this recovery context.

```sh
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <details>) Fresh))"
orchestrate "(Claim (<LaneName> [(Path <absolute-path>)] <reason>))"
orchestrate "(Release <LaneName>)"
meta-orchestrate "(Unregister (<SessionName> <LaneName> <details>))"
```

Observe only when coordination state is evidence after registration or during audit:

```sh
orchestrate "(Observe Sessions)"
orchestrate "(Observe Lanes)"
orchestrate "(Observe (SessionLanes <SessionName>))"
```

Do not edit projected lock files by hand. If a checkout is already claimed or visibly in use, do not share it; create an isolated `main`-based worktree or JJ workspace, claim that path under the registered lane, and file a BEADS/beads item naming the repository, branch, worktree, and required disposition: discard, partial merge, or full merge.

For bead-managed Git worktrees, use `bd worktree create <worktree> --branch <branch>`. For JJ workspaces, use `jj workspace add --revision main --message '<branch>' <worktree>` and move the feature bookmark to the completed commit with `jj bookmark set <branch> -r @-`.
