# Module - skill source core

## Skill Source Core Purpose

Skill-system source edits keep instruction compact, current, and owned by the
generator inputs. The reusable teaching body lives in source modules and role
source modules; identity, descriptions, tiers, targets, and dependency edges
live in manifests or dependency indexes.

## Skill Source Prose

Write present-tense guidance that teaches one capability. Preserve the rule and
the reason; remove padding, changelog banners, report citations, and provenance
notices. Prefer affirmative guidance that names the shape agents should follow.

Role source starts with the role contract and stays mostly role prose. Shared
procedure belongs in modules so it is not copied into every role. Source modules
have no harness frontmatter.

## Skill Source NOTA Manifests

Keep data in NOTA records, not comments. Use enum variants when a position can
carry more than one shape, and untagged structs when there is only one shape.
Use named enum variants rather than numeric codes. Preserve positional field
order and bare atoms for canonical strings.

The active output manifest lists emitted outputs. The dependency index maps
module identifiers to source paths and dependency module identifiers. Assemble
role packets from the active manifest and dependency index.

## Skill Source Reconciliation

After source edits, run the generator or check command when available. Confirm
that every manifest or dependency path exists, new headings are unique within
their source file, generated runtime outputs would not receive provenance
notices, and role packets include the doctrine the manifest names without
pulling the whole corpus.
