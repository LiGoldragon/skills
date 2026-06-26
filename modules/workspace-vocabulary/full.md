# Skill — workspace vocabulary

Canonical glossary for load-bearing workspace terms. Consult before writing ARCH text, reports, code identifiers, or chat that names one of these concepts.

## What this skill is for

Settled vocabulary is binding: new writing uses the canonical form; existing surfaces converge to it as they're touched. This skill collects the load-bearing terms in one place so agents recognise a non-canonical phrasing without scanning the intent log.

Use it when writing or editing any surface that names a listed concept, and when reviewing another agent's output for vocabulary drift. Each entry names its predecessors (the non-canonical forms to converge away from), the scope where the rule binds, and any carve-out where the term means something else.

## Load-bearing terms

### Pocock planning term — `PRD`

Predecessors: `PDR` when referring to Matt Pocock's workflow.

Scope: references to Matt Pocock's `to-prd`, `implement`, and
`grill-with-docs` flow. `PRD` expands to Product Requirements Document. In
primary, the local behavior is a pre-implementation alignment pass, not a
mandatory manually written report: resolve the problem, solution, user-facing
outcomes, implementation decisions, proof or test seams, out-of-scope
boundaries, dependency graph, and agreed vocabulary through an intense
alignment interview before implementation workers start. The exit gates are
explicit: the psyche locks alignment, then approves the implementation method
or dispatch plan.

When writing psyche-facing primary guidance, prefer plain descriptions such as
"alignment pass", "worker-ready brief", or "dependency graph" unless the text
is explicitly comparing to Matt's PRD. Use `PRD` for the source concept; do not
introduce `PDR`.

### Shared domain language

Predecessors: `ubiquitous-language file`, `UBIQUITOUS_LANGUAGE.md`, ad-hoc
synonyms for the same concept.

Scope: agreed terms that reduce miscommunication across psyche, agents, docs,
and code. During grilling or alignment, challenge fuzzy or overloaded terms,
pick one canonical term, name the avoided synonyms, and then use the canonical
term consistently in worker briefs, bead descriptions, architecture text, and
chat.

Storage follows ownership. Spirit holds durable psyche intent, referents,
clarifications, and supersessions. This file holds workspace-wide vocabulary
that future primary agents must use. Repo-specific domain language belongs in
that repo's established guidance surface: usually `skills.md`,
`ARCHITECTURE.md`, `INTENT.md`, or an existing `CONTEXT.md` convention. Create a
new glossary/context file only when the repo has that convention or no existing
durable surface can hold the resolved terms cleanly.

### Version-pair vocabulary — `main` / `next`

Predecessors: `current` / `next`, `current_*` / `next_*` field prefixes.

Scope: every surface naming the two adjacent versions in a handover — ARCH text, reports, contract docs, prose. The active version is `main`; the version being upgraded to is `next`. Rust field names in `persona/src/upgrade.rs` still use `current_*` because the wire contract calls that side "current"; renaming those to `main_*` is operator-side work (see below).

Don't use it when quoting verbatim psyche text, or when citing a code identifier that still uses `current_*` — parenthesize the legacy name: "`current_owner_socket_path` (main owner socket)".

Branch sense (consonant). `next` also names the standard designer development branch — the per-repo branch where breaking changes are developed before an operator integrates them to `main`. The two senses are consonant: the `next` branch is where the `next` version gets built. The `-next` repo suffix (`nota-next`, `schema-next`, `schema-rust-next`) is legacy, from the retired practice of spinning up successor repos for major breaks. Major breaks are branches now; a new repository is only for a genuinely new project. Surviving `-next` repos are canonical-by-default crates never renamed back — treat the suffix as a name, not a license to make more.

See also `skills/feature-development.md` and `skills/double-implementation-strategy.md`.

### Component name — `Persona`

Predecessors: `Persona Engine Manager Daemon`, `engine-manager daemon`, `persona engine` (when referring to the component, not the AI-work scope below).

Scope: the conceptual entity carries the short name "Persona". Concrete surfaces:

| Surface | Canonical form |
|---|---|
| Repo | `persona` (lowercase, at `/git/github.com/LiGoldragon/persona`) |
| Daemon binary | `persona-daemon` |
| CLI binary | `persona` |
| Conceptual entity | Persona |
| Role / relation | engine-management (what the daemon does) |

"Engine-management" names the role Persona plays — the daemon's external-facing concern. "Persona" names the entity that fills the role. Engine-manager-as-noun (referring to the daemon itself) is non-canonical. Use "Persona" or "persona-daemon" depending on whether the prose talks about the conceptual entity or the running binary.

Don't use it for the larger AI-work scope of the stack. "Persona engine" is specifically the AI-work part of the Criome stack — the agent/harness/mind-state surface, not the engine-management daemon. Use the longer phrase only when the AI-work meaning is intended; otherwise default to "Persona". The workspace stack as a whole is "the Criome stack".

When in doubt: is the prose talking about the engine-management daemon (supervises components, owns the engine catalog, drives upgrades) or about the AI-work surface (the agent/mind/harness collaboration an engine hosts)? The first is "Persona"; the second is "Persona engine".

### Engine-management socket axis — `engine_management` (not `supervisor` / `supervision`)

Predecessors: `supervisor`, `supervision_socket_path`, `supervision_socket_mode`, `.supervision.sock` constants, `SupervisionSupervisor`, `signal_persona::supervision` module.

Scope: every surface naming the Persona ↔ supervised-component authority relation — the typed cross-engine management surface where the Persona daemon manages component lifecycles, observes health, drives the spawn envelope, and listens for component readiness. The deployed code already uses the canonical snake_case form in `signal-persona` and `persona/src/`; the rename target is the same form everywhere.

| Where | Predecessor | Canonical |
|---|---|---|
| `signal-persona` module | `supervision::` | `engine_management::` |
| Socket-path identifiers | `supervision_socket_path` | `engine_management_socket_path` |
| Socket-mode identifiers | `supervision_socket_mode` | `engine_management_socket_mode` |
| File-name constants | `.supervision.sock` | `.engine_management.sock` |
| Wire-vocabulary types | `Supervision*` | `EngineManagement*` |
| ARCH prose | `supervision socket` | `engine-management socket` |
| Prose role-noun | `the engine-manager (daemon)` | "Persona" or "the Persona daemon" |

Don't use it when `supervisor` / `supervision` refers to something else: Kameo supervision-tree topology (`EngineSupervisor` inside the persona repo, Kameo parent-child supervision graphs), systemd unit-supervision, or generic operating-system process-supervision. Those are unrelated technical uses and keep their established names. The rename targets only the Persona-specific socket and contract surface — not every occurrence of `supervisor`.

## Remaining operator-side work

Two canonical-vocabulary axes carry Rust-side renames that are operator-scoped:

- `current_*` → `main_*` field renames in `/git/github.com/LiGoldragon/persona/src/upgrade.rs`. Operator chooses whether to land these alongside the next handover-driver code change or as a standalone rename.
- `supervision_*` → `engine_management_*` identifier and constant renames across the `persona` and `signal-persona` source trees. The deployed code already uses `engine_management::` and `engine_management_*` in several places; remaining occurrences (`SupervisionProtocolVersion`, `Supervision*` reply types, `supervision_socket_path` / `.supervision.sock` constants in adjacent crates) follow.

When operator lands these, the ARCH text and reports already carry the canonical vocabulary, so the parenthetical bridges in this skill and in `persona/ARCHITECTURE.md` can retire.

## How to apply

When writing new content, use the canonical form. When editing a surface that uses a predecessor, converge it in the same edit — drift compounds the longer it's left, since every new report and ARCH section written against the old form needs re-edit later. The exception is verbatim quotes from older psyche text or older operator commits: preserve their original wording.

## See also

- `skills/naming.md` — the upstream English-words + no-ancestry rules; this skill is the vocabulary-specific application.
- `skills/intent-log.md` — where settled vocabulary comes from.
- `skills/component-triad.md` §"Vocabulary" — upstream triad-shape vocabulary (component triad, working signal, policy signal, signal types, signal tree) lives there, not here.
