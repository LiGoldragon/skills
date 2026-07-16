# Skill - repository publication

## Publication Rules

Use this when a code or engine repository lacks a remote, needs a public remote, or must make dependency pushes portable.

Repositories are public by default. Use private visibility only when a concrete privacy or safety constraint requires it, such as secrets, credentials, personal data, unpublished third-party code, or explicit confidentiality.

Public creation under the psyche's GitHub owner is standing pre-authorized. Do not ask or repeatedly seek visibility permission absent such a conflict. Public-by-default visibility never authorizes publishing private information, secrets, credentials, or unreviewed private material.

Before creation, inspect configured remotes and query the canonical owner/name on the forge. Create a repository only when no remote repository already exists. Then create the public GitHub repository from the local source:

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
