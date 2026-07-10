# Module - edit coordination core

## Edit Coordination

Before editing shared files or running a command that writes them, register the assigned Session/Lane with `meta-orchestrate`, then claim the exact path or repository with ordinary Orchestrate under that lane. The ordinary claim field is role-shaped, but it carries the lane identity.

If the task needs editing and no session name, lane name, or Fresh/Recovery mode is assigned, pause and report the missing coordination identity. Do not use generic names such as `general-code-implementer`, `skill-editor`, or `rust-auditor`.

Lane registration is the atomic check. Do not pre-observe before registration. Treat Fresh duplicate registration as a conflict/blocker. Treat manager-declared Recovery duplicate as inherited only when the active lane clearly matches this recovery context.

Keep an owned long-running operation's wait in the foreground within the turn. Never end a turn with the operation still in flight expecting a background waiter to resume it; the waiter dies with the turn and the lane parks silently.

Do not edit projected lock files by hand.

```sh
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <detail-string>) Fresh))"
orchestrate "(Claim (<LaneName> [(Path /absolute/path)] <reason-string>))"
orchestrate "(Release <LaneName>)"
meta-orchestrate "(Unregister (<SessionName> <LaneName> <detail-string>))"
```

Use exactly one NOTA string object in each detail or reason slot. Prefer a single bare atom such as `coordination-doctrine`. For multi-word text, use the bracket string form accepted by String slots, such as `[refresh coordination docs]`. Do not write multi-word bare text; it is parsed as extra positional objects and fails.

Observe only when coordination state is evidence after registration or during audit. When relaying observed claims, show direct age, not only a start timestamp.

```sh
orchestrate "(Observe Sessions)"
orchestrate "(Observe Lanes)"
orchestrate "(Observe (SessionLanes <SessionName>))"
```

Do not claim `.beads/`. Treat an Orchestrate claim on `.beads/` as invalid agent policy state; force-release or remove that claim instead of treating it as a lock.

If the local repository or worktree is already claimed or visibly in use, do not share that checkout. Start from `main` in an isolated feature worktree, claim that worktree path under the registered lane, and file a bead naming the repository, branch, worktree, and required final disposition: discard, partial merge, or full merge.

```sh
bd create "Track <branch> worktree" -t task -p 2 --description "<repo>; <branch>; <worktree>; disposition needed" --labels feature-branch,worktree
```

For Git worktrees managed by beads, create from a clean `main` checkout with `bd worktree create <worktree> --branch <branch>`. In JJ workspaces, create from `main` with `jj workspace add --revision main --message '<branch>' <worktree>` and move the feature bookmark to the completed commit with `jj bookmark set <branch> -r @-`.

When daemon worktree inventory is needed, the meta API shape is:

```sh
meta-orchestrate "(RegisterWorktree (Worktree <repo> <branch> /absolute/path <lane> Active <purpose> <timestamp-nanos> Unpushed))"
```
