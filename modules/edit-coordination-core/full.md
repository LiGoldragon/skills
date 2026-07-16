# Skill - edit coordination

## Coordination Rules

Before writing shared files, register the assigned Session/Lane with `meta-orchestrate`, then claim the exact repository or path with `orchestrate`. If Session, Lane, or Fresh/Recovery mode is missing, stop and return the missing identity to the assigning authority. Use task-specific PascalCase alphanumeric names; never substitute a generic role name.

```sh
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <detail>) Fresh))"
orchestrate "(Claim (<LaneName> [(Path <absolute-path>)] <reason>))"
```

Registration is the atomic conflict check. Do not observe first. A Fresh duplicate blocks the work. Accept Recovery inheritance only when the active lane matches the assigned recovery context; a released or unregistered inherited lane grants no mutation authority.

A claim grants coordination ownership, not proof that the path exists or is the intended checkout. After acceptance, verify the path and repository identity. For a new file, verify its parent. Never claim `.beads/`, edit projected lock files, or share a checkout that is already claimed or visibly active.

The deployed client may not support isolated-worktree creation or conclusion. Do not guess an unavailable operation. Report the rejected capability and request explicit fallback authority. The assigning authority may authorize a specific isolated clone or workspace, its exact claimed path, creation method, publication target, and cleanup disposition. Claim that path before creation and follow only the authorized fallback.

Keep an owned long-running command in the foreground until it finishes. A background waiter dies with the turn and can park the lane silently.

Observe only after registration when live coordination state is evidence:

```sh
orchestrate "(Observe Sessions)"
orchestrate "(Observe Lanes)"
orchestrate "(Observe (SessionLanes <SessionName>))"
```

After the owned work is committed and pushed, release only this lane's claims and unregister it:

```sh
orchestrate "(Release <LaneName>)"
meta-orchestrate "(Unregister (<SessionName> <LaneName> <detail>))"
```
