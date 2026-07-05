# Skill — session lanes

## Rules

A Session is a first-class cognitive grouping named by the orchestrator in CamelCase. It groups related lanes and is not an edit lock.

A Lane belongs to one Session and names one worker's live task. The orchestrator assigns each editing worker a meaningful lane name. Do not use a role, discipline, or generic agent type as the lane name.

The lane carries role and authority metadata. The role says what kind of agent acts; the lane says what this worker is doing now.

If a worker needs to edit but the brief lacks a session name, lane name, or Fresh/Recovery mode, pause and report the missing coordination identity instead of inventing one.

## Registration

Lane lifecycle mutation is meta-owned: register a lane, unregister a lane, and clear or end a session through `meta-orchestrate`. Agents may call meta lifecycle directly until an engine owns registration for them.

Lane registration is the atomic check. Do not pre-observe before registration.

```sh
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <details>) Fresh))"
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <details>) Recovery))"
```

A Fresh duplicate registration is a conflict and blocker. An orchestrator-declared Recovery duplicate inherits the active lane and may proceed when the returned active lane matches the recovery context.

Observe with the ordinary Orchestrate surface when coordination state is evidence: sessions, all lanes, or lanes in one session. Lane observations include age, status, and resource claims.

```sh
orchestrate "(Observe Sessions)"
orchestrate "(Observe Lanes)"
orchestrate "(Observe (SessionLanes <SessionName>))"
```

Large passive timeout or age is evidence for judgment only; do not invent a heartbeat requirement.

## Lifecycle

Before editing shared files or running write commands, register the assigned lane, then make ordinary Orchestrate claims under that lane.

At closeout, release the lane's resource claims and unregister that lane. Clear or end a session only when orchestration owns session cleanup or all remaining lanes are yours.

```sh
meta-orchestrate "(Unregister (<SessionName> <LaneName> <details>))"
meta-orchestrate "(ClearSession (<SessionName> <details>))"
```

Handover ends active lanes. Do not inherit lanes through handover; the next worker receives a new lane or an explicit Recovery registration. Put handover content in chat or the response, not only in a file.

At drain, route every leftover idea to exactly one fate: accepted durable intent, tracked work, owned source documentation, or abandoned as landed, stale, or wrong.
