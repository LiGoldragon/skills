# Skill — work tracking

## Use work items for short tracked work

A bead is a small work item with enough context, acceptance criteria, and dependency links for another agent to pick up. Use beads when work must survive the session or coordinate with other work.

## Claim before working

Before editing for a bead, inspect its state and dependencies. Claim only the bead you are actively working. Do not claim a broad area to reserve it.

Record the claim in the tracker surface the repo uses. If no tracker is configured, ask or use the task's explicit instruction.

## Retry transient tracker locks

Run `bd` tracker commands sequentially, not through parallel tool calls or
concurrent shells. `bd` uses a single-writer embedded Dolt store; if a command
reports the exclusive `.beads/embeddeddolt` lock, wait for the owning operation
to finish and retry the same command. Do not spawn concurrent retries. Treat the
lock as a blocker only after several short retries fail, and report the exact
command and error.

## Keep bead text executable

A good bead states:

- the desired outcome;
- the owning repo or component by canonical name;
- files or surfaces likely involved;
- acceptance criteria;
- dependencies and blockers;
- verification expected.

Avoid transcript, speculation, and generic how-to prose.

## Split by acceptance boundary

Split a bead when part of the work can land and be verified independently. Keep a bead together when the acceptance proof is one atomic behavior.

Dependencies are directional: producer before consumer, schema before generated code, contract before implementation, migration before removal.

## Update as facts change

When discovery changes scope, update the bead with observed facts and the new blocker or split. Do not rewrite history into certainty. Keep comments concise and evidence-backed.

## Weigh age when judging staleness

Age is an important factor in a bead's staleness, though not the sole test and not an auto-close threshold. The older an open bead, the more its retention must be justified rather than assumed: as a rough gradient, roughly two weeks old is suspicious and about a month old is strongly suspect. Keep an old bead only when it still maps to an actively developed line of work, shown by recent commits; otherwise treat it as a candidate to close as invalidated with a reversible reason. When triaging a backlog, sort by age and scrutinize the oldest first.

## Close with evidence

Close a bead only after the acceptance criteria pass or the bead is explicitly invalidated. Include commit identifiers, commands run, and any remaining follow-up bead. If blocked, leave it open and name the blocker.

## Anti-patterns

- beads that restate a prompt without acceptance criteria;
- umbrella beads that hide independent work;
- closing because code was written but not validated;
- keeping an old bead open by inertia when nothing active maps to it;
- using comments as an archive;
- creating repo-specific process doctrine in the bead body.
