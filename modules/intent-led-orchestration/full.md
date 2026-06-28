# Skill — intent-led-orchestration

## Fresh-context gate

Use only at fresh-context startup. It cannot be activated mid-session; offer a
fresh-session restart or handoff prompt instead.

After reading this skill, the lead uses no tools: no shell, file reads, web,
MCP, status checks, edits, verification, commits, or pushes. The lead's only
inputs are this skill, psyche chat, and final returns from subagents. Links in
subagent returns are locators for future workers, not lead-readable context.

All task work goes through subagents. The lead asks the psyche one question at
a time, builds the decision graph, dispatches subagents, waits for returns, and
synthesizes.

## Alignment interview

Intent-led orchestration starts with an intense alignment interview. This is a
serious interview, not a quick clarification pass. For nontrivial work, one or
two questions followed by planning or implementation is a protocol failure.

Ask multiple rounds, but ask exactly one focused question per psyche-facing
turn. Do not batch several questions into one prompt. Choose the highest-leverage
next uncertainty, ask it, wait for the psyche's answer, then choose the next
question. Across the interview, pressure test:

- desired outcome and user-facing behavior;
- non-goals and out-of-scope boundaries;
- authority, privacy, safety, rollback, and decision ownership;
- success criteria, proof, tests, and acceptance language;
- constraints, deadlines, cost sensitivity, and blast radius;
- shared terms, avoided synonyms, and contested vocabulary;
- risks, failure modes, reversibility, and assumptions;
- how the implementation method should be chosen.

Do not silently choose defaults that affect authority, priority, scope, safety,
privacy, certainty, importance, rollout, method, or decision ownership. Offer
recommendations only as candidate answers for the psyche to accept, reject, or
revise.

The lead is not the domain specialist. Keep the interview focused on what the
psyche wants, what the psyche does not want, and the constraints that bound the
work. Do not design specialist implementation details from the lead lane unless
the psyche is explicitly deciding them as intent.

## Grounding

The lead must not ask domain, design, history, repository, schema, or
architecture questions from an ungrounded paraphrase. If good questioning
depends on discoverable context, the first move is exactly one lightweight
read-only subject-understanding subagent.

That subagent returns current ground truth, workspace terms, contested
vocabulary, a decision-graph sketch, and the single best next psyche question. It
must not edit files, write reports, commit, push, or prepare implementation
briefs.

Use more than one initial subagent only when the psyche explicitly asks for
parallel exploration or the questions are truly independent and bounded.

## Gates

Two explicit gates control the work:

1. **Alignment locked.** Before implementation planning, sequencing,
   implementation-subagent dispatch, edits, reports, commits, or pushes, the
   psyche must explicitly lock alignment or use equivalent clear language.
2. **Method approved.** After alignment is locked, the lead proposes the method
   or dispatch plan and waits for explicit psyche approval. Until then, no
   implementation subagent, edit, report, commit, or push is authorized.

A clear directive to implement is not enough to bypass these gates while inside
intent-led orchestration. If the psyche wants immediate implementation, leave
this protocol and use ordinary task rules.

## PRD and language

For Matt Pocock-style `grill-with-docs`, emulate the behavior, not the report
artifact: grill intent, settle vocabulary, then brief workers only after the
gates. `PRD` means Product Requirements Document; do not call it `PDR`.

Maintain ubiquitous language during the interview. Challenge fuzzy or
conflicting terms, propose canonical terms and avoided synonyms, and carry
agreed vocabulary into worker briefs. Spirit is the durable home for psyche
intent, referents, clarifications, and supersessions. Workspace-wide settled
terms belong in `skills/workspace-vocabulary.md`.

Chat and harness output are the normal transient artifact. Write a report only
when the psyche explicitly wants one or a fresh-context pickup artifact is
actually warranted.

## Dispatch

Every subagent brief states task, authority, working directory, dependency
position, allowed sources, return shape, claim/worktree expectations,
verification expectations, and commit/push policy, then includes:

```text
Read `AGENTS.md` and `skills/skills.nota`; select any additional triggered
skills; then follow this brief's worker instructions for lane choice,
orchestration claims, worktree handling, verification, return shape, and
commit/push policy.
```

Before both gates, briefs must say `read-only` and forbid implementation,
edits, reports, commits, and pushes. Implementation briefs are allowed only
after method approval and must name the approved method.

Implementation briefs state intent, boundaries, authority, dependencies, and
success evidence. Leave domain translation and implementation details to the
subagent's selected skills unless the psyche has made a detail load-bearing
intent.

Tell domain/referent subagents to query Spirit first. Spirit-maintenance
subagents classify psyche answers as clarification, supersession, new record,
or non-Spirit task material.

## Final synthesis

Synthesize subagent returns and psyche chat. Do not claim to have inspected
files, reports, command output, or links yourself.
