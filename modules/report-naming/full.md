# Skill — report naming

Naming, iteration, and supersession discipline for reports. Companion to `skills/reporting.md` (chat-vs-report, inline summaries, visuals, the pickup-point principle, session drain).

## Reports are fresh-context pickup points

When a report is warranted, write it so an agent starting from a clean context can pick the work up, reason about it, and — where the work is implementable — implement it. The reader has none of the writer's session memory; the report supplies it. Implementable work is linked into a bead dependency graph (`bd dep <blocker> --blocks <blocked>`), so a fresh agent finds both the reasoning (the report) and the ordered work (the beads) without the original session. A continuation or review report states explicitly what it supersedes and deletes its predecessor in the same commit.

## Filename

`reports/<lane>/<N>-<Variant>-<primary-topic>[-<secondary-topic>]-<title-slug>.md`

- `<lane>` is the session-intent name — the directory for this work session, named for what the session is about (`newLanesDesign`, `schemaWorkAudit`), not for a fixed role. The lane's discipline (designer, operator, …) is metadata that loads skills and authority, not the directory name.
- `<N>` is **per-lane**, the next integer after the highest-numbered report in this lane's directory — `reports/newLanesDesign/4-…` and `reports/schemaWorkAudit/4-…` coexist because the lane directory disambiguates. No leading zeros. No date prefix: commit history records when a report landed.
- `<Variant>` is the capitalized report kind — `Psyche`, `Design`, `Audit`, `Research`, `Synthesis`, `Closeout`, `Handover`, `Update`, `Refresh`. Every report has one. It appears in BOTH the filename (for grep discoverability) and the YAML `variant:` field (for typed metadata). `Update` is the recurring workspace update report (`skills/workspace-update-report.md`); `Refresh` is the context-maintenance output that agglomerates prior reports on a topic into one better form, deleting the merged sources as it lands (`skills/context-maintenance.md`).
- `<primary-topic>` is the durable topic cluster, placed first so `rg --files reports | rg '/[0-9]+-schema-'` finds a topic's current report surface without knowing exact titles. Keep topic atoms short and stable: `nota`, `schema`, `macros`, `emission`, `spirit`, `wire`, `upgrade`, `runtime`, `reporting`, `orchestrate`.
- `<title-slug>` is the specific subject in kebab-case.

Example: `reports/schemaWorkAudit/3-Audit-schema-macros-index-and-loading.md`

Find the next number, then add 1. Numbers are gap-tolerant and never reused after deletion.

```sh
ls ~/primary/reports/<lane>/ | grep -E '^[0-9]+-' \
  | sort -t- -k1,1n | tail -1
```

## Iteration with `-v2` / `-v3`

While a report is actively refined with feedback, the edited version takes a `-v2` / `-v3` suffix (v1 is implicit, no suffix): `225-…`, `225-v2-…`, `225-v3-…`. The current version is canonical; delete the predecessor in the same commit that lands the successor. Don't accumulate versions side by side.

## Supersession with a new number

When the topic shifts enough that the name after the number should change (concept → implementation, scope redirect, absorbing an audit's findings), write a new numbered report carrying forward anything still relevant, and delete the predecessor in the same commit. The predecessor's number is retired; the next report takes next-highest-plus-one, not the freed number.

## Topic agglomeration

When a topic accumulates many reports, do not bulk-rename old files to tidy the directory. Write one current primary-topic report, carry forward the load-bearing substance, list the sources read inside the new report, then delete only the predecessors whose substance fully migrated — in the same committed change that lands the replacement. Historical filenames remain valid locators in git history; new reports use the topic-prefix convention forward.

## Commit before delete

**Never delete an uncommitted report** — that is total loss. Deleting a *committed* report only removes it from the work tree; git history retains the substance and it stays recoverable. So for any rename, supersession, or agglomeration: the new report must be committed in the same commit as the predecessor's deletion. Both the addition and removal land in one whole-working-copy `jj commit` (no path arguments — see `skills/jj.md`), keeping the replacement and its source visible together in one change.

## Session drain and directory retirement

The lane directory is the garbage-collection unit. When a session drains at close — every idea routed to exactly one of intent (captured via the Spirit CLI), work (a bead linked into a dependency graph), or abandon (already-landed, stale, or wrong; git preserves it) — the whole `reports/<lane>/` directory is deleted in one move, not file by file. Git history and the session transcript are the archive. Record the retirement with a single append-only entry in `protocols/retired-lanes.md` (lane name, discipline, the git revision range holding its reports, transcript pointer, drain date, one-line statement of what it decided) so the drained session stays discoverable for regression and model-behavior forensics without re-growing the working report tree.

## See also

- `skills/reporting.md` — the larger reporting discipline.
- `skills/jj.md` — the version-control flow these commits use.
