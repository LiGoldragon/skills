# Skill - repository closeout

## Closeout

Use repository closeout after implementation and validation evidence exist. Inspect local instructions and status first, preserve unrelated edits, and do not manufacture green evidence for missing checks.

Use `jj`/Jujutsu for normal version control. Every description-taking command uses an inline message. Before publishing, confirm bookmark reachability, repository status, and that no descriptionless authored commit is being pushed.

For primary-style direct main closeout:

```sh
jj status --no-pager
jj commit -m 'short imperative message'
jj bookmark set main -r @-
jj git push --bookmark main
```

Use BEADS/beads for tracked work that must survive the session or coordinate with other work. Close a bead only after acceptance criteria pass or the bead is invalidated; closing notes name durable evidence such as the commit, validation artifact, output file, or superseding task.

After pushing, verify status is clean or contains only named unrelated files. Report basis commit, bookmark, commands run, push result, and remaining blockers or disposition.
