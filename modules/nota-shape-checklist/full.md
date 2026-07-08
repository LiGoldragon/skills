# Skill — NOTA shape checklist

## Rules for Shape

Start from the expected type; it is always known at a correct NOTA boundary. The file kind, schema field, operation argument, reply slot, test fixture, or prompt-supplied schema tells the decoder what type to read.

Write the value of the expected type. Do not prefix a value with its own type name. A leading atom is valid only when the expected position is an enum and that atom is one of its variants.

Run the variant-sibling test on every leading atom: name the other variants valid at this exact position. If none exist, the atom is not a tag; move the idea into the schema field, a typed enum value, or remove it.

Choose cardinality before syntax. A closed exactly-one-per-slot set is a positional record. Use a vector only for homogeneous repeatable elements where order or duplicates are meaningful, or where validation rejects duplicates. Do not encode fixed slots as tagged rows in a list.

Records are positional. Emit field values in schema order; do not put field labels in the value.

Use maps only for real key/value domains: arbitrary keys, lookup by key, and key identity as data. Do not use a map because labels feel readable.

Prefer closed enums and typed records over strings. A bare atom is valid only as a real enum variant, stable identifier, or canonical atom under a typed field; it is not a field label.

Before accepting a shape, state the expected type, sibling variants for each tag, cardinality for each collection, and duplicate/order semantics for each vector. If any part is unknown, pause and ask; do not bury uncertainty in a special parser, ad hoc labels, or JSON-like shape.
