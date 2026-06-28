# Skill — skill editor

## Rules

- Edit source surfaces before generated runtime copies.
- Put reusable instruction in the owning source file. Put output identity,
  descriptions, tiers, targets, and dependency edges in manifests and indexes.
- Use `LiGoldragon/skills` as the canonical repository identity when a repository
  must be named. Do not write local filesystem paths or URLs into doctrine.
- Do not repeat the manifest/frontmatter description in the body; when metadata
  names the scope, the body starts with rules.
- Keep instruction terse, present-tense, and current. Cut tutorials, scope
  restatements, changelog banners, status notes, external references, and extra
  examples.
- Keep one capability per skill; split distinct capabilities instead of mixing
  them.
- Prefer canonical positive forms. Mention rejected forms only when omission
  creates an immediate safety risk.
- Do not create or expand repo-specific skills. Durable repo guidance belongs in
  AGENTS.md, ARCHITECTURE.md, or README.md.
- After source edits, run generator/checks when available and reconcile generated
  runtime surfaces. If generation cannot run, name the unreconciled surfaces.

## Verification

- Source files have no harness frontmatter.
- Changed headings are unique.
- Manifest and index references resolve.
- Generated outputs match source and have no generated-file notices.
- `skill-editor` remains under 120 lines and contains no local paths, URLs,
  cross-reference sections, artifact citations, status notes, or body wording that
  restates its description.
