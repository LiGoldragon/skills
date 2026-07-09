# Skill — NOTA shape checklist

## Rules for Shape

Start from the expected type; it is always known at a correct NOTA boundary. The file kind, schema field, operation argument, reply slot, test fixture, or prompt-supplied schema tells the decoder what type to read.

Write exactly the value of the expected type. The known document, record, object, or application shape fixes slot count: no extra slots, no missing slots, and no omitted positional optionals. Optionality is typed data in a known position.

Meaning comes from expected type plus position. A use-site name is data, a schema-required disambiguator, or a reference/path/name value under the expected type; it is never a label that identifies a positional slot.

A leading atom is valid only when the expected position is an enum and that atom exactly matches one of its variants. Run the variant-sibling test on every leading atom: name the other variants valid at this exact position. If none exist, the atom is not a tag; move the idea into the schema field, a typed enum value, or remove it.

Choose cardinality before syntax. A closed exactly-one-per-slot set is a positional record. Use a vector only for homogeneous repeatable elements where order or duplicates are meaningful, or where validation rejects duplicates. Do not encode fixed slots as tagged rows in a list.

Records are positional. Emit field values in schema order; do not put field labels in the value. Treat `Vector Vector`, same-name self-labeling, and `Name Value` adjacency as design alarms for self-labeling instead of typed positional data.

Use maps only for real key/value domains: arbitrary keys, lookup by key, and key identity as data. A value is a map because the expected type is a map, not because labels feel readable.

Prefer closed enums and typed records over strings. A bare atom is valid as a string when the expected type is `String`; capitalization does not infer type state. Enum slots decode by exact variant match.

Before accepting a shape, state the expected type, sibling variants for each tag, cardinality for each collection, and duplicate/order semantics for each vector. If any part is unknown, pause and ask; do not bury uncertainty in a special parser, ad hoc labels, or JSON-like shape.
