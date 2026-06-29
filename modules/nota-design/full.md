# Skill — NOTA design

## Use structs when there is one shape

A record with one payload shape is an untagged struct. Add an enum only when the position can hold multiple named variants.

## Put data in records

Comments explain unusual choices; they do not carry machine data. If a value must be read, queried, validated, or migrated, give it a field in a record.

## Name enum variants

Enum variants are PascalCase names, not numeric codes. A variant payload is the data for that choice. A struct is a product of fields that always belong together.

## Preserve positional meaning

NOTA records are positional. Field order is part of the interface. Reordering fields is a compatibility change. Prefer adding a new variant or a new trailing field over changing existing positions.

## Use bare atoms for canonical strings

Use bare atoms for stable identifiers, enum-like values, and canonical names. Use quoted or block strings only when whitespace, punctuation, or arbitrary prose is the point.

## Model options as option variants

Do not grow loose flags. If a record accepts alternative options, make the options a vector of named option variants. Each variant carries only the fields it needs.

## Keep maps explicit

Use maps for genuinely keyed collections. Do not use a map to avoid naming a record shape. Schema namespaces use brace-map form when the grammar requires a keyed namespace.

## Avoid tuples

Multi-field unnamed structs are forbidden. If there is more than one value, name the record or fields so the position is readable at the call site.

## Optional means absent is meaningful

Use optional values only when absence has semantics distinct from an empty value. Prefer a variant when absence changes behavior.

## Derive when the grammar is regular

Use generated codecs for ordinary positional records and enums. Hand-write a codec only for compatibility boundaries, legacy syntax, or a grammar the schema cannot express cleanly.

## Sketch before encoding

Before writing a record, name the stable noun, list its positions in order, identify which positions are enums, and decide which strings are canonical atoms.
