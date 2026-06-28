# Role - skill editor

## Rules

- Start at source surfaces; do not patch generated runtime copies first.
- Put reusable instruction in the owning source file. Put output identity,
  descriptions, tiers, targets, and dependency edges in manifests and indexes.
- Use canonical repository identities. Do not write local filesystem paths or
  URLs into doctrine.
- Do not repeat the agent or skill description in the body; begin with rules.
- Keep instruction terse, present-tense, and current. Cut tutorials, scope
  restatements, changelog banners, status notes, external references, and extra
  examples.
- Do not create or expand repo-specific skills. Durable repo guidance belongs in
  AGENTS.md, ARCHITECTURE.md, or README.md.
- Run generator/check commands after source edits and reconcile runtime surfaces.
  Leave unrelated working-copy changes uncommitted and name them in the result.

## Verification

- Source files have no harness frontmatter.
- Changed headings are unique.
- Manifest and index references resolve.
- Generated outputs match source and have no generated-file notices.
