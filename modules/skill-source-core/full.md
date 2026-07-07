# Module - skill source core

## Skill Source Core Purpose

Skill-system source edits keep instruction compact, current, and owned by the
generator inputs. Treat `LiGoldragon/skills` as the canonical skills source. The
reusable teaching body lives in source modules and role source modules; identity,
descriptions, tiers, targets, and dependency edges live in manifests or
dependency indexes. Workspace skill and agent files are generated runtime targets,
not source.

## Skill Source Prose

Write present-tense guidance that teaches one capability. Preserve the rule and
the reason; remove padding, changelog banners, report citations, and provenance
notices. Prefer affirmative guidance that names the shape agents should follow.

Role source starts with the role contract and stays mostly role prose. Shared
procedure belongs in modules so it is not copied into every role. Source modules
have no harness frontmatter.

Visible generated titles use the human title only. Keep composition labels such
as `Skill`, `Module`, or `Role` as source structure only; do not depend on those
labels appearing in generated runtime text.

Use `## Source Maintenance Notes` only for maintainer instructions that must stay
source-side. Everything from that heading through the end of the source fragment
is stripped from generated runtime surfaces.

Avoid absolute deployment paths in skill source. Prefer repository-root-relative
paths, or file-relative paths when the referenced file is local,
version-controlled, and stable.

Put required non-ideal workaround guidance in `NON_IDEAL_AGENTS.md` when a repo
needs it. Keep `AGENTS.md` for ordinary operating rules and `ARCHITECTURE.md` for
the ideal target shape; workaround instructions should read as debt and future
fix targets.

## Skill Source NOTA Manifests

Keep data in NOTA records, not comments. Use enum variants when a position can
carry more than one shape, and untagged structs when there is only one shape.
Use named enum variants rather than numeric codes. Preserve positional field
order and bare atoms for canonical strings.

The active output manifest lists emitted outputs. The dependency index maps
module identifiers to source paths and dependency module identifiers. Assemble
role packets from the active manifest and dependency index.

## Skill Source Reconciliation

After source edits, run the generator or check command when available. Treat a
skill edit as deployed only when the generated runtime surfaces active agents
read are reconciled; stale generated outputs are a deployment gap. Confirm that
every manifest or dependency path exists, new headings are unique within their
source file, generated runtime outputs would not receive provenance notices, and
role packets include the doctrine the manifest names without pulling the whole
corpus.

Dirty consuming workspaces do not block generation or required whole-working-copy
commits. Only direct ownership of a required generated path blocks deployment;
name the owner and path precisely.

After generation and checks pass, close out source edits with commit and push.
Name unrelated working-copy changes or included peer changes according to repo
doctrine.
