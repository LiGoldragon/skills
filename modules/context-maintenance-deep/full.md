# Skill — deep context maintenance

Cross-lane orchestration, lane retirement, and the cross-lane meta-report directory — the heavier maintenance patterns. Everyday single-lane sweeps live in `skills/context-maintenance.md`.

## When this fires

Reach here when a maintenance sweep is broader than one lane:

- Cross-lane sweeps spanning several session lanes in one pass — judged by topic-recency across them.
- Lane retirement, when a drained session lane is being deleted and recorded in the registry.
- Successor sweeps that retire prior maintenance ledgers.

## Cross-lane meta-report directory

A cross-lane sweep lands as **one meta-report directory in the dispatching lane** — never scattered across each swept lane's `reports/`, which would split the maintenance across the very session directories it oversees. The directory mechanics — numbering, `0-frame-and-method.md`, the no-`meta-`-prefix rule, one-unit garbage collection — live in `reporting.md` §"Meta-report directories". This skill adds only what is specific to a cross-lane sweep:

- **Organize topic first, lane second.** Slices are topic aggregations (`1-<topic>.md` under an `<N>-cross-lane-context-maintenance-<date>/` directory), because stale/forward/migrate/keep is judged by topic-recency across lanes. The synthesis (`N-overview.md`) gathers the topic reports into per-lane handoffs.
- A per-lane slice is acceptable only for a narrow single-lane sweep, or when the psyche asks for lane-by-lane output — and even then, rank by topic inside the lane.

### Per-topic sub-report shape

Each topic slice carries:

1. **Topic arc** — one paragraph: the topic and any major era shift.
2. **Current canonical surface** — the newest reports or permanent docs that remain load-bearing.
3. **Stale / forward / migrate / keep bands by lane** — per-report recommendations grouped under the lane that owns the action.
4. **Landing evidence** — for every stale/drop, the successor report or permanent home that absorbed the substance.
5. **Drop ownership / handoff** — "when this lane next does maintenance, the relevant actions are: …".

### Dispatcher authority

The receiving agent applies recommendations within its own lane. **The dispatcher executes only the actions it owns in its own lane, and records — never executes — another lane's drop**, after verifying the landing gate. This single boundary governs the whole pattern: a cross-lane sweep produces handoffs, not cross-lane edits.

### Dispatching sub-agents

A sweep across more than 4–5 lanes or 3 major topics suits parallel sub-agent dispatch; assign agents by topic cluster, and allocate slot numbers + paths up-front (`reporting.md` §"Pre-launch lane allocation"). The everyday agent-dispatch mechanics are in `context-maintenance.md` §"Using agents for the sweep".

### Retired lanes — route by topic, then delete the directory

A retired lane's interesting content does not move into a "main lane" subdirectory — there is no fixed main lane under the session model. Instead, route each piece by the three-fate disposition: durable meaning → a Spirit record (intent); implementable work → a bead in the dependency graph (work); everything else → released (abandon). Live working-artifact substance that another active session still leans on agglomerates by **topic** into that topic's current canonical report. Reports whose only purpose was auditing or summarizing the retired lane's target are stale candidates too; keep or migrate only their independent design rationale. Once routed, the retired lane's whole report directory is **deleted** — git history and the transcript are the archive (see "Retiring a lane" below).

## Successor sweeps retire maintenance ledgers

A context-maintenance meta-report is itself a working artifact. It retires when a newer sweep covers the same lanes/topics, re-ranks the current surface, and re-issues the still-live handoffs. Never keep two live cross-lane sweep directories for the same scope: the newer becomes the active ledger, and the older is dropped by its owning lane once its handoffs are applied or superseded — confirm the newer absorbs the older's live handoffs first (the landing gate from `context-maintenance.md`).

## Retiring a lane

Retiring a lane is gated on context maintenance completing on the lane's leftover memories: the lane does not retire until its reports and beads find their right homes via the three-fate disposition. A cross-lane sweep may flag a lane as a retirement candidate when all its reports are drained — but that is a recommendation, not the retirement. The full methodology:

1. **Triage every report** under `reports/<retiring-lane>/` with the drop/forward/migrate/keep rule (`skills/context-maintenance.md`). Live working-artifact substance forwards into the canonical report on its topic (which may belong to another active session); mature substance inlines into permanent docs (architecture, skills, per-repo `INTENT.md`); implementable work becomes a bead; the rest is released. Nothing load-bearing may remain unrouted.
2. **Triage every bead** tagged with the lane's label:
   - **Close** — done, abandoned, or absorbed elsewhere; close-with-breadcrumb naming the new home.
   - **Reassign** — work continues; relabel it and link it into the dependency graph (`bd dep <blocker> --blocks <blocked>`) so a fresh-context agent can pick it up in order.
   - **Promote to architecture** — a design idea that should live as a "Possible features" entry (`skills/architecture-editor.md`); migrate it, then close the bead.
3. **Resolve pending design decisions** the lane carried — settle now (a Spirit record if the psyche stated durable meaning), abandon, or park as a "Possible future design" entry in the relevant architecture file.
4. **Delete the report directory.** Once everything is routed, `rm -rf reports/<retiring-lane>/`. Git history holds the reports and the session transcript holds the reasoning; the working tree shows only active, undrained lanes.
5. **Append one line to the retired-lane registry** at `protocols/retired-lanes.md` — the single append-only index of retired lanes. The line carries: the lane name, its discipline, the git revision range holding its reports, a transcript pointer, the drain date, and a one-line statement of what the lane decided. This keeps the drained session discoverable for regression and model-behavior forensics without re-growing the working report tree. The orchestrate daemon's live `LanesObserved` registry indexes active lanes; this file indexes retired ones.

Whichever lane the psyche directs handles retired-lane sweeps as standard context maintenance.

## See also

- `skills/context-maintenance.md` — everyday single-lane sweep core.
- `skills/intent-maintenance.md` — Spirit capture sweep, supersession, dedup.
- `skills/reporting.md` — meta-report directory shape and report hygiene.
