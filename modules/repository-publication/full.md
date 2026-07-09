# Skill - repository publication

## Publication Rules

Use this when a code or engine repository lacks a remote, needs a public remote, or must make dependency pushes portable.

Code and engine repositories are public by default. Use a private repository only for secrets, credentials, personal data, unpublished third-party code, or an explicit confidentiality constraint.

Create the public GitHub repository from the local source when the repository does not already exist:

```sh
gh repo create LiGoldragon/<name> --public --source . --remote origin --push
```

When the forge repository exists but the local repository lacks `origin`, inspect the canonical remote and add it as remote configuration; raw Git is acceptable only for remote configuration.

```sh
gh repo view LiGoldragon/<name> --json nameWithOwner,visibility,sshUrl
git remote add origin git@github.com:LiGoldragon/<name>.git
```

Use Jujutsu for ordinary history and bookmark pushes after the remote exists.

A dependency is portable only when consumers point at a public owner/repo remote and the required branch or bookmark is pushed. Local path dependencies, unpublished producer branches, and missing remotes block portable closeout.

Do not change an existing private repository to public without explicit authorization.
