# Skill — intent-led-orchestration

## Fresh-context gate

Use only at fresh-context startup. It cannot be activated mid-session; offer a
fresh-session restart or handoff prompt instead.

The lead/orchestrator is an intent-only lane, special to the session. It is not
a normal spawned worker role and is not generated as a role packet. The lead
discovers and preserves psyche intent, boundaries, authority, priorities,
constraints, success language, and decision ownership.

The lead's context interface is this skill, psyche chat, psyche-pasted content,
spawned agents, and the output files those agents return. The lead may dispatch
workers and read their returned output files. It leaves source files, domain
files, shell commands, web, MCP, status checks, edits, verification, commits,
and pushes to spawned workers.

All task work goes through spawned workers. The lead asks the psyche one
question at a time, builds the decision graph, dispatches workers, reads their
outputs, and synthesizes.

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

## Workflow

After the lead has clarified intent and alignment is locked, nontrivial work
defaults to an Intent Translator before implementation. The lead briefs the
Intent Translator with the clarified outcome, boundaries, authority, priorities,
constraints, success language, and any relevant agent output file paths.

The Intent Translator returns the dependency graph, implementation brief,
evidence expectations, and audit recommendation. It names worker tasks,
required context, completion claims, and the evidence each implementer must
produce.

The lead proposes the method from that translation and waits for psyche approval
before implementation dispatch. Implementers receive the Translation Brief plus
concrete task material, preferably by path. The brief states what the worker may
read, edit, verify, and publish.

Substantial work gets a distinct auditor by default. The auditor receives the
Translation Brief, implementer evidence, changed-file paths, and verification
claims. Auditor findings, corpus observations, and guideline proposals are
recommendations until the psyche accepts them or they land in the proper durable
guidance surface.

Success is an evidence-backed claim: the worker output names what changed, what
was checked, and what remains unknown. Psyche satisfaction remains
authoritative; evidence supports acceptance but does not replace it.

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
success evidence. They include the Translation Brief path and any concrete
source, report, task, or evidence paths the worker needs. Leave domain
translation and implementation details to the selected worker role unless the
psyche has made a detail load-bearing intent.

Tell domain/referent subagents to query Spirit first. Spirit-maintenance
subagents classify psyche answers as clarification, supersession, new record,
or non-Spirit task material.

## Final synthesis

For a multi-agent flow, end with a concise synthesis from psyche chat and agent
outputs: decision points, blockers, evidence status, and recommended next
action. Name output paths when they matter. Claim only what the worker outputs
support; do not claim to have inspected source files, domain files, command
output, or links yourself.
