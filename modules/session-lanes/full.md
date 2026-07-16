# Skill — session lanes

## Rules

A Session is a first-class cognitive grouping named by the Manager in PascalCase alphanumeric. It groups related lanes and is not an edit lock.

A Lane belongs to one Session and names one worker's live task. The Manager assigns each editing worker a meaningful lane name. Do not use a role, discipline, or generic agent type as the lane name.

The lane carries role and authority metadata. The role says what kind of agent acts; the lane says what this worker is doing now.

If a worker needs to edit but the brief lacks a session name, lane name, or Fresh/Recovery mode, pause and report the missing coordination identity instead of inventing one.

## Registration

Lane lifecycle mutation is meta-owned: register a lane, unregister a lane, and clear or end a session through `meta-orchestrate`. Agents may call meta lifecycle directly until an engine owns registration for them.

Lane registration is the atomic check. Do not pre-observe before registration.

```sh
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <detail-string>) Fresh))"
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <detail-string>) Recovery))"
```

Name sessions and lanes in PascalCase alphanumeric — an uppercase first letter, then letters and digits only (`OsDeploymentDoctrine`, `SkillDriftReview`). The daemon strictly enforces this for the session name; its error text calls it `CamelCase alphanumeric`.

Use exactly one NOTA string object in each detail slot. Write a single canonical word bare (`done`, `coordination-doctrine`), never bracketed — the daemon rejects `[done]` and accepts `done`. Reserve the bracket form for genuinely multi-word text, such as `[coordination doctrine]`. Do not write multi-word bare text; it is parsed as extra positional objects and fails.

A Fresh duplicate registration is a conflict and blocker. A manager-declared Recovery duplicate inherits the active lane and may proceed when the returned active lane matches the recovery context. To resume a lane this session previously registered and released, register it in Recovery mode, not Fresh; Fresh conflicts with the session's own released record.

Observe with the ordinary Orchestrate surface when coordination state is evidence: sessions, all lanes, or lanes in one session. Lane observations include age, status, and resource claims. When showing claim information to agents, include direct age rather than only a wall-clock or start timestamp.

```sh
orchestrate "(Observe Sessions)"
orchestrate "(Observe Lanes)"
orchestrate "(Observe (SessionLanes <SessionName>))"
```

Large passive timeout or age is evidence for judgment only; do not invent a heartbeat requirement.

## Lifecycle

Before editing shared files or running write commands, register the assigned lane, then make ordinary Orchestrate claims under that lane.

Keep an owned long-running operation's wait in the foreground within the turn. Never end a turn with an owned operation still in flight expecting a background waiter to resume it; the waiter dies with the turn and the lane parks silently until someone notices.

At closeout, a lane that owns a worktree concludes it with `ConcludeWorktree` (Merged or Rejected) so the orchestrator tears down the workspace; then release the lane's resource claims and unregister that lane. Clear or end a session only when Manager owns session cleanup or all remaining lanes are yours.

```sh
meta-orchestrate "(Unregister (<SessionName> <LaneName> <detail-string>))"
meta-orchestrate "(ClearSession (<SessionName> <detail-string>))"
```

Handover ends active lanes. Do not inherit lanes through handover; the next worker receives a new lane or an explicit Recovery registration. Put handover content in chat or the response, not only in a file.

At drain, route every leftover idea to exactly one fate: accepted durable intent, tracked work, owned source documentation, or abandoned as landed, stale, or wrong.
