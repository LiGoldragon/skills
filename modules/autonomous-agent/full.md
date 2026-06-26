# Skill — autonomous agent

How to act when no human is in the loop for routine obstacles.

## Scope

This skill is the work-loop discipline: bead-flow, claim, work,
close, release, and the routine obstacles agents resolve without
asking. For the psyche-interface side — how prompts come in, how
intent gets captured, how chat replies are shaped, when to dispatch a
subagent, when to ask vs decide — see `skills/human-interaction.md`.

## Active beads — check first, work them through

Before anything else in a session, check active beads:

```sh
bd ready                          # ready-to-work
bd list --status open --limit 0   # every open bead
```

Open beads are the workspace's continuing intent. They outrank
session-default behavior: if you don't know what to do, the answer is
the highest-priority open bead that fits your topic. The user's direct
prompt in the current turn always wins (they file the beads); absent a
contrary instruction, work the bead. Skipping the check means working
from a stale model of what's outstanding between sessions.

Beads carry topic labels (`nota`, `persona`, etc.), not `role:*`
labels. Any agent picks up any bead by topic fit, not by lane.

### The flow

1. Check active beads. Take the highest-priority open one fitting
   your topic.
2. Claim the scopes (the daemon claim coordinates concurrent editing; bead
   pickup itself needs no claim):
   `orchestrate "(Claim (<your-lane> [(Task <bead-id>) (Path /absolute/path)] [reason]))"`.
3. `bd update <bead-id> --status in_progress`.
4. Work it through.
5. `bd close <bead-id> -r "<note pointing at where the work landed>"`.
6. `orchestrate "(Release <your-lane>)"`, or re-claim with the next
   bead's scopes.
7. Loop.

### Session end — prove the active work is not forgotten

Before the final response after non-trivial work, re-check the bead
you claimed and related open beads (`bd show <bead-id>`,
`bd list --status open --limit 0`), then make one state true:

- Shipped: close the bead with a note naming the commit, report,
  skill, or path where the work landed.
- Still active: leave a short bead note naming the current blocker or
  exact next action.
- Moot: close with a supersession note.

Do this before releasing the lane lock. The lock file says what you
are actively touching; the bead says why the work is not done yet. A
context compaction erases the harness's memory — it must not erase the
workspace's memory.

### Naming reports in chat

When a session produced reports, name each by its full workspace path
(`reports/<role>/<N>-<topic>.md`), not a bare number, `/N`, filename,
or "the report." The user copies the path from chat and opens the
exact file without guessing.

### When the user's prompt and the active bead disagree

The user's direct instruction wins — beads are durable intent, user
prompts are live intent, live overrides durable. Acknowledge the open
bead, note you're deferring (or close it if moot), and work the
instruction.

But if the prompt is generic ("what should we do?") or exploratory
("any thoughts?"), the active bead IS the answer: report it and ask
whether to start.

### File a bead vs. just do it

File a bead when the work spans more than this session, has a
definition of done, and isn't already tracked (see `skills/beads.md`).
Just do it when the work is a routine obstacle with a standard
solution — the rest of this skill.

## What this skill is for

When you hit a known-solvable obstacle mid-work, you solve it and
continue. You do not stop and ask permission for problems that have
standard solutions in this workspace. Asking produces friction;
friction produces stalling; stalling produces stale context.

The trade-off is sharp: you ask only when an action is destructive,
hard to reverse, significantly out of scope, or operates on shared
state outside what was asked. Everything else, you do.

## Standard solutions

### Shared workspace work needs orchestration

Before editing, creating, formatting, or deleting files, claim the
scopes through `orchestrate/AGENTS.md`:

```sh
orchestrate "(Claim (<lane> [(Path /absolute/path)] [reason]))"
orchestrate "(Release <lane>)"
```

The daemon commits accepted claim state, projects lane lock files, and rejects
overlapping active scopes. Release as soon as the work is done. If the work
can't proceed, file a short bead with the blocker and the next required action.

Beads are never claimed — any agent may write a bead at any time; do
not claim `.beads/`. If `bd` reports a backend database-lock error,
treat it as transient contention, not ownership: retry as the next
natural action, or continue with a clear note.

### When you finish a batch of changes, commit and push

That's the standing rule — blanket authorization, no asking. Commit
the whole working copy (`jj commit` with no path arguments), sweeping
in any other agent's in-flight files; leaving a peer's change
undrained forks the history, committing everything keeps it linear.
Multi-lane commits are accepted. Before finishing, run `jj status` in
every area you changed and leave nothing uncommitted. Full procedure
and the standard fixes (HTTPS push failure, divergence, missing
`.jj/`) live in `skills/jj.md`. If a VCS obstacle blocks you and jj.md
doesn't name the fix, surface it instead of inventing one — that's how
the skill grows.

### A design wants polling

Symptom: the next step wants a sleep loop, a periodic file reread, a
retry timer for unknown state, or "check again later."

Fix: producers push, consumers subscribe. Wire the producer's
subscription primitive, defer the dependent feature, or escalate. Do
not add polling "for now" — it is a design failure unless it is a
named carve-out in `skills/push-not-pull.md`.

### A required tool is missing from PATH

Symptom: `command not found` for `rustfmt`, `clippy`, `jq`, etc.

Fix: invoke via Nix without installing globally:

```
nix run nixpkgs#<package> -- <args>
```

Don't reach for `cargo install`, `pip install`, `npm install -g`, or
distro package managers. The setup is Nix-managed end-to-end;
out-of-Nix installs break reproducibility.

### A stateful or custom test command is becoming part of the work

Symptom: while debugging, you keep running a long command by hand — an
ignored integration test, a real-harness test, a stateful script that
depends on local auth.

Fix: turn it into a named repo script exposed through the flake:

```
scripts/test-actual-thing
nix run .#test-actual-thing
```

The script may still run stateful commands (`cargo test` against the
working tree). The point is not a pure derivation — it's documenting
the command, its environment, and its setup in versioned files.
Iteration becomes: edit the script, run the named Nix command, inspect
output, repeat. If a one-off debug command teaches you something, keep
it under a `debug-*` name or fold it into the real test script before
finishing. This is the stateful-test branch of `skills/testing.md`.

### A doc references a removed/renamed thing

Fix: update the reference to the new home. Don't leave half-broken
text "for the user to clean up later." If the new home doesn't exist,
raise the question — don't paper over.

### A repo has no `skills.md` after substantive work in it

Symptom: you've spent meaningful time in a repo — read its
ARCHITECTURE.md, AGENTS.md, source; understood its role and
invariants; landed a non-trivial change — and `<repo>/skills.md` does
not exist.

Fix: write it before finishing. Read `skills/skill-editor.md` for the
conventions, gather what you learned (the repo's intent, its
invariants, what it is the canonical owner of, neighboring repos worth
pointing at), write `skills.md` at the repo root, commit and push.

A 10-minute typo fix is not enough context — you don't yet know what's
load-bearing vs incidental, and a skill written without depth is worse
than none because future agents trust it. If unsure whether your work
was substantive enough, err toward writing it; a thin-but-honest skill
still helps.

## When to ask anyway

Solving a routine obstacle is autonomy. These are not routine — ask
first:

- Destructive operations that aren't pure undo: deleting branches
  outside your scope, dropping data, force-pushing, deleting files
  outside your stated work, removing dependencies.
- Hard-to-reverse operations: amending pushed commits, rewriting
  public history.
- Out-of-scope cleanup: "while I was here I noticed X is ugly" — say
  so, don't just do it.
- Shared state: actions affecting other agents' or humans' work
  in-flight (visible in locks, recent commits, open PRs).
- Large-scope assumptions: when the task expands beyond what was
  asked. Surface the expansion; let the caller decide.

## See also

- `skills/human-interaction.md` — the psyche-interface side.
- `skills/skill-editor.md` — read before editing any skill file.
- `skills/jj.md` — version-control discipline; commit/push fixes.
