# Skill — jj

Version control here is `jj`. Commit and push through `jj` after every
meaningful change; raw `git` survives only as the two named escape hatches below.

## Primary is always main — no branches, ever

On **primary** (the coordination repo at `/home/li/primary`: reports, skills,
`AGENTS.md`, `INTENT.md`, `ESSENCE.md`, `protocols/`, `orchestrate/`) everyone
ALWAYS works on `main` directly. The entire flow is three lines:

```sh
jj commit -m '<short verb + scope>'
jj bookmark set main -r @-
jj git push --bookmark main
```

No feature branches, no `next`, no `wip`, no `push-*` bookmarks, no
rebase-onto-main choreography, no `~/wt` worktree for primary. Do NOT stage a
primary edit on a side change or bookmark before main.

The ONLY divergence handling on primary is the fetch-and-rebase escape hatch
(see §"Push rejected"): on a rejected push, `git fetch origin` + `git rebase
origin/main` + push, or `jj new main@origin` to start fresh on the latest main.
No bookmark choreography ever recovers a rejected push on primary.

The branch / worktree / `next` models in `skills/main-next.md` and
`skills/feature-development.md` apply ONLY to the code repos under
`/git/github.com/LiGoldragon/` — there operators own `main` and designers work
on `next` or feature branches in `~/wt`. Primary is the exception.

### Keep primary clean

- **Commit eagerly and impersonally.** Commit the entire working copy (no path
  arguments) — never just your own paths. Committing is janitorial, not the
  report-creator's job. See dirty state in primary? Commit it, naming the
  contents plainly without apology (`commit pending reports + skill edits`).
- **Lock selectively.** When you must claim, claim only the specific files or
  subfolders you will edit — never the whole workspace; over-locking bred the
  fork-for-push dance. Reports need no lock at all: report lanes are per-role
  and claim-exempt. A lane writes only in its own `reports/<role>/`.

## At-a-glance — inline-form cheat sheet

Every description-taking command takes `-m '<msg>'` inline; never let jj fall
back to an editor.

| Command | Canonical form |
|---|---|
| Commit working copy | `jj commit -m '<msg>'` |
| Edit a parent's description | `jj describe @- -m '<msg>'` |
| New empty change | `jj new -m '<msg>'` |
| Split out paths | `jj split -m '<msg>' <paths>` |
| Squash into ancestor | `jj squash --into <rev> --use-destination-message` |
| Move bookmark | `jj bookmark set main -r @-` |
| Push | `jj git push --bookmark main` |
| Create + push new branch | `jj bookmark create <name> --to @ && jj git push --bookmark <name> --allow-new` |

These BLOCK the session on `Waiting for Emacs...`, so they are forbidden:
`jj describe` / `jj commit` / `jj split` without `-m`, and `jj new` without `-m`
(except when `-A`/`-B` revsets imply the description).

## What this skill is for

Whenever you make meaningful changes — even a shipped one-line edit — apply this
before moving on:

1. Group the changes into logical commits.
2. Commit each group with a short verb-plus-scope message.
3. Push immediately, per commit, not in batch.

This applies to every tracked repo. Don't ask permission for routine
commit/push; the user has granted blanket authorization for the standard flow.
Save asking for the listed exceptions at the end.

Every Li repo is a Git-backed colocated jj repository: the working-history
interface is `jj`; Git is the remote/storage layer. If a repo lacks `.jj/`, run
`jj git init --colocate`. For the underlying CLI (`jj` commands, the `@` model,
undo, bookmarks) see lore's `jj/basic-usage.md` — that is *how jj works*; this
is *how we use it here*.

## Raw `git` is forbidden for daily commits

The default tool for every commit is `jj`. Don't reach for `git add` / `git
commit` / `git push` / `git checkout` for normal work. When a commit feels hard,
learn the jj idiom, don't drop to `git`.

`git` survives only as two named escape hatches, both detailed under "Standard
fixes":

1. **Per-repo HTTPS → SSH remote fix** (one-time config repair on push failure).
2. **Manual divergence resolution** when two peers pushed in parallel and
   `jj git push` rejects.

Reaching for raw `git` outside these two cases: stop, find the jj equivalent, or
escalate (`skills/autonomous-agent.md`).

## The standard flow

In a clean working tree after an edit batch:

```sh
jj st                                # see what changed (for the message)
jj commit -m '<short verb + scope>'  # finalise @, advance to fresh empty
jj bookmark set main -r @-           # point main at the just-committed change
jj git push --bookmark main          # publish
```

`-r @-` because `jj commit` advances `@` to a new empty child; the commit you
want to push is its parent. If the message contains apostrophes, use double
quotes (`-m "<msg>"`) — apostrophes inside `'…'` terminate the shell string.

## `jj describe @` is forbidden for finalising new work

The canonical commit form is `jj commit -m '<msg>'` — nothing else. Never use
`jj describe @ -m '<msg>'` to finalise new working-copy work, even though it is
functionally similar.

`jj commit` explicitly advances `@` to a new empty child, so the next edit
can't accidentally pile onto the just-described commit. `jj describe @` just
sets a description without advancing — a follow-up edit lands in the same
commit, growing it silently. The friction of `-r @-` is the discipline: the
thought "am I targeting the right commit?" is the moment to read `jj st`.

`describe` is allowed only for editing an already-committed revision's message:
`jj describe @- -m '<msg>'` (typo fix / message update before push), or
`jj describe <rev> -m '<msg>'` for an earlier revision. The forms
`jj describe @ -m '<msg>'` and bare `jj describe -m '<msg>'` (implicit `@`) are
forbidden — that is the path that bundles peer files into your commit. If you
find yourself typing `jj describe`, ask: editing an already-committed
description, or finalising new work? If finalising new work, use `jj commit`.

## Never let jj open an editor

Every description-taking jj command has an inline flag; always use it. An agent
that lets jj fall back to `$EDITOR` blocks the session on a no-op editor, or
leaves a half-described commit when the editor exits without saving.

| Command | Inline form |
|---|---|
| `jj commit` | `jj commit -m '<msg>'` |
| `jj describe @-` | `jj describe @- -m '<msg>'` (only for already-committed descriptions) |
| `jj split <paths>` | `jj split -m '<msg>' <paths>` |
| `jj split -i` | `jj split -i -m '<msg>'` |
| `jj squash --into <rev>` | `jj squash --into <rev> --use-destination-message` |
| `jj new` | `jj new -m '<msg>'` |

`jj duplicate` and `jj rebase` are not editor-bound (rebase only if conflicts
surface). The deeper rule: if a jj command would prompt for text without a flag,
find the flag — every description-taking command has one.

Do NOT reach for `EDITOR=true`, `GIT_EDITOR=true`, or any no-op-editor shim.
Those hide the fact that the wrong invocation form was used; the right answer is
the inline `-m` flag.

## Descriptionless commits are forbidden

`(no description set)` on a commit you authored is a workspace contract
violation, on equal footing with the ban on raw `git` for daily commits. The
failure mode: `jj commit` without `-m`, editor returns empty, the commit
succeeds with an empty description, no bookmark is set, and the work becomes
reachable only by op-log spelunking (this caused a 117-orphan incident).

Before every push, run:

```sh
jj log -r 'main..@- & description(exact:"")'
```

If anything appears, fix it before pushing: `jj describe <rev> -m '<msg>'`. If
`jj st` or any `jj log` ever shows `(no description set)` on a commit you
authored, describe it immediately — even before the next file edit. The instant
you continue past it, the next agent's `jj log` filters hide your work.

## Commit the whole working copy — never path-scoped

Commit the ENTIRE working copy — `jj commit -m '<msg>'` with no path arguments
— never path-scoped (`jj commit <paths>`, `jj split <paths>` to "isolate my
scope", `git add <paths>`).

The reason is the shared working copy: all agents share one. A path-scoped
commit captures only the named paths and leaves every other agent's in-flight
change undrained. With the copy still dirty, two agents can each commit their
own paths off the same base — producing sibling commits, an off-main fork that
strands work and can orphan uncommitted files.

Committing everything drains the shared copy and serializes agents through jj's
working-copy lock: whoever commits sweeps in all in-flight work, history stays
linear, nothing is orphaned. The resulting commit is often multi-lane /
"impersonal" — one message covering several lanes' changes. That is accepted;
a brief impersonal message (`commit pending reports + skill edits`) is correct.
Read `jj st` only to write an accurate message, not to decide which paths to
commit.

```sh
jj st                                 # what's in the working copy (for the message)
jj commit -m '<short verb + scope>'   # the WHOLE working copy — no paths
jj bookmark set main -r @-
jj git push --bookmark main
```

`jj split` is still legitimate for grouping your own multi-concern work into
logical commits when no peers have in-flight work — but never to leave a peer's
change undrained. When in doubt, commit everything.

## Logical commits

When the working tree contains more than one concern (and holds nothing but your
own work), split before committing. The grouping criterion, in priority order:

1. **By concern.** A documentation update is one commit; a code change another.
2. **By feature.** A multi-step feature lands as several commits, each a
   coherent step that compiles, passes tests, and reads cleanly in the diff.

This does NOT authorise leaving a peer's in-flight change undrained — see
§"Commit the whole working copy". If a peer's work is in the copy, sweep it in
with one impersonal commit rather than splitting it out. Don't fold unrelated
edits into one "miscellaneous" commit; "while I was here" cleanups get their own
commit with a clear message.

## Commit message style

Single line, short: a verb plus scope, plus an optional short clause naming the
change. The repo is implicit. Detail lives in the diff and the report. Examples:
`Slot<T> migration`, `report add 119`, `reader for typed slots`,
`AGENTS commit-style shortened`. If a single change touches multiple repos, each
repo gets its own short commit.

## Always push

After every logical commit, push immediately — blanket authorization, proceed
without asking. Unpushed work is invisible to other machines and to flake-input
consumers; forgotten pushes cause divergence and surprising forks. Don't batch
pushes "to be clean"; one push per commit.

The exception: when one logical change spans several interdependent commits (a
refactor with three sequential steps), push the whole sequence at the end. Each
commit message still names its step, not the sequence.

## Standard fixes for routine obstacles

These have known answers; fix them and keep moving.

### A repo lacks `.jj/` (jj not initialised)

`jj st` prints `Error: There is no jj repo in "<path>"` though `.git/` exists —
the repo is git-only. Fix from inside the working tree, then proceed normally:

```sh
jj git init --colocate
```

### Push fails because the remote is HTTPS — named git escape hatch

`jj git push` returns `fatal: could not read Username for
'https://github.com'`. This workspace authenticates over SSH. One-time per-repo
config repair (one of the two named raw-`git` escape hatches):

```sh
git -C <repo> remote set-url origin git@github.com:<owner>/<repo>.git
jj git push --bookmark main
```

After it lands, the normal `jj` flow resumes.

### Push rejected — remote has commits you don't have — named git escape hatch

`jj git push` returns `Updates were rejected because the remote contains work
that you do not have locally.` — another agent or machine pushed in parallel.
This is the second named raw-`git` escape hatch, and on primary it is the only
divergence handling there is (no bookmark choreography):

```sh
git fetch origin
git rebase origin/main        # replay your commits on the latest main
jj git push --bookmark main   # publish
```

Equivalently, `jj new main@origin` to start fresh and re-land. If conflicts
surface (modify/delete or content), resolve in favour of your scope's changes
(per the orchestration lock); ask only if resolution genuinely changes the
meaning of the peer's work. Then `git rebase --continue` + `jj git push
--bookmark main`. Do NOT respond to a rejected push by creating a `wip-…` or
`push-…` bookmark and rebasing onto `main@origin` — that is the forbidden
branch-dance. Fetch, rebase, push.

### Working tree has uncommitted state when you expected clean

`jj st` shows files that aren't yours — prior work landed but wasn't committed
(yours from an earlier session, or a peer's not yet pushed). Fix: `jj st` for
the message, then commit the whole working copy (no paths), sweeping in any
peer-owned changes; set main and push. Leaving a peer's change undrained is what
forks history; committing everything is the fix. Don't `jj split` to leave peer
files behind.

### `jj restore` is hazardous mid-commit

`jj restore -f <rev>` reverts the working copy to match `<rev>` without moving
`@`, silently discarding any uncommitted changes — including peers' in-flight
work in the shared copy. Use sparingly; never to "clean up before a commit" (the
right answer is committing the whole working copy). If you reach for it during
normal work, stop and check `jj st`: you almost always want to commit the whole
working copy, or `jj abandon @` with deliberate intent. A `restore into commit`
op was a load-bearing step in the 117-orphan failure.

## Per-logical-commit pushes — not batch

Don't accumulate three commits and push at the end. Each commit gets its own
push. The cost is one extra `jj git push`; the benefit is consumers see work as
it lands, parallel agents fetch the latest tip each iteration, and recovery from
a bad commit is `jj undo` rather than rolling back multiple changes. (Sole
exception: the interdependent-sequence case above.)

## End-of-session check

Before ending a session — closing the conversation, releasing a claim, handing
off, or running `jj new main` / `jj edit main` (which moves `@` off the current
chain) — confirm every commit you authored is reachable from a bookmark or from
`main`:

```sh
jj log -r 'main..@ ~ bookmarks()'
```

Empty output (or only the empty `@`) means the session ends clean. Anything else
is unbookmarked descendants of main — pushable work no one but you can find,
exactly the 117-orphan failure shape.

On **primary** there is exactly one option: land on main —
`jj bookmark set main -r <rev> && jj git push --bookmark main`. Primary never
carries `push-<topic>` or any side bookmark, including for reports.

In a **code repo**, each row needs one of:

- **Land on main** — `jj bookmark set main -r <rev> && jj git push --bookmark main`.
- **Bookmark for review** — `jj bookmark create push-<topic> -r <rev> && jj git push --bookmark push-<topic>`.
- **Explicit abandon** — `jj abandon <rev>`, only if you genuinely want the work
  gone. Discarded work is the most expensive to recover; bias always toward
  bookmark-then-decide-later.

Prefer landing on main when work is yours and complete. Reserve `push-<topic>`
for code-repo work needing review before landing — not a default "stash so I can
move on." A long-lived chain of `push-*` bookmarks is a smell; it usually means
someone forgot to advance `main`.

## `jj git push -c @` is forbidden for routine commits

`jj git push --change @` (or `-c @`) creates an auto-named `push-<change-id>`
bookmark on the remote and pushes to it. It does not advance `main`, and the
bookmark accumulates on the remote until manually deleted. Use the standard flow
instead — the commit lands on `main`, no auto-named bookmark is created,
consumers see the work immediately.

`--change` is allowed only, narrowly:

- **Orphan recovery** — bringing back abandoned prior work onto a fresh `@`:
  `jj op log -n 50` to find the orphan, `jj show <id>` to confirm,
  `jj new -m '...' <id>` to bring it back, then push.
- **Explicit "needs review before main"** — use a descriptive bookmark name
  (`jj bookmark create push-<topic>`), not the auto-naming form; descriptive
  names are findable and easy to delete after merge.

Auto-named bookmarks accumulate silently with no clean-up step — the workspace
grew to 63 stray `push-*` bookmarks before one designer pruning pass. The psyche
does not want per-change or per-report push-named bookmarks: land work on main
through the standard flow, or use a clearly-named review branch only when review
is genuinely needed and delete it after merge.

Do not create a `push-*` bookmark for routine or per-report work. Its single
legitimate use is genuine pre-main review in a code repo under
`/git/github.com/LiGoldragon/`, deleted the moment it merges. On primary there
is no legitimate `push-*` use at all — reports land on `main` like every other
primary edit.

## Bookmark cleanup after merge

When a `push-<topic>` bookmark's commit becomes an ancestor of `main` (the work
merged), delete the bookmark locally and on the remote:

```sh
jj bookmark delete push-<topic>
jj git push --deleted
```

(`--deleted` is its own mode — it can't combine with `--bookmark`; run it alone
after the local delete and it pushes every locally-deleted bookmark in one
call.) Long-lived `push-*` bookmarks mislead reviewers ("still in flight?"),
bloat `jj bookmark list`, and grow forever if no one prunes.

Include `jj bookmark list` in the session-end check. Any bookmark starting with
`push-` whose commit is already an ancestor of `main` should be deleted before
the session ends:

```sh
jj bookmark list | awk '/^push-/ {print $1}'   # candidate bookmarks
jj log -r '<commit>::main' --no-graph          # nonempty = ancestor = delete
```

## When to ask anyway

Routine obstacles are autonomy. These are not routine; ask first:

- `git reset --hard` or anything discarding uncommitted work that isn't clearly
  yours.
- Force-push to any branch, especially main.
- Amending pushed commits or rewriting public history.
- Deleting branches not in your scope.
- Changing remote URLs for reasons other than HTTPS→SSH on push failure.
- Reaching for raw `git` outside the two named escape-hatch cases above.

## See also

- `skills/autonomous-agent.md` — when to act on routine obstacles without
  asking; this skill is the VCS leaf it points at.
- lore's `jj/basic-usage.md` — `jj` CLI reference (the `@` model,
  commit/describe distinction, undo, bookmarks).
