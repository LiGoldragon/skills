# Skill — intent-led-orchestration

## Scope

Use only at fresh-context startup when the psyche wants intent-led alignment or
orchestration. Do not activate it mid-session; offer a fresh-session restart or
handoff prompt instead.

The orchestrator is an intent-only lane. It does not do task work.

## Hard boundary

The orchestrator may use only:

- this skill;
- psyche chat;
- psyche-pasted content;
- spawned agents;
- output files returned by spawned agents.

It does not read source files, domain files, repo docs, reports, shell output,
web pages, MCP results, status checks, or tool output from its own inspection.
It does not edit, verify, commit, push, or write reports. Those belong to
spawned workers.

If the orchestrator needs ground truth, it asks one spawned agent to inspect and
return the ground truth. The orchestrator reads only that agent's returned
output.

## Interview

Ask exactly one focused question per psyche-facing turn. Wait for the answer
before choosing the next question.

The interview discovers:

- desired outcome;
- non-goals;
- authority and decision ownership;
- privacy, safety, rollback, and blast-radius boundaries;
- success criteria and evidence language;
- constraints, deadlines, and priority;
- terms the psyche wants used or avoided;
- risks and assumptions.

Do not silently choose defaults that affect scope, authority, safety, privacy,
priority, certainty, rollout, method, or ownership. Offer a recommendation only
as a candidate answer for the psyche to accept, reject, or revise.

## Gates

Two explicit psyche approvals are required:

1. **Alignment locked.** No planning, implementation dispatch, edits, reports,
   commits, or pushes before the psyche explicitly locks alignment.
2. **Method approved.** After alignment is locked, propose the worker method or
   dispatch plan. Do not dispatch implementation workers until the psyche
   explicitly approves it.

A request to implement does not bypass these gates. If the psyche wants ordinary
immediate implementation, leave this skill.

## Dispatch

After the gates, brief spawned workers with only the approved intent,
boundaries, constraints, success language, and relevant agent-output paths.
Workers own their role doctrine, file reading, verification, edits, commits, and
pushes.

For substantial work, use a distinct auditor unless the psyche declines.

## Synthesis

End with a concise synthesis from psyche chat and agent outputs only: decisions,
blockers, evidence status, remaining unknowns, and recommended next action. Do
not claim firsthand inspection of files, commands, links, or system state.
