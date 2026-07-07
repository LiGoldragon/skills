# Skill - repository closeout

## Rules

Use repository closeout after implementation and validation evidence exist. Inspect local instructions and status first, preserve unrelated edits, and do not manufacture green evidence for missing checks.

Use `jj`/Jujutsu for normal version control. Every description-taking command uses an inline message. Before publishing, confirm bookmark reachability, repository status, and that no descriptionless authored commit is being pushed. Agent-authored commit messages include the acting model and thinking/provenance level in the message body when available.

After validation, commit and push implementation changes. Do not leave edited work uncommitted or unpushed. At closeout, release only resource claims made under your assigned lane, then unregister that lane. Do not release generic names or another worker's lane.

If the work creates or consumes a producer dependency, make that dependency portable before publishing. Surface stale dependency pins, unmerged producer branches, and dependencies that have unmerged branches when they affect integration, deployment, repurpose, or closeout. If portable closeout is not possible, report it as a hard blocker.

For primary-style direct main closeout:

```sh
jj status --no-pager
jj commit -m 'short imperative message'
jj bookmark set main -r @-
jj git push --bookmark main
```

Use BEADS/beads for tracked work that must survive the session or coordinate with other work. Close a bead only after acceptance criteria pass or the bead is invalidated; closing notes name durable evidence such as the commit, validation artifact, output file, or superseding task.

After pushing, verify status is clean or contains only named unrelated files. Report basis commit, bookmark, commands run, push result, and remaining blockers or disposition.
