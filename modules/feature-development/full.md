# Skill — feature development

## Scope — code repos only, NOT primary

This worktree model applies only to the code repositories under
`/git/github.com/LiGoldragon/`. It does NOT apply to primary (the
workspace coordination repo at `/home/li/primary`): on primary everyone
works on `main` directly — edit, commit, push straight to `main`, never
a feature branch or `~/wt` worktree. See `skills/jj.md` §"Primary is
always main".

## When to use it

When a feature spans more than one commit and one session — multi-step
refactors, multi-repo arcs, anything tracked by a `feature` bead — the
work lives on a non-`main` branch in a separate worktree. Do NOT check
out a feature branch in the canonical ghq checkout: that makes `main`
unavailable to peer agents (and to you in another session) until the
feature lands. The worktree keeps the canonical checkout on `main` while
the feature work happens elsewhere; many agents and branches coexist
without competing for one checkout.

Skip the worktree when:

- The change is a single commit landing on `main`.
- You're only reading code that exists on `main`.
- It's a throwaway experiment — use `jj new` on the ghq checkout and
  abandon if it doesn't pan out.

The trigger is *"this work needs its own branch."* If it's going
straight to main, no worktree.

## A branch has no limits — the major-break mechanism

A feature branch has no limits. To test something radical — a different
architecture, a from-scratch rewrite, a contrarian shape — do it on a
branch. You can wipe the entire working tree and rebuild from nothing:
delete every file, start over, throw it all away if it fails. Nothing is
lost, because `main` is untouched until the branch integrates.

This is why a major architectural break does NOT justify a new
repository — the clean slate a fresh repo seemed to offer is exactly
what a branch already gives you. Create a new repository only for a
genuinely new project (`skills/repository-management.md`); breaks,
rewrites, experiments, mockups, and repros are all branches.

## Paths

- `/git/github.com/<owner>/<repo>/` — canonical ghq checkout. Stays on
  `main`, indexed by `ghq list`, never holds a feature branch.
- `~/wt/github.com/<owner>/<repo>/<branch-name>/` — feature worktree.
  Same repo, separate working copy, on the named branch. Under the
  user's home (writable without sudo), not indexed by ghq, created and
  removed per feature.

`~/wt/` mirrors the `/git/github.com/...` structure underneath so paths
are predictable. The branch name is the directory leaf — one branch per
worktree. A repo can host multiple worktrees at once, each independent.

## Branch naming

Bare descriptive names — `horizon-re-engineering`, `pty-fanout`,
`mind-graph-redesign`. Never `push-` prefixed: the `push-` convention
(`skills/jj.md`) is for short-lived review-cycle bookmarks; long-lived
feature arcs are a different shape.

Use the same branch name across every repo the feature touches, so a
multi-repo feature ends up with matching branches and worktrees at the
parallel paths in each repo. The feature bead's description carries the
branch name explicitly (`skills/beads.md`) so any agent picking up the
bead lands on the right branches.

## Creating a worktree

Most repos here are jj-colocated. From inside the canonical checkout:

```sh
mkdir -p ~/wt/github.com/<owner>/<repo>/
jj -R /git/github.com/<owner>/<repo> workspace add \
    ~/wt/github.com/<owner>/<repo>/<branch-name>
```

`jj workspace add` creates a workspace that shares the original's
operation log and bookmark space; its `@` is independent, so you can
edit different commits in the canonical checkout and the worktree
without conflict. Then point the worktree's `@` at the branch:

```sh
cd ~/wt/github.com/<owner>/<repo>/<branch-name>
jj edit <branch-name>          # or: jj new <branch-name> for a fresh change on top
```

For a plain git repo (rare here, lacks `.jj/`), fall back to
`git worktree add`, then run `jj git init --colocate` inside the
worktree if jj operations are needed (`skills/jj.md`).

## Working, pushing, cleaning up

A worktree is a normal jj working copy — standard `skills/jj.md`
commit/push discipline applies, claims work per-path
(`orchestrate "(Claim ...)"`), and reports go in workspace-level
`reports/<role>/`, not in the worktree.

Push the feature branch (`--allow-new` on the first push of a new
bookmark):

```sh
jj bookmark set <branch-name> -r @-
jj git push --bookmark <branch-name>
```

When the feature lands and merges to `main`, delete the worktree before
deleting the branch:

```sh
jj -R /git/github.com/<owner>/<repo> workspace forget --workspace <branch-name>
rm -rf ~/wt/github.com/<owner>/<repo>/<branch-name>
jj -R /git/github.com/<owner>/<repo> bookmark delete <branch-name>
jj -R /git/github.com/<owner>/<repo> git push --deleted
```

(For a git worktree, use `git worktree remove` instead of the first two
lines.) Stale `~/wt/` directories that no longer match an active feature
bead are smell — they confuse the next agent about what's in flight, so
clean up at merge time.

## Subagent feature work

Subagents always create feature branches when touching repos. A subagent
launched to edit code, run a prototype that may ship, or scaffold a repo
starts on a feature branch in a separate worktree; the parent assigns
the branch name and report path before launch, and the dispatch prompt
must state the feature-branch requirement so the subagent does not commit
to `main`. Research-only subagents that write only their preassigned
report need no worktree.

## When the repo is already locked — worktree from main

If the canonical checkout is claimed by another lane and work must start
now, cut the feature-branch worktree from the last remote `main`. The
locked repo's `@` may be mid-edit on the other lane's branch; cutting
from `main` keeps the new branch independent of that in-flight state and
preserves the lock-holder's ability to land without collision. The
worktree path is its own claim scope, so the lock is not contested.

## Why a worktree, not just a branch

A bare branch (no worktree) means whoever is checked out in the
canonical location can't switch to `main` without losing their working
state — two agents end up fighting over what the checkout should be.
Worktrees avoid this: the canonical checkout stays on `main` for anyone
who needs to read it, while each feature worktree is a separate working
copy with independent state and `@` over the same underlying repo.
Coordination is structural rather than serialized through one shared
mutable checkout (`skills/push-not-pull.md`).

## Interaction with the orchestration protocol

A worktree's path is its own scope for `orchestrate "(Claim ...)"`. Claim
it when you start work there:

```sh
orchestrate "(Claim (<lane> [(Task primary-XXX) (Path /home/li/wt/github.com/<owner>/<repo>/<branch-name>)] [reason]))"
```

This is distinct from the canonical checkout's path — two scopes, no
overlap. Multiple agents can hold claims on different worktrees of the
same repo; they conflict only if both claim the same worktree path.
Together with the bead naming the branch, this gives a complete
coordination story: the bead names the branch, the branch lives in
worktrees at predictable paths, agents claim worktrees individually.

## See also

- `skills/jj.md` — version-control discipline for commits and pushes.
- `skills/beads.md` — feature beads carry their branch name.
- `skills/repository-management.md` — canonical ghq checkout layout.
