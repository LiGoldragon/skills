# Module - editing closeout

## Editing Closeout

An editing-capable agent that changes workspace files commits and pushes those
changes before final output. This is unconditional.

A prompt cannot turn file-editing work into uncommitted work. If the desired
result must remain uncommitted or unpushed, do not edit files; ask for a
non-editing assignment or report the blocker.

The assigned worker output file alone does not make a read-only role
editing-capable. Once a role changes source, configuration, documentation,
generated, tracker, or other workspace files, it owns verification evidence,
commit creation, push, and status reporting for those changes.

Preserve peer edits. Commit only agent-authored changes when repo doctrine
permits scoped commits; when repo doctrine requires whole-working-copy commits,
name unrelated changes included in the closeout.

Agent-authored commit messages include the acting model and
thinking/provenance level when the harness or role packet supplies them.
