# Skill — language-design instincts

## Rules

Use this discipline before creating or changing a human-authored notation,
request surface, schema tree, or query language.

Do not create a new text syntax. Human-facing and harness-facing text uses the
canonical record syntax. Add typed records, verbs, or sub-vocabularies inside the
existing syntax instead of inventing a new language.

Syntax is delimiter-first. A construct starts with enough visible structure for
the parser to know what it is reading without fallback rules, interior scans, or
multi-token guesses.

Position defines meaning. Dispatch on delimiter, position, and record head, not
reserved words or ambient schema state.

Keep the keyword budget closed. New concepts land as typed records, enum
variants, or schema positions, not new parser keywords or sigils.

Use case to carry kind when the syntax depends on it: PascalCase for types,
variants, and structural names; camelCase for fields and local instances.

Every stored value is structured. A string that controls behavior is a missing
type; replace it with a closed enum, typed identifier, or record tree.

Whitespace is only separation. Newlines and indentation must not change parse
meaning.

Records on the wire are positional. Field names live in the schema. Reordering
fields changes the wire; trailing optionals may be accepted for compatibility,
but canonical encoders emit the explicit shape.

Canonical encoding owns durable identity. Hash the canonical bytes when a value
needs immutable identity; mutable handles sit above that identity.

Parser rules stay rare. Schema growth is the normal extension path; parser growth
needs a structural shape records and sequences cannot express.

Delimiters earn their place. Do not spend a delimiter pair on cosmetic
distinctions such as set versus list when the receiving type already carries the
meaning.

Names describe roles. Avoid placeholder names for types, fields, parameters, and
locals. Distinct roles get distinct names even when their representation matches.

Use named fields for multi-value products. Single-field newtypes are fine;
multi-field unnamed tuples hide roles and make call sites ambiguous.

Mutation is marked at the declaration site. Readers should not scan a body to
learn whether a value can change.

Domains come from declarative data. Do not hand-maintain enum lists, dispatch
tables, or duplicated variant knowledge.

Binary means bytes. Hex, base64, and JSON arrays are projections, never the
canonical binary protocol.

Every pipeline component declares its inputs and outputs. No context bag,
passthrough metadata, or untyped side channel substitutes for a named type.

When a limitation appears, extend the language properly. Raw-text passthroughs,
partial parsers, and "temporary" escape hatches break round trips and spread.
