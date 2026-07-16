---
name: planner
description: 'Planner role.'
model: 'openai-codex/gpt-test'
thinking: high
projectRoleIdentity: planner
projectRoleDispatchKind: nested
allowedChildRoleNames:
  - reader
  - writer
---

# planner

## Contract

Plan work.
