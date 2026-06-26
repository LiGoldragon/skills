# Skill — architecture editor

## What an `ARCHITECTURE.md` is

It describes **what the system IS** at a specific scope — the canonical
reference for shape: components, ownership boundaries, invariants, the typed
contracts between parts. Not a tour, not a tutorial, not a history.

Two scales:

- **Per-repo** — describes that repo's niche: what it owns, what it doesn't,
  the major types and their relationships, the contracts on its boundaries.
- **Meta** — for an ecosystem of related repos, lives in the ecosystem's
  coordination (apex) repo and describes how the niches fit together.

Active component and contract repos are listed in
`protocols/active-repositories.md`. Worked examples:
`repos/criome/ARCHITECTURE.md` (meta) and `repos/signal/ARCHITECTURE.md`
(per-repo contract crate).

**Scope discipline.** When a doc describes a system whose *eventual* form is
larger than what exists today, name the scope in a marker near the top. Bodies
describe what is true today in present tense; the eventual shape is labelled,
not implied.

## Where each kind of statement lives

| Doc | What goes there | Permanent? |
|---|---|---|
| `ARCHITECTURE.md` (meta) | How components fit together: runtime topology, wire vocabulary, state flow across processes, named clusters and boundaries. | Yes |
| `ARCHITECTURE.md` (per-repo) | This repo's role, what it owns and doesn't, code map, invariants, edge contracts. | Yes |
| `skills.md` (per-repo) | How an agent works *in* this repo: what's load-bearing when editing here. | Yes |
| `AGENTS.md` (per-repo) | Thin shim naming the repo's role + carve-outs from the workspace contract. | Yes |
| Reports | Decision records, rationales, audits, syntheses. Working surfaces. | No — ephemeral, retires |

"What the system IS" goes in `ARCHITECTURE.md`. "What an agent should do" goes
in `skills.md`. "Why we chose this" goes in a report — and when that rationale
is load-bearing for understanding the shape, **inline the claim** into the
architecture rather than citing the report.

## Format

Markdown. No required schema beyond these conventions. The structure that has
worked:

```markdown
# <repo> — architecture

*<one-line essence>*

> Status note / read-this-first banner if the file is meta-scope.

## 0 · TL;DR
<2–4 paragraphs: the system in its sharpest form>

## 1 · Components and clusters
<typed map of the components; visual diagram>

## 2 · Wire vocabulary
<the contract types; how processes speak; the contract repos>

## 3 · State and ownership
<who owns what; where each piece of data lives; transaction boundaries>

## 4 · Boundaries
<what this scope owns vs doesn't; neighboring repos>

## 5 · Constraints
<line-by-line obligations, simple enough to become test names>

## 6 · Invariants
<system-wide truths this scope preserves; fewer and broader than constraints>

## 7 · Possible future design
<deferred features, undecided designs, open questions — see "Carrying uncertainty">

## Code map
<per-repo: directory tree with one-line annotations>

## See also
<sibling ARCHITECTURE.md files this one connects to>
```

The TL;DR is the most load-bearing section: a reader who reads only the first
30 lines should come away with the right mental model.

Diagrams are first-class — Mermaid `flowchart`, `sequenceDiagram`,
`stateDiagram-v2`. Quote labels containing parentheses or slashes (see
`reporting.md` §"Mermaid label quoting").

### Constraints are the test seed

Every component architecture has a **Constraints** section: short, direct
sentences naming what the component must do, intentionally simpler and more
numerous than invariants. They read like test names in prose:

- The `mind` CLI accepts exactly one NOTA record.
- The daemon owns `mind.redb`.
- Queries never send write intents.
- A contract crate contains no runtime actors.

Each load-bearing constraint needs an architectural-truth test named after it.
The test can be strange — static source scans, dependency-graph checks, actor
trace witnesses, redb fixture chains, process-boundary probes, compile-fail
guards — all valid if they prove the constraint. The constraint says what must
be true; the test names the observable witness that makes lying hard.

The split: **Constraints** = many concrete obligations, often one test per
line. **Invariants** = few broad truths preserved across all constraints.
**Tests** = constraint-name witnesses proving the architecture path was used.

If a constraint cannot be tested, rewrite it until it names an observable
witness, or move it to a report as unfinished thinking.

## What an `ARCHITECTURE.md` does NOT contain

- **Implementation code.** The type system enforces shape; prose decays. A
  few-line snippet of a type's surface is fine; an implementation block is not
  (see `ESSENCE.md` §"Skeleton-as-design").
- **Decision history.** "We considered X but went with Y" lives in a report.
  The architecture describes Y as *what is*; if rationale is load-bearing,
  inline it.
- **References to reports.** See the next section — load-bearing rule.
- **Implementation scheduling.** *When* something ships belongs in beads or
  coordination notes. The architecture is what the system *is* — or, in
  uncertainty sections, what it *might be*. Not a roadmap.
- **Tour-style narration.** Architecture is reference, not a guided tour.
- **Restatement of workspace skills.** Cite them; don't repeat them.

## Architecture files never reference reports

`ARCHITECTURE.md` files do not cite reports. Reports under `reports/<role>/`
are ephemeral working surfaces that retire, get renumbered, or change as the
design evolves. An `ARCHITECTURE.md` describes *what is*; it must stand without
time-stamped citations into a surface that may be deleted. "See report 161 for
the verb spine" rots the moment 161 is deleted or superseded.

When an architecture needs content that currently lives in a report:

- **Inline the load-bearing claim** — copy the constraint, typed shape,
  invariant, or table into the body. The architecture becomes self-contained.
- **Reference another permanent doc** — a sibling `ARCHITECTURE.md`, a
  `skills.md`, an `ESSENCE.md` section, or code by file path (not a deep URL).
- **Drop the reference** if nothing in the report is load-bearing.

If the design isn't yet settled enough to inline as present-tense "this IS"
prose, the report stays canonical; the architecture is updated only when the
design is ready to be stated that way. This rule has no exception.

## Carrying uncertainty — possible features and undecided designs

Architecture files are not only for cemented decisions. `ARCHITECTURE.md`,
per-repo `INTENT.md`, and workspace skills can carry possible features,
undecided designs, and open questions — provided the uncertainty is named
explicitly, not smuggled into present-tense "this IS" prose.

A **Possible future design** section is a **standard part of every
architecture file**, not an escape hatch added only when uncertainty happens
to exist. Its job is to record future direction *in place* — deferred
features, possible designs, the direction the component is heading — so it is
not lost to chat or a retiring report. The quality bar holds: real deferred
features and named open questions, not idle speculation. Example: a **Restore**
operation for Spirit (re-asserting archived records into the hot store, the
inverse of collect) is slated as possible-future-design rather than built now —
it lives here, framed well enough to be picked up later, until the psyche
decides to build it.

### The shape

A dedicated, clearly-labelled section carrying tentative substance:

```markdown
## Possible features (not decided)

*Items here are under consideration, not committed. Each names the open
question; moves to the cemented body when settled; retires when ruled out.*

- **Feature X**: open question — how do we handle Y? Considered: A, B.
  Status: undecided, blocked on decision Z.
```

Acceptable names: `## Possible features`, `## Open questions`, `## Undecided
boundaries`, `## Future considerations`, `## Under discussion`. Pick one or two
per file; consistency over creativity.

### Disciplines for uncertainty sections

- **Name the certainty explicitly.** A status sentence at the top, or a prefix
  per item ("Considered:", "Possible:", "Undecided:"), prevents a reader from
  mistaking tentative for decided.
- **Name the question, not just the option.** "Open question: how do we handle
  Y? Considered: X, Z" beats "Possible feature: X." The question is the anchor.
- **Keep entries brief.** One paragraph per item. When substance grows past
  that, write the report; the entry collapses to a one-line pointer.
- **Move out when decided.** When a question settles, the substance moves into
  the cemented body (or retires if ruled out). Don't leave settled content in
  an "undecided" section.
- **Reference, don't restate.** While a report is still alive, an uncertainty
  entry may point at it — uncertainty entries can name reports because they're
  explicitly tentative. The moment they're decided, the citation gets inlined
  and the report retires. Cemented claims never cite reports.

### Cemented vs uncertain — keep them visually separate

Cemented sections (Components / Wire vocabulary / State / Boundaries /
Constraints / Invariants) describe what IS. Uncertainty sections sit AFTER the
cemented body, not interleaved — so a reader who stops at the constraints has
read only cemented architecture. This is why §7 sits after invariants, before
the code map.

### Where uncertainty lives by file

| File | Cemented content | Uncertainty headings |
|---|---|---|
| `ARCHITECTURE.md` (meta) | Components, wire vocabulary, cross-component invariants | `## Possible future components` / `## Open questions` |
| `ARCHITECTURE.md` (per-repo) | This repo's components, contracts, invariants | `## Possible features` / `## Undecided boundaries` |
| `<repo>/INTENT.md` | Psyche-stated goals/constraints/principles | `## Possible directions` / `## Open questions` (psyche-derived only) |
| `skills/<name>.md` | Decided discipline | `## Open questions` / `## Under discussion` |

For `<repo>/INTENT.md` the discipline is stricter: every uncertainty entry is
100% backed by a psyche statement (see `repo-intent.md`) — the agent doesn't
invent open questions, just records ones the psyche named without deciding.

## Continuous manifestation discipline

Architectural intent must be manifested into the repo's `ARCHITECTURE.md` **at
all times, not just at the workspace level**. When an architectural decision
lands in Spirit affecting repo R's structural shape — a typed contract change,
a new component, a moved boundary, an invariant — reflect it into R's
`ARCHITECTURE.md` **as part of the work cycle, not as a deferred pass**.

The trigger is wider than "did the psyche specifically address R?" — any intent
whose architectural shape **applies to R** (because R is in the affected stack,
or because the rule binds R's component category) is in scope.

Flow:

1. When you capture an architectural intent through Spirit (`spirit-cli.md`),
   also identify the repos whose architectural shape it affects.
2. For each affected repo, edit its `ARCHITECTURE.md` on a designer feature
   branch in `~/wt/github.com/<owner>/<repo>/<branch>/`.
3. Land the Spirit capture and the `ARCHITECTURE.md` edits in the same work
   cycle — not as a "now / later" pair.

This prevents the failure mode where architectural intent stays in Spirit +
reports but never reaches `ARCHITECTURE.md`: an agent reading the repo's
architecture sees the prior shape and codes to it. The architecture file is the
load-bearing **what the system IS** surface; if it lags the typed-contract
decisions, the rest of the work loses its anchor. The prose-intent companion
(psyche goals / constraints / principles in `INTENT.md`) lives in
`repo-intent.md` §"Continuous manifestation discipline".

## When to edit

Edit `ARCHITECTURE.md` when:

1. **The shape has changed** — a new component, renamed contract, moved
   boundary, different transaction owner. Edit immediately; the architecture is
   a current-shape document and lag costs comprehension.
2. **A reader will be confused** by the current state. If a statement is
   technically right but easy to misread, rewrite.
3. **A new constraint is now load-bearing.** Add it to Constraints and name the
   witness test it implies.
4. **A new invariant is now load-bearing.** Add it to Invariants.
5. **A cross-reference to a neighbor is stale.** Update it; if the neighbor's
   architecture has drifted, surface it (or open a bead for that owner).

Don't edit for historical interest (the path is in commit history); for
speculation with no real open question behind it (genuinely-undecided designs
earn an uncertainty section, pure speculation belongs in conversation or a draft
report); or for a typo in a comment block (just edit, skip the ceremony).

## Editing rules

- **Edit in place; don't fork or version.** The current shape is the
  authoritative shape. Old versions live in commit history.
- **Present tense.** Describe what IS, not what was or will be.
- **Positive framing.** When an option is excluded, state the criterion
  positively ("must be Rust"), not the negative ("Go is excluded"). When a
  direction was wrong, the doc shows the new direction; the wrong one disappears
  (see `ESSENCE.md` §"Positive framing").
- **Cross-reference, don't duplicate** workspace skills, ESSENCE principles, or
  neighboring `ARCHITECTURE.md` files.
- **Commit immediately after a meaningful edit.**

## Retire legacy paths when the working interface exists

Do not keep old design-convenience APIs after the working interface exists. The
stack keeps ONE active working API moving forward and removes legacy/convenience
surfaces rather than maintaining parallel ways to express the same runtime path.

This is the "edit in place; don't fork or version" rule applied to the code
substrate. Two parallel ways to express the same runtime path is the wrong
shape: a reader can't tell which is canonical, an agent picks one without
reason, and the legacy surface accumulates consumers that slow the eventual
cleanup. Consistent with `ESSENCE.md` §"Backward compatibility is not a
constraint" — the system being shaped is not a published API under semantic
versioning.

The pattern: when the working interface lands, the retirement of the legacy
surface is a follow-up commit in the same wave, not a deferred cleanup pass.
The applied test: if someone reading the code today would have to ask "which of
these is the right one to use?" — both shouldn't exist. Pick the working
interface and retire the other in the same wave.

## When to create one

If a repo lacks an `ARCHITECTURE.md` and you've done substantive work in it,
create one. The check: can a fresh agent read it and form the right mental model
of the repo's shape? If yes, the file earns its place. If no — vague, missing
key types, no ownership map — keep iterating.

A thin-but-honest `ARCHITECTURE.md` beats no file. But **don't write one for a
repo you haven't worked in deeply** — a confidently-wrong architecture is worse
than none, because future agents will trust it.

## Meta vs per-repo split

When an ecosystem grows past one repo:

```mermaid
flowchart TB
    meta["meta repo<br/>persona / criome / sema"]
    a["component A"]
    b["component B"]
    c["component C"]

    meta -. ARCHITECTURE.md describes whole .-> all["ecosystem topology"]
    a -. ARCHITECTURE.md describes A's niche .-> sa["A's role + boundaries"]
    b -. ARCHITECTURE.md describes B's niche .-> sb["B's role + boundaries"]
    c -. ARCHITECTURE.md describes C's niche .-> sc["C's role + boundaries"]

    meta -- "imports" --> a
    meta -- "imports" --> b
    meta -- "imports" --> c
```

The meta `ARCHITECTURE.md` describes the runtime topology (which processes
exist; what speaks to what), the wire vocabulary (the contract repo and what it
carries), cross-component invariants (transaction boundaries, store ownership,
schema-version discipline), and the named clusters and how they map to repos.

The per-repo `ARCHITECTURE.md` describes this repo's role in the ecosystem, the
major types it owns, the contracts at its boundaries (what it imports from the
contract crate; what it exposes), its code map, and repo-specific invariants.

The split keeps the ecosystem architecture from growing into one huge file and
keeps per-repo files from repeating ecosystem-wide context. Each scope carries
the appropriate amount of detail.

## See also

- `skill-editor.md` — parallel skill for `skills.md` files; same conventions
  for cross-references, scope, and the no-report-references rule.
- `reporting.md` — when to write a report versus update an architecture;
  permanent homes for each report shape.
- `ESSENCE.md` §"Skeleton-as-design" — why architecture stays
  prose-plus-visuals, not implementation blocks.
