# Skill — intent log

## What intent is

Intent is the rare, orienting will of the psyche — what he steers toward (an aim), holds as worth (a value), or fundamentally believes. It is not a decision, default, wish, or rule; those are matter. Intent has a magnetic, unbending quality: the psyche holds it even against his own convenience, and it bends many downstream choices toward it like a North Star. It is often hard to fully verbalize. Capture is the exception, not the reflex; when unsure, it is not intent — ask, do not infer.

The psyche is the human author. Agent messages, implementation choices, test failures, artifacts, and summaries are never psyche intent.

Do not capture private personal substance to a public record. Route private material only through an authorized private or elevated-privacy surface.

## Discrimination test

Capture as intent only when all five hold. Any miss makes it matter — route it to code, docs, skill source, or a tracker item.

1. Aim, value, or belief — not a how, default, mechanism, or rule.
2. Unbending — the psyche would hold it against cost or convenience; for the spirit, not for profit.
3. Orienting — it bends a whole class of future decisions, not one local case.
4. Its "why" bottoms out in a value — what the psyche wants, not an engineering or efficiency tradeoff.
5. From the psyche and felt — not agent-synthesized to close a loop.

## Do not be fooled

These halos read as intent but are matter:

- Rule-grammar — must, never, always.
- A "why" that is only engineering or efficiency justification.
- Vivid or eloquent phrasing.
- A sensible one-off default.
- Agent-operation or Spirit-operation procedure.

Matter never enters the intent log. Single-component instructions, architecture decisions, tool usage, task state, cleanup requests, and instructions about operating Spirit itself belong in code, docs, tracker items, or skill source.

Worked example: "new repos default to public" fails gate 1 (a default), gate 2 (convenience), gate 3 (one local case), and gate 4 (an operational why). It is matter — it belongs in repo, docs, or skill source, not in Spirit.

## Classification

Use the deployed intent kinds:

- `Decision`: the psyche chooses a direction.
- `Principle`: a durable rule or value.
- `Correction`: a prior statement or record is wrong.
- `Clarification`: meaning is narrowed or attached to an existing target.
- `Constraint`: a hard boundary on future work.

Short affirmations count only with their antecedent. Record the question or context that makes the answer meaningful.

## Before writing

Search the relevant domain and referents first. Most apparent new records are duplicates, clarifications, supersessions, or matter.

Prefer maintaining the existing record over creating a neighbor when the psyche is clarifying, correcting, retiring, or superseding it.

Choose certainty and importance from the statement's strength and blast radius. Do not elevate because the topic feels important to the agent.

Populate referents for named technologies, repos, components, people, records, or topics so later lookup and duplicate checks work.

## Writing

Use the Spirit CLI. A record request carries a positional `Entry` and `Justification`. The description is clarified agent prose; testimony is verbatim psyche wording with optional antecedent context.

Do not paraphrase testimony. Do not omit required positional fields. Use the current wire shape from the Spirit CLI guidance when exact syntax matters.

If Spirit is unavailable and capture is required, stop with a blocker. There is no legacy file fallback.

## Manifestation

Capture is incomplete until affected guidance surfaces reflect the settled intent. Manifest repo-specific intent into repo guidance, architecture intent into architecture, reusable agent discipline into skills, and workspace-level intent into the workspace intent surface.

Manifest only what the psyche stated. Keep analysis and rationale in the owning work artifact or commit, not in the intent record.

## Citation

When a durable file depends on a record, cite the record identifier and summarize the relevant meaning. Do not make the citation a substitute for readable guidance.
