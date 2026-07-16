# Skill — repository management

## Rules

Use this for GitHub repository objects, metadata, issue or PR operations, and local clone discovery through `ghq`.

Use `ghq` to fetch and discover local clones. Do not hand-create clone directories or rely on filesystem searches as the repo index. Preserve the canonical owner/repo casing reported by the forge.

Examples use canonical identities, not local paths:

```sh
ghq get github.com/<owner>/<repo>
ghq get --update github.com/<owner>/<repo>
ghq list
ghq look <substring>
```

Use `gh` for forge-side objects and metadata:

```sh
gh repo view LiGoldragon/<name> --json nameWithOwner,visibility,description,homepageUrl
gh issue list --repo LiGoldragon/<name> --state open
gh pr checks --repo LiGoldragon/<name> <number>
```

Use Jujutsu for local history and bookmark pushes. Do not use raw Git for ordinary commits or pushes.

## New repositories

Create a new repository only for a genuinely new project or concern. Major rewrites, experiments, mockups, fixtures, reproductions, and alternate versions of an existing project belong on branches or tracked work items in the existing repository.

Before creating or repurposing a repository, surface unmerged branches, stale dependencies, and dependencies with unmerged branches that affect the decision.

When a new repository is justified, treat public visibility as the default and use `repository-publication` for remote discovery, creation, and privacy gates. Ask about visibility only when a concrete privacy or safety conflict applies.

When unsure whether the project is genuinely distinct, ask about the project boundary before creation.
