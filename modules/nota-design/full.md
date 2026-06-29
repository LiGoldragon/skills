# Skill — NOTA design

## Rules

NOTA is structural data. The raw grammar has atoms, parenthesized records, vectors, maps, pipe text, pipe parenthesis, pipe brace, and `;;` comments. Schema and codec layers assign meaning.

Records are positional. Field order is part of the interface; reordering fields is a compatibility change. Prefer a trailing field or a new variant over changing existing positions.

Use an untagged struct when there is one payload shape. Use an enum only when a position can hold multiple named variants. Enum variants use names, not numeric codes.

Use bare atoms for stable identifiers, enum-like values, and canonical names. Use pipe text or quoted/bracket string forms when whitespace, punctuation, comments, or arbitrary prose are the point.

Put machine data in records, not comments. Comments explain unusual choices; they do not carry values that must be read, queried, validated, or migrated.

Model alternatives as variants or named option variants, not loose flags. A variant carries only the fields that choice needs.

Use maps only for genuinely keyed collections. Do not use a map to avoid naming a record shape.

Avoid multi-field unnamed tuples. If there is more than one value, name the record or fields in the schema so the positional call site stays readable.

Use optional values only when absence has meaning distinct from empty. Prefer a variant when absence changes behavior.

Use generated codecs for regular records and enums. Hand-write a codec only for compatibility boundaries, legacy syntax, or grammar the schema cannot express cleanly.
