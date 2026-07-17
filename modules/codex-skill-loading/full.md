# Module - Codex skill loading

## Skill-read de-duplication

A pasted `<skill ...>...</skill>` block is complete when it has matching opening
and closing `<skill>` tags, a skill name, a location, and non-empty body text.
Treat a complete pasted skill block as already loaded for this session. Read the
same skill location again only when the block is structurally missing content,
the user asks to verify source or freshness, or a higher-priority instruction
explicitly requires verification.
