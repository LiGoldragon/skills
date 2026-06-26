# Skill — context maintenance

## Context maintenance is the session-drain discipline

A lane is a throwaway work-session named for its intent; its
reports live in a session directory `reports/<lane>/`. Context
maintenance is what a lane does as it **drains at close**: every
idea the session produced routes to exactly one of three fates, and
once drained the lane retires — its report directory is deleted and
one line is appended to the retired-lane registry. The everyday
sweeps below (agglomeration, topic-recency ranking) are the same
discipline applied mid-life, before a compaction or a soft-cap, but
the close-of-session drain is the spine.

## Two surfaces, one discipline

A report is just context saved to disk. Both surfaces decay
together and need the same maintenance, usually in the same pass:

- **Reports on disk** under `reports/<lane>/` — the session
  directory. Working artifacts whose substance matures upward to
  permanent docs (skills, architecture, code) or retires when done.
- **Context in the live conversation**. The session's working
  memory; the unsaved part is lost on compaction or clear unless
  it has migrated to disk.

The rule for both: keep load-bearing substance, move it to its
right permanent home, retire what's done. Maintaining only one is
half a pass.

## The three-fate disposition

When a session drains, every idea it produced — every report, every
open thread, every half-formed thought in the live context — routes
to exactly one of three fates:

| Fate | Where it goes |
|---|---|
| **Intent** | A durable meaning the psyche stated — captured through the Spirit CLI as a Decision / Principle / Correction / Clarification / Constraint. Run the Spirit gate; an edit to an existing record beats a sibling. |
| **Work** | Something still implementable — a bead, linked into the dependency graph (`bd dep <blocker> --blocks <blocked>`) so a fresh-context agent can pick it up in order. |
| **Abandon** | Already landed, stale, or wrong — let it go. Git history and the session transcript preserve it; nothing needs to stay in the working tree to remember it. |

A report that survives the drain does so only because its substance
hasn't yet reached its fate — it is still a working artifact for a
live topic. The drain is complete when nothing is left that isn't
either captured (intent), tracked (work), or released (abandon).

## The goal — fewer reports, same information

A mid-life maintenance pass **reduces the number of reports without
losing information**. The primary move is **agglomeration**: take
the several reports on one topic, merge their un-contradicted,
un-superseded substance into ONE better report on that topic, then
delete the merged sources — the new report is the landing witness.
Agglomerate by **topic, not by lane**: one topic's reports across
all lanes collapse into one report on it.

### The Refresh variant

A report that rewrites or agglomerates prior reports carries the
`Refresh` variant tag: `<N>-Refresh-<topic>-<date>.md`. One Refresh
MAY merge several sources. After it lands, the source reports are
deleted (git history preserves them).

Test for agglomerate-and-Refresh vs migrate-then-drop: substance
still working-artifact-shaped (a topic's design state, open
questions, the arc of decisions) agglomerates into a Refresh
report; substance that is mature or leaned-on doesn't belong in a
report at all — it migrates to the permanent layer.

## When to invoke

- **Session-drain trigger (the spine).** A session lane is closing.
  Run the three-fate disposition over everything the session
  produced, then retire the lane (delete its report directory,
  append one line to the retired-lane registry). Favor a fresh
  session over endless compaction — so most sessions end in a drain
  rather than carrying on degraded.
- **Compaction trigger.** A continuing session's context window is
  near its limit, or the user is about to clear/compact. Sweep
  before the loss event so the live working memory reaches disk.
- **Report soft-cap trigger.** A lane's `reports/` session
  directory crosses the 12-report soft cap (see `reporting.md`).
  Sweep older reports to migrate substance and land back under cap.
- **Explicit user direction.** "Do a context maintenance",
  "do a handover", "drain the lane", or similar.
- **Lane retirement.** A retiring lane's leftover memories must
  first be triaged via this skill — reports under
  `reports/<retiring-lane>/` and beads tagged with the lane label.
  Retirement is gated on that triage completing; only then is the
  report directory deleted and the registry line appended. See
  `context-maintenance-deep.md` for the methodology.

Triggers often coincide; treat them as one pass.

## Method

### 1 · Inventory

List the reports for each relevant lane (`ls
~/primary/reports/<lane>/`). For context, review the conversation's
themes — what was worked on, decided, left open, surfaced. Don't
dump everything; categorize.

### 2 · Topic-recency ranking (cross-lane)

Rank **by topic, across all lanes** — a topic like "schema-derived
nota stack" threads through operator implementation reports,
designer design reports, audits, deploy notes. Pulling them
together shows the topic's whole arc.

1. **Gather all reports on the topic across all lanes.**
2. **Recency-rank within the topic.** Newest at top; date is in
   the filename suffix or metadata header, commit history is the
   tiebreaker.
3. **Name the supersession spine.** Identify the current canonical
   surface, any permanent landings, any old-era/new-era boundary.
   The spine is the evidence that lets old reports retire without
   losing substance.
4. **Flag what's stale.** A report is stale when a newer report on
   the same topic supersedes it AND the older substance is already
   absorbed in the newer report or a permanent doc — *or* when a
   newer Spirit capture reframes its topic (recent intent need not
   land in a successor report to make older framing obsolete).
   Not-yet-stale older reports — substance still load-bearing,
   design alternatives a newer report inherits, decision rationale
   permanent docs don't carry — keep, forward, or migrate.
5. **Recent intent prevails.** When older content conflicts with
   newer intent (newer Spirit capture, newer report, recently
   landed code), the newer is canonical. The older stays only if it
   carries substance the newer doesn't — design alternatives the
   newer chose between, omitted rationale, skipped intermediate
   insight. Spirit captures are the highest-priority recency
   signal: a Maximum-magnitude reframe supersedes the entire prior
   framing on that topic, including reports canonical the day before.
6. **Spirit capture sweep.** Alongside the report sweep, audit Spirit
   captures with the production query shape. Start broad with
   `spirit "(PublicRecords (Any None))"`, then retrieve the stash with
   `LookupStash`. Narrow with the full eight-field query when needed,
   e.g. `spirit "(Observe ((Full [(Information Documentation)]) Any
   (ContainsText [context maintenance]) Any None (Exact Zero)
   (AtLeastCertainty Minimum) Any))"`. Multi-agent sessions accumulate
   near-duplicate captures when each agent records on a forwarded prompt
   without first checking what the originally-addressed agent captured.
   Earlier capture wins; remove duplicates per `intent-maintenance.md`.

Reports without a topic peer get the same treatment on a
single-lane timeline.

**Staleness has a landing gate.** A report is droppable only after
its load-bearing substance has landed in a successor report or a
permanent doc. If the topic has moved on but the landing is not
verified, the action is **Forward** or **Migrate**, not Drop.
During major era shifts: first identify the new canonical landing,
then retire the older pile with that landing as evidence. Older
reports from a prior era can retire as a group — but only
report-by-report after each stale flag names the surface that
absorbs it. Bulk retirement without a landing witness is context loss.

### 2a · Per item, decide

The three-fate disposition is the *outcome* of a drain; these four
actions are the *moves* that reach it. Forward and Migrate land
substance so it can be safely released (abandon); the part of a
report that is implementable work becomes a bead (work); a durable
psyche meaning surfaced along the way becomes a Spirit record
(intent). Drop is the release once a fate is secured.

Pick one of four actions per report or context theme:

| Action | When |
|---|---|
| **Forward** | Substance still load-bearing as a working artifact. Roll it into a successor report or extend an existing one; retire the predecessor. For cross-lane forward-then-drop, the receiving lane confirms absorption and the source lane owns deletion. |
| **Migrate** | Substance mature enough to be permanent. Inline it into a skill, `ARCHITECTURE.md`, `ESSENCE.md`, or code. Retire the source. |
| **Keep** | Substance load-bearing on its own with no permanent home yet. Rare — foundational decisions still searching for final shape. Pending psyche-review items stay Keep/Escalate until resolved, abandoned, or parked as uncertainty in a permanent doc. |
| **Drop** | Substance stale, addressed, superseded, or already captured elsewhere, with both superseder and landing named. Delete the report. If the proof pair is missing, Forward or Migrate instead. |

When the action is uncertain because the agent cannot tell the
psyche's intent — especially Keep vs Drop, or whether a stale-looking
item should be abandoned — ask the psyche a focused question before
deciding. For fresh-context goal shaping or orchestration, use
`intent-led-orchestration.md`; do not restate that skill here.

Heuristics:

- **Audit reports retire with their audited target** unless the
  audit holds independent design rationale or a reusable pattern
  that must migrate.
- **Deploy-event logs, refresh reports, and orientation handoffs
  retire as blocks** once live state is the baseline and durable
  state lives in permanent docs, runbooks, code, or current reports.
- **Pending psyche-review flags are not stale merely for being
  old.** Keep and surface them until the psyche resolves them or an
  agent parks them in a permanent uncertainty section.

Context-item destinations:

| Context type | Destination |
|---|---|
| Decisions made in conversation | The permanent doc the decision lives in (ARCH, skill, `ESSENCE.md`). |
| Half-formed insights, possible patterns | A short `note:` in the right report or skill, or a sentence in a handover. Don't lose them; don't over-format them. |
| Process / workflow reflections | A skill if generalizable, a session-summary if local. |
| In-flight work pending pickup | The successor task / `bd` item / report naming what's next. |
| Already-on-disk content | Drop — already saved. |

### 3 · Distribute

Land substance in its right home, **not** a catchall handover dump.
Preferred order:

1. **Existing reports on the same topic** — extend them in place.
   When a report contains stale illustrative code, diagrams, or
   recommendations a newer implementation invalidated, **rewrite
   that section in place** or mark it retired inside the report. A
   later synthesis report is not enough; stale examples keep
   teaching the old pattern when search lands on the older file.
2. **Permanent docs** — inline as the rule, constraint, or
   invariant they actually are. Permanent docs are where rules
   live, and they never cite the report that produced them (see
   `skill-editor.md`, `architecture-editor.md`).
3. **A new rollover/handover report** — last resort only.
   Substance that fits no existing home and is too unsettled to be
   permanent. A working artifact for the next session, not an archive.

### 3a · Migrate competing-alternatives substance, then retire

A report carrying **competing design alternatives** — multiple
options sketched, one chosen, others rejected — is load-bearing as
design rationale until its substance migrates. Closed reports are
not kept merely for rationale or history. Both the chosen design
and the competing-alternatives reasoning migrate to durable
surfaces; then the report retires.

Migration targets:

- **Chosen design** → architecture file (`ARCHITECTURE.md`) or skill.
- **Competing alternatives + the selecting reasoning** → an
  architecture decision record (when durably worth knowing), or
  Spirit intent records (Decision / Clarification with reasoning
  inline), or both.
- **Empirical evidence (benchmarks, prototypes, witnesses)** → the
  git history carries the prototype code; cite commit IDs from the
  architecture file rather than keeping a report just to point at them.

Once migrations land, **retire the report** (`rm`). Don't leave it
as an archive with duplicate rationale. Keeping live patterns close
to the code or contracts they govern beats a status-banner archive
that preserves contradictions and noises up search.

Signal that a report needs this explicit migration: it enumerates
two or more designs (Design A/B/C, Option 1/2) and chose one.
Standard single-shape design reports migrate cleanly — the chosen
shape lands in an architecture file or skill and the report retires.

### 3b · Manifest leaned-on design into architecture — prefer constraints

When agglomerating, mature or leaned-on substance belongs in the
permanent layer, not a report. Two rules sharpen where it lands:

- **Architecture carries leaned-on design even without explicit
  intent.** When the project's forward direction implies a design
  has been accepted or leaned on for now, manifest it into the
  repo's `ARCHITECTURE.md`. The architecture IS the design layer; a
  leaned-on direction belongs there without waiting for an intent
  record. (The intent layer still requires an actual psyche
  statement — never infer intent — but the architecture layer does
  not.)
- **Prefer constraints.** Constraints are among the most important
  architecture content: a stated constraint lets us write a test
  that verifies it. When manifesting design into architecture,
  express it as a **constraint** wherever possible, and pair it
  with a constraint-witness test that proves the intended path.
  Design as prose teaches; design as a constraint teaches AND
  becomes a test.

### 4 · Small thoughts are OK

A one-sentence side note landed in the right place beats losing the
thought. Tags like `note:`, `possibly useful:`, or `undecided:`
make a thought discoverable without committing the workspace to act
on it. A line like `note: agents asked about X three times this
week — might warrant a skill if it recurs` is better than no
record. Discovery later is the value, not formality now. Don't
over-engineer: a note-line is a note-line, not a numbered section.

## The rollover / handover report (when one is needed)

If genuinely needed, it lives at
`reports/<lane>/<N>-handover-<date>.md` (or `<N>-rollover-…`):

- **What landed** — committed and pushed; one line each.
- **What's open** — discussed but not yet resolved.
- **Side notes** — small thoughts worth keeping, each marked
  `note:` / `possibly useful:` / `undecided:`.
- **Next-session targets** — concrete pickup points.

It retires once its substance migrates to a permanent home or the
next-session work absorbs it; it follows standard forwarding hygiene.

## Using agents for the sweep

A pass over many older reports suits parallel agent dispatch — the
orchestrator needn't read every report into its own context.

- Inventory and topic-cluster first; don't deep-read hundreds of
  reports before you know which topic arcs matter.
- Deep-read stale candidates and their proposed successors;
  skim obvious non-candidates.
- Give each agent a bounded slice (a topic cluster, a lane within a
  topic, a small batch) plus the drop/forward/migrate/keep rule.
- Each agent reads the report, checks surrounding permanent docs
  (does this already live in ARCH or a skill? has it been
  superseded?), then proposes the action.
- Prefer topic-cluster agents over lane-only agents for large
  cross-lane sweeps; the stale judgment is topic-recency across
  lanes.
- Review proposals; execute migrations; retire reports.

The orchestrator applies decisions, doesn't re-read every report.
Agents recommend; the dispatcher decides and applies only the
actions it owns. For context-only substance (live conversation, not
disk), the orchestrator sweeps itself — agents can't see the
conversation's working memory.

Cross-lane sweeps, lane retirement, and the meta-report-directory
pattern live in `context-maintenance-deep.md`. Reach for it when
the sweep spans multiple lanes, multiple topic arcs, or a lane is
being retired.

## Anti-patterns

- **Dumping all context into a "handover" report.** The handover is
  a fallback, not a default. Most substance has a better home; a
  dumping-ground handover decays as fast as the context it replaced.
- **Keeping reports indefinitely "because they might be useful."**
  Git log preserves history; the filesystem holds only what's
  actively load-bearing. Even foundational decisions and incident
  lessons get absorbed into permanent docs.
- **Preserving content the intent has reframed.** A Spirit capture
  or permanent-doc change that reframes a topic supersedes the
  older report's framing even with no successor report. Migrate the
  still-live substance and let the reframed parts drop; waiting for
  a successor before retiring is a smell — recent intent IS the
  supersession evidence.
- **Leaving stale examples alive in older reports.** If a report
  still shows old code as "current", "implemented", or "canonical",
  the pass is incomplete even if a newer report corrects it. Search
  and agent recall find the stale example. Rewrite it to the
  current shape or replace it with a supersession note naming the
  current implementation.
- **Retiring a report whose substance hasn't migrated.** Confirm
  the load-bearing parts are captured elsewhere first, then drop.
- **Treating context and reports as different disciplines.** Both
  are working surfaces; both follow the same forward/migrate/keep/
  drop rule. A pass over only one is half a pass.
- **Keeping successor-superseded ledgers or deploy-event logs.**
  Once a newer sweep reissues the live handoffs, or a live system
  state becomes the baseline, the older chain is stale unless it
  still carries unresolved substance.
- **Over-formatting small thoughts.** The shape matches the
  substance; don't promote a half-formed observation to a section.

## See also

- `context-maintenance-deep.md` — cross-lane meta-report directory,
  successor sweeps, lane retirement.
- `intent-maintenance.md` — Spirit capture sweep, dedup, supersession.
- `intent-led-orchestration.md` — the question rhythm for uncertain
  abandon/keep/migrate/forward decisions.
