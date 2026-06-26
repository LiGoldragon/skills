# Skill — beads

BEADS is the workspace's short-tracked-item store, exposed through the
`bd` CLI. It is **transitional**: the destination is Persona's typed
messaging fabric. Don't deepen the BEADS investment and don't bridge to
Persona; use it for what it's good at today and design new shapes
assuming it goes away.

## Never a bare bead id to the psyche

A bead id like `primary-l89s` is a database handle. The psyche cannot
decode it in their head and has no `bd` to query. When you mention a
bead to the human — chat or report — lead with what it IS and keep the
id to a quiet trailing reference, or drop it from chat entirely. Say
"the runner-extraction work," not "`primary-l89s`". The id exists so
*agents* can find the bead; alone it is noise to the human, and a chat
reply full of bare ids reads as if addressed to a peer machine. Same
for Spirit record numbers. See `reporting.md`.

## When to file a bead

A bead is the right home when **all** of:

1. **It's a discrete unit of work** — has a definition of done; will be
   resolved or explicitly deferred, not "ongoing forever."
2. **It needs cross-session memory** — a chat note or report would be
   lost; the work spans more than one session.
3. **It's not better-tracked elsewhere** — not a code change (file an
   issue or just edit), not a discipline (write a skill), not a design
   decision (write a designer report).

Fits: *"Migrate chroma to current nota-codec API"* — discrete,
concrete, spans sessions, closes when shipped.

## When NOT to file a bead

**Durable-backlog beads.** *"Every X should have Y, incrementally"* is
a discipline statement, not a task — it never closes as one unit and
sits forever as a P2 that doesn't move. Fix: land the rule in the right
skill, and close the bead with a note pointing at the rule. If
visibility into the gap is the value (which repos haven't done X yet?),
that's a workspace doc or a CI check.

**Design questions.** *"Figure out X"* without a definition of done is a
design question; its home is a designer report. Acceptable bead form:
*"Land designer report on X"* — discrete, closes when the report lands.
Not: *"Decide what X should be."*

**Ongoing concerns.** *"Monitor build performance"*, *"keep an eye on
the chroma daemon"* — these are alerting (write the alert), monitoring
(write the dashboard), or noise (don't track).

**Reminders for a small fix.** A bead is heavyweight relative to *"fix
this stale comment next time you're in the file."* If it's a one-line
edit and you're in the file, fix it. If it's forgettable but trivial,
leave a `TODO` next to the code.

## Beads are not ownership locks

Any agent may create, update, comment on, or close any BEADS task at any
time; never claim `.beads/`. The corollary: *"someone filed a bead"*
does not mean *"someone is going to do this work."* Beads are
queue-shaped tracking, not assignment. If no agent picks it up, the bead
sits open until pruned.

## Taking on a bead — the task-lock bridge

When you start work on a bead, claim it through the orchestration
protocol so other agents see the work is in flight. Task locks use typed task tokens:

```sh
orchestrate "(Claim (system-operator [(Task primary-f99)] [chroma migration]))"
# … do the work …
orchestrate "(Release system-operator)"
bd close primary-f99 -r "<closing note>"
```

The daemon enforces exact-match overlap across roles: a second role claiming
`(Task primary-f99)` is rejected.

This bridges two layers BEADS alone doesn't span: BEADS lifecycle
(filed/open/closed — durable, visible via `bd list`) and orchestration
locks (claim/release — in-flight coordination on this machine, visible
via `orchestrate "(Observe Roles)"`). A bead in *open* state doesn't tell
other agents someone is working on it right now; the task lock does.
Without it, two agents race the same bead — each does the work, one push
lands, the other discovers stale commits.

When done, release the lock and close the bead in the same flow. Don't
leave a stale lock after the bead closes; don't close the bead while
still holding the lock.

The same syntax extends to non-BEADS work: `'[pr:42]'` to coordinate
review of a PR, `'[draft:role-redesign]'` for a draft report not yet
filed. The helper treats brackets as exact-match identifiers; projecting
the token to the underlying artifact is the agent's responsibility.

### Beads as session anchors

For any task larger than a tiny one-step edit, ensure a bead names the
goal before the work sprawls. If one exists, claim it with a task lock.
If none exists and the work could survive a context compaction or a
handoff, create one. At session end, read the bead again: if its
definition of done is satisfied, close it; if not, leave the next action
or blocker in the bead before releasing the lock. Don't rely on chat
history or harness memory to carry that state.

## Feature beads carry their branch name

A `feature` bead represents work that lives on a **non-main branch** for
the feature arc — typically spanning more than one commit, often across
multiple repos and agents. `task` beads land directly on main; feature
beads name the parallel branch where the work happens.

The bead description declares the branch name explicitly, near the top:

```text
Branch: horizon-re-engineering
Repos:  horizon-rs, lojix, signal-lojix, CriomOS, CriomOS-home, goldragon
```

For multi-repo features, name every repo whose branch carries the work —
every repo uses the same branch name so any agent picking up the bead
lands on the right surface in each.

**Why:** without an explicit branch name, agents picking up the same
bead at different times each create a fresh branch with a slightly
different name (`feature/horizon`, `horizon-refactor`, …), producing
parallel reimplementations to reconcile or throw away. The bead is the
rendezvous; the branch name makes it concrete on the file system.

**How:**

- When filing a `feature` bead, name the branch before any agent picks
  it up. If unknown, say so: `Branch: TBD — first agent to claim picks
  the name and updates this bead`.
- When picking up a `feature` bead, find the declared branch; if
  missing, comment a branch name before starting so the next agent sees
  what you chose.
- Sub-task beads blocked-by a feature bead inherit the parent's branch
  unless their scope is genuinely narrower.
- Branch names are bare descriptive names (`horizon-re-engineering`),
  not `push-` prefixed — `push-` is for short-lived review-cycle
  bookmarks (`jj.md`); long-lived feature branches use the bare form.
- When the feature lands and merges, close the bead with a note pointing
  at the merge commit, and delete the merged branch in every repo
  (`jj.md`).

`task` beads don't need this — they land on main as small commits with
no parallel branch life.

## When to close a bead

Every closed bead's `-r` reason is the durable record of *why this isn't
tracked anymore*. A future agent finding the id in old git history or a
stale report reads the closing note and knows whether to revive (rare),
reopen (rare), or move on (almost always). The closing note is the
bead's small designer report — name the path forward, point at where the
substance lives now, not just *"done."*

**Shipped.** Close with a note pointing at the canonical home (the
commit, the skill change, the `ARCHITECTURE.md` section):

```sh
bd close primary-8b6 -r "Shipped via chroma daemon (replaces darkman + \
nightshift). See chroma repo HEAD and skills/system-operator.md §'Chroma daemon'."
```

**Superseded.** When a design change renders a bead moot (e.g. a
migration for a derive that no longer exists), close with a note naming
the supersession.

**Duplicate — preserve information from both.** When two beads cover the
same work, closing one as a duplicate must preserve all information from
the closed bead. Competing design ideas in particular are kept rather
than collapsed: agents working those fields compare and essay the
alternatives, and premature collapse destroys that comparison surface.
The closing note absorbs every load-bearing field the closed bead
carried — design substance, alternative approaches, blocker analysis. If
the surviving bead doesn't already carry that content, update its
description before closing the duplicate.

```sh
bd close primary-XYZ -r "Duplicate of primary-ABC (which now carries the \
alternative-approach analysis from this bead). All design substance preserved on primary-ABC."
```

**Reformulated as a discipline.** For a durable-backlog bead, close with
a pointer to where the discipline now lives.

**Won't ship.** When a bead is genuinely abandoned — wrong direction,
not going to happen, cost outweighs benefit — close with a note naming
why. Don't leave zombies open.

## Stale internal references in bead descriptions

Bead descriptions decay the same way reports do — a bead filed against
an old report-number or an old crate name (`NexusVerb`) names something
that no longer exists. Don't fight to keep descriptions current. Two
acceptable approaches:

1. **Description as timestamp** — what was true when filed; edit only
   when actively misleading future agents (rare).
2. **Close + new bead** — when the premise has moved enough that the
   description doesn't survive, close with a forwarding note and file a
   new bead carrying current context.

Default: option 1 + close-as-resolution. Don't accumulate edits trying
to keep descriptions fresh; let the canonical home (skill, report, code)
carry current substance.

## The `bd` CLI shape

| Command | Use |
|---|---|
| `bd list --status open` | Workspace queue |
| `bd show <id>` | Read a single bead's full description + status |
| `bd create "title" -t task -p <P> -d "description"` | File a new bead |
| `bd close <id> -r "<closing note>"` | Close with reason |
| `bd dep <a> --blocks <b>` | a blocks b |
| `bd dep remove <blocker> <blocked>` | undo |

Priorities (`-p`): `1` (urgent), `2` (normal), `3` (deferred). Types
(`-t`): `task` (default), or other types the project defines. For the
full reference and project conventions, see `lore/bd/basic-usage.md` if
it exists, or `bd help <command>`.

## Periodic audit

A workspace's open-beads list should be small (~5-15 items) and most
beads should be *moving* (recently filed, updated, or closed). When it
grows past ~15 items or contains beads filed weeks ago that haven't
moved, audit each open bead:

1. Still load-bearing? Stale → close (per "When to close").
2. Active but stuck → name the blocker (closing note or updated
   description).
3. Active and unstuck → name what it needs to move.

The audit produces a designer report.

## When `.beads/` reports a database lock

Symptom: `bd` returns a database-lock error. Cause: storage-engine
contention — two `bd` processes writing at once, or a stale lock file.
Treat it as transient storage contention, not coordination ownership.
Fix: retry as the next natural action. If retries keep failing, the lock
file may be stale — `ls -la .beads/` to inspect. Recovering from lock
state is a tooling concern, not a coordination one.

## See also

- `autonomous-agent.md` — when to file a BEADS task for blocked work.
- `reporting.md` — the parallel hygiene discipline for designer reports.
- `jj.md` — branch-naming and bookmark cleanup after merge.
