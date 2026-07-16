# Skill - session lanes

## Rules

A Session groups related work; a Lane names one worker's live task and carries its role and authority. The Manager assigns task-specific PascalCase alphanumeric names plus Fresh/Recovery mode. Never invent a missing identity or use a generic role as the lane.

Register through `meta-orchestrate` before observing or claiming. Registration is the atomic conflict check: a Fresh duplicate blocks; Recovery inherits only when the active lane matches the assigned recovery context.

```sh
meta-orchestrate "(Register ((<SessionName> <LaneName> ([<RoleToken>...] Structural) <detail>) Fresh))"
```

Use ordinary Orchestrate to observe live state only when it is evidence, and show direct claim age rather than only timestamps. Age informs judgment; it is not a heartbeat.

```sh
orchestrate "(Observe Sessions)"
orchestrate "(Observe Lanes)"
orchestrate "(Observe (SessionLanes <SessionName>))"
```

Register before write commands, then claim exact resources under the lane. Keep an owned long-running operation in the foreground until it finishes.

At closeout, use only lifecycle operations the deployed client supports. If required isolated-workspace cleanup is unavailable, obtain explicit fallback disposition from the assigning authority. Release this lane's claims and unregister it; clear a Session only when Manager owns cleanup or all remaining lanes are yours.

```sh
meta-orchestrate "(Unregister (<SessionName> <LaneName> <detail>))"
```

Handover ends active lanes. A later worker receives a new lane or explicit Recovery. Route leftovers to durable intent, tracked work, owning source guidance, or deliberate abandonment.
