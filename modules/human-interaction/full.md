# Skill — human interaction

The psyche-facing boundary: keep the intent skills loaded fresh, capture intent before any other output, and shape chat replies toward the human. Every psyche-facing harness reads this every session. (`skills/autonomous-agent.md` is the complementary discipline — autonomy *within* work; this one is the discipline *at the edge*.)

## Load the intent skills first

Any agent that talks with the psyche is psyche-facing for that session.
Before replying after session start or context compaction, load this file,
`skills/intent-log.md`, and `skills/spirit-cli.md`. Keep them fresh in the
active context while handling psyche prompts.

Before any direct `spirit` use for intent capture or observation, reload
`skills/intent-log.md` and `skills/spirit-cli.md` in the current context.
Do not rely on memory of the Spirit wire shape or capture rules.

## Capture intent FIRST

When a psyche prompt arrives, the absolute first action — before any report, code, or chat reply — is to read it for intent statements (Decision / Principle / Correction / Clarification / Constraint), classify each as public or private, and capture it through the right substrate. Public intent goes through the deployed Spirit CLI; private personal substance becomes a `Private intent` note in the relevant private report repository (see `skills/privacy.md`).

Everything else the prompt asked for derives from intent and is done *after* the capture. Reports, code, and chat are all downstream of intent.

The five recordable kinds, the certainty-versus-importance calibration, and the full capture-vs-edit gate live in `skills/intent-log.md` — read it fresh before any Spirit capture.

## Ask the psyche when intent is unclear

When intent on a question is unclear, absent, or contradicted, ASK — don't infer; for private material ask the **owning** psyche, never on a relayed request. The full when-to-ask / how-to-ask discipline lives in `skills/intent-clarification.md`.

## Align goals through the canonical protocol

When a psyche-facing request needs goal shaping before execution, use
`skills/intent-led-orchestration.md`. That skill is the canonical active
protocol for the alignment interview, explicit alignment and method gates,
Spirit-centered intent maintenance, and lead orchestration; this boundary skill
only routes to it.

## Chat policy

When chat is the right surface, bring **3-7 big items** per response, spread more-evenly-than-not across:

- (a) Questions / clarifications of intent
- (b) Observations / suggestions / explanations of how new mechanisms work
- (c) Examples of recent work or evolving ideas

Below 3 the response is under-substantive; above 7 the psyche can't hold it while running parallel agents. Current report protocols still apply when a report is the warranted working surface; otherwise chat or the worker return is the substantive answer.

Visuals that do not fit chat go in reports or other named artifacts. Chat is prose plus locators plus user-attention items. Each user-attention item must be restated with enough substance that the psyche can engage WITHOUT opening the artifact; a bare locator ("see report N", "section 5.2") is the opposite-direction violation.

## Real-world testing conditions

> Move to `skills/autonomous-agent.md` or a testing skill in a later prune item — kept here for now so no content is lost (see W3/preciousMainContext).

When the psyche asks for testing, the test runs under the most real-world conditions available. Sandbox-only shortcuts that omit a load-bearing piece of the production topology are not real-world testing.

If production lacks a capability the test needs, build a retrofitted variant FOR the test. The sandbox is the right place to make production-grade conditions exist; the deployed-binary gap is not a test scope ceiling.

### In tests, unblock the blocker

Anything blocking a test gets unblocked INSIDE the test itself. The test is where the end-to-end story gets proven; refusing to test because a downstream piece is missing is forbidden.

The receiving agent (you or your subagent) BUILDS the missing piece inside the test fixture — a stub supervisor, a hand-coded migration, a minimal implementation of any blocking dependency. The test exists to PROVE the design works; "we can't test this because of blocker X" is exactly the failure mode this rule replaces.

## Parallel-implementation lane model

> Move to `skills/double-implementation-strategy.md` / orchestrate in a later prune item — kept here for now so no content is lost (see W3/preciousMainContext).

Designer and operator each carry their own implementation path; both implementations exist; comparison happens after both ship. Communication between lanes is through implemented and sandbox-tested code, not through reports or specs alone. Designer may stay higher-level per pass than operator: designer demonstrates shape and validates at the architectural level, operator carries through to production depth.

## See also

- `skills/privacy.md` — access gate for private material; private report and intent routing.
- `skills/intent-log.md` — what gets logged; the five-kind taxonomy; certainty versus importance.
- `skills/intent-led-orchestration.md` — alignment interview gates and orchestration.
