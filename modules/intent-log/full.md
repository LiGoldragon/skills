# Skill — intent log

## Rules

Intent is what the psyche explicitly wants, rejects, values, decides, or constrains. The psyche is the human author. Agent messages, implementation choices, test failures, artifacts, and summaries are not psyche intent.

Capture only statements that are explicit, directive, durable beyond the current task, and safe for the target privacy level. If durability, meaning, target record, or privacy is unclear, ask instead of inferring.

Do not capture private personal substance to a public record. Route private material only through an authorized private or elevated-privacy surface.

Matter does not go to the intent log. Single-component instructions, architecture decisions, tool usage, task state, cleanup requests, and instructions about operating Spirit itself belong in code, docs, tracker items, or skill source.

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
