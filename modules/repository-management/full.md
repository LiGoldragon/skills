# Skill — repository management

## Rules

Use this for GitHub repository objects, metadata, issue or PR operations, and local clone discovery through `ghq`.

Repositories are public by default. Use a private repository only for secrets, credentials, personal data, unpublished third-party code, or an explicit confidentiality constraint.

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
gh repo create LiGoldragon/<name> --public --source . --remote origin --push
gh repo view LiGoldragon/<name> --json nameWithOwner,visibility,description,homepageUrl
gh issue list --repo LiGoldragon/<name> --state open
gh pr checks --repo LiGoldragon/<name> <number>
```

Use Jujutsu for local history and bookmark pushes. Do not use raw Git for ordinary commits or pushes.

## New repositories

Create a new repository only for a genuinely new project or concern. Major rewrites, experiments, mockups, fixtures, reproductions, and alternate versions of an existing project belong on branches or tracked work items in the existing repository.

When unsure whether a name should become a repository, ask before creating the repo.
