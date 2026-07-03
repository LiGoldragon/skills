# Module - agent output artifact

## Rules

Use only when the brief explicitly requests a worker output artifact for
downstream pickup or fresh-context handoff. Otherwise return normal completion
in chat or the harness-required worker output.

If the brief names an exact path, use that path. Otherwise write under:

```txt
agent-outputs/<SessionName>/
```

`<SessionName>` is CamelCase and names the active weave, investigation, or
handoff. Use the supplied session name; if none is supplied, derive one from the
work title and keep it stable for the thread.

Name files:

```txt
<RoleLabel>-<ArtifactName>.md
```

`<RoleLabel>` is the role name in PascalCase without spaces, such as
`Scout`, `SkillEditor`, or `RustAuditor`. `<ArtifactName>` is a short PascalCase
description of the output, such as `SituationalMap`, `Evidence`, or
`Review`.

Start with a title naming the artifact. Include the context a fresh agent needs
to use the file without reading the chat transcript:

- task and scope;
- files or commands consulted;
- observed facts separated from interpretations where discovery is involved;
- changed files or proposed changes where implementation is involved;
- checks run and exact result;
- blockers, unknowns, and follow-up requirements.

Do not include generated-file notices in runtime agent outputs. Do not include
secrets, private personal material, or auth tokens.

After writing the artifact, return its path plus any status the brief or harness
requires.
