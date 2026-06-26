# Skill — repository management

Use this when a local repo needs a GitHub remote, repository metadata changes,
basic GitHub issue/PR operations through `gh`, or finding / fetching / updating
local clones via `ghq`.

Repositories are public by default. A private repository is an exception that
needs a concrete reason: secrets, private credentials, personal data,
unpublished third-party code, or another explicit confidentiality constraint.
Absent that reason, create public.

## Where repositories live — the ghq layout

Every local clone lives at `/git/<host>/<owner>/<repo>`. `ghq` is the canonical
fetcher and indexer.

```
/git/
├── github.com/
│   ├── LiGoldragon/<repo>     ← canonical case (matches GitHub's user.login)
│   ├── Criome/<repo>          ← canonical case (Criome org)
│   ├── nix-community/<repo>
│   └── … (every other owner you've cloned from)
├── codeberg.org/<owner>/<repo>
├── gitlab.com/<owner>/<repo>
└── git.sr.ht/~<owner>/<repo>
```

`~/primary/repos/<repo>` symlinks into this tree. The symlinks are gitignored —
a local index regenerated from the filesystem.

### Cloning — `ghq get`

```sh
ghq get https://github.com/<owner>/<repo>
ghq get -p https://github.com/<owner>/<repo>     # SSH (preferred for own repos)
ghq get --update github.com/<owner>/<repo>       # update if already cloned
```

`ghq` derives the destination from the URL; don't create directories under
`/git/...` by hand. The URL's case becomes the on-disk case, so use GitHub's
canonical case (`gh api users/<name> | jq .login`): `LiGoldragon` not
`ligoldragon`, `Criome` not `criome`.

### Finding a local clone — `ghq list` / `ghq look`

```sh
ghq list                              # every clone, host/owner/repo path
ghq list | grep nota                  # find by substring
ghq list -p                           # full filesystem paths
ghq look <substring>                  # cd into a matching clone
```

`ghq list` is the source of truth for "what repos do I have locally" — faster
than `find /git/`, and shows only real git checkouts. It indexes the filesystem
on each call; there is no separate "add to index" step.

### Updating clones in bulk

```sh
ghq get --update --shallow github.com/<owner>/<repo>     # one
ghq list | xargs -I{} ghq get --update {}                # all (slow; sparingly)
```

For per-repo updates, prefer `jj git fetch` inside the checkout — it integrates
with the workspace's jj discipline. `ghq get --update` is for bulk passes.

### Don't deviate from the layout

| Don't | Do |
|---|---|
| Use `git clone` directly | `ghq get` (preserves the layout) |
| Lowercase a path GitHub canonicalises as mixed-case | Match GitHub's casing |
| `rm -rf` a clone then re-clone elsewhere | `ghq get --update` in place |
| Move a clone manually | `ghq get` it at the right location, delete the wrong one |

If the layout drifts, the fix is mechanical: `git remote set-url origin
<canonical-url>`, `mv` to the canonical path, update any `primary/repos/`
symlinks.

## When to create a new repository — only for a genuinely new project

A new repository is justified ONLY when you are creating a genuinely different
project — another product or concern entirely. It is NOT justified for a new
version, shape, rewrite, or major architectural break of an *existing* project,
and never for an experiment, mockup, repro, or design pass.

A feature branch has no limits: an agent testing something radical can wipe the
entire working tree and rebuild from scratch on a branch — delete everything,
start over, whatever the experiment needs. The clean slate a "major break"
seems to want is fully achievable on a branch, so **major breaks are branches,
not new repos.** Spinning up `-next` / `-v2` / `design-` repos for major breaks
produces a sprawl of throwaway repos and `-next` repos that never get renamed
back, leaving permanent confusion about which repo is canonical.

The test before `gh repo create`:

| Situation | Where it goes |
|---|---|
| A new, distinct project (different product/concern) | **New repository** |
| Major architectural break / rewrite of an existing project | **Branch** (wipe the tree, rebuild — `skills/feature-development.md`) |
| Experiment / spike / "test something crazy" | **Branch** |
| Mockup, repro, fixture, sandbox | **Branch** |
| A new version or alternate shape of an existing thing | **Branch** |

When unsure whether it's a genuinely new project, ask the psyche. Creating a
repository touches the repo-name surface every agent reads every session; the
bar is high and the default is always a branch.

## Create a repository

From the repo root:

```sh
gh repo create LiGoldragon/<name> --public --source . --remote origin --push
```

If the local repo already has an `origin` remote, create the remote without
rewriting local config, then push with `jj`:

```sh
gh repo create LiGoldragon/<name> --public
jj git remote add origin git@github.com:LiGoldragon/<name>.git
jj git push --bookmark main
```

Private creation is explicit and rare — use `--private` only when the reason is
clear in the task or in the repository contents:

```sh
gh repo create LiGoldragon/<name> --private --source . --remote origin --push
```

## Change visibility and metadata

```sh
gh repo edit LiGoldragon/<name> --visibility public --accept-visibility-change-consequences
gh repo edit LiGoldragon/<name> --visibility private --accept-visibility-change-consequences  # explicit reason only
gh repo edit LiGoldragon/<name> --description "Short description"
gh repo edit LiGoldragon/<name> --homepage "https://example.test"
gh repo view LiGoldragon/<name> --json nameWithOwner,visibility,url,description,homepageUrl
```

## Issues and pull requests

```sh
gh issue create --repo LiGoldragon/<name> --title "Short title" --body "Actionable body"
gh issue list --repo LiGoldragon/<name> --state open
gh pr create --repo LiGoldragon/<name> --draft --title "Short title" --body "What changed and why"
gh pr checks --repo LiGoldragon/<name> <number>
```

Use the GitHub plugin skills for deep PR review or CI triage; this skill is the
minimal daily repository-management layer.

## Version-control boundary

Use `gh` for GitHub repository objects and metadata. Use `jj` for local history
and pushing bookmarks. Do not use raw `git` for ordinary commits or pushes.

## See also

- `skills/jj.md` — local history, commits, bookmarks, pushes.
- `skills/feature-development.md` — branch-based experiments and rewrites.
