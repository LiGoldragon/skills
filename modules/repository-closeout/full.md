# Skill - repository closeout

## Rules

Use repository closeout after implementation and validation evidence exist. Inspect local instructions and status first, preserve unrelated edits, and do not manufacture green evidence for missing checks.

Use `jj`/Jujutsu for normal version control. Every description-taking command uses an inline message. Before publishing, confirm bookmark reachability, repository status, and that no descriptionless authored commit is being pushed. Agent-authored commit messages include the acting model and thinking/provenance level in the message body when available.

After validation, commit and push implementation changes by default. Do not leave validated implementation work uncommitted unless the brief explicitly says review-only, experiment-only, or no-commit.

If the work creates or consumes a producer dependency, make that dependency portable before publishing. If portable closeout is not possible, report it as a hard blocker.

For primary-style direct main closeout:

```sh
jj status --no-pager
jj commit -m 'short imperative message'
jj bookmark set main -r @-
jj git push --bookmark main
```

Use BEADS/beads for tracked work that must survive the session or coordinate with other work. Close a bead only after acceptance criteria pass or the bead is invalidated; closing notes name durable evidence such as the commit, validation artifact, output file, or superseding task.

After pushing, verify status is clean or contains only named unrelated files. Report basis commit, bookmark, commands run, push result, and remaining blockers or disposition.
