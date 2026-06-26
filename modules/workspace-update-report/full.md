# Skill — workspace update report

## What it is for

The workspace generates a lot of motion — Spirit captures, commits,
reports, skill edits, component changes. An agent reading at a single
moment can't see the *direction* of that motion without rebuilding the
picture from scratch. The workspace update report is the periodic
synthesis that holds the direction together: files changed and why,
new skills, new components, new intent, new patterns, retired surfaces.
The psyche reads it to understand recent shape without opening every
commit.

## When to write one

- **Psyche explicitly requests one** — the most common trigger.
- **End of an extended busy period** — a major sweep, a multi-day
  implementation push, a wrapping meta-report cycle. Cadence isn't
  fixed; it fires when there's enough motion to summarize.
- **Before a long context-loss event** — re-platform, lane retirement,
  context compaction across many sessions. The report becomes the
  durable summary.

It is NOT for per-commit narration (commit messages), per-session
handover (the `Handover` variant), auditing a surface (`Audit`), or
capturing intent (Spirit).

## Method

### 1. Find the baseline

```sh
ls /home/li/primary/reports/*/ | grep -E '\-Update-' | sort | tail -1
```

The previous report's date (and its declared commit hash) is the
baseline. If this is the first one, pick a sensible baseline — a recent
date with clear workspace state, the last major checkpoint, or roughly
two weeks back — and name the choice plus rationale in the report.

### 2. Survey the period

Five surfaces, in roughly increasing depth:

- `git log --since=<baseline>` — commit titles, authorship; skim the
  shape of work.
- `git log --since=<baseline> --stat` — per-commit file changes; which
  files moved and how much.
- `spirit "(PublicRecords (Any None))"` plus `LookupStash` — broad
  Spirit capture survey; narrow with an eight-field `Observe` query when
  the report needs a specific domain or phrase.
- `ls reports/<role>/ | grep '<date-range>'` — reports written across
  lanes.
- `skills/` diff against baseline — discipline evolution.

The first three are mechanical; the last two need judgment about what's
significant versus noise.

### 3. Synthesize

Six sections plus front matter:

| Section | Substance |
|---|---|
| Baseline + period | Dates / commits / Spirit range covered. |
| Headlines | The 3-7 most important shifts, each one paragraph. |
| Intent captures | Spirit records grouped by topic; bracket-quote the load-bearing ones. |
| Skill + ESSENCE / INTENT evolution | Which discipline files changed, what changed, why it mattered. |
| Reports — landed, retired, in-flight | Per role/lane: what was produced, what retired. |
| Component / repo state | Code-side motion: new code/skills, production-orientation shifts, what stayed quiet. |

Close with a **forward-look** section naming what's queued or pending
for the next period.

Write in a narrative voice — talk to a human being, not citation-heavy.
Use Spirit record numbers to mark a *range* when highlighting a span of
intent, not as constant inline citations on every claim. When a single
record is load-bearing, name the number and bracket-quote its summary.
When summarizing a region of intent, use ranges or phrasings
("the recent corpus shows …").

### 4. Chain to the previous report

Name the previous workspace update report's path (if any) and declare
this report's own commit hash as the next baseline. The chain is the
discipline's continuity.

## Filename and front matter

```
reports/<role>/<N>-Update-<period-name>-<date>.md
```

`<period-name>` is a short kebab-case label for the span (e.g.
`since-baseline`, `week-of-trace-redesign`, `spirit-next-cutover`); the
date is the first-written date. Example:
`reports/designer/491-Update-workspace-changes-since-baseline-2026-06-03.md`.

```yaml
---
title: 491 — Workspace update report (since baseline)
role: designer
variant: Update
date: 2026-06-03
topics: [workspace-update, change-survey, context-maintenance]
description: |
  First workspace update report in the series. Baseline pick:
  <date + rationale>. Surveys git log, Spirit captures, reports,
  and skill evolution across the period. Names what shifted and
  what's queued.
---
```

The `variant: Update` field marks this as an update report; indexing
tools key off it.

## Tone and scope

Narrative, terse, substance over format. The job is to compress a
period's motion into something the psyche reads in one sitting; bloat
defeats the purpose. A typical update report is 200-400 lines — larger
when the period was unusually busy, smaller when quiet.

An update report does NOT propose new design (`Design`), audit a
surface (`Audit`), ratify decisions (it can surface candidates, but
ratification is the psyche's job), or duplicate Spirit captures (it
references them by number + bracket-quoted summary).

## See also

- `skills/reporting.md` — report variants and substance-first
  discipline.
- `skills/report-naming.md` — filename + front matter.
- `skills/context-maintenance.md` — the adjacent maintenance kit.
