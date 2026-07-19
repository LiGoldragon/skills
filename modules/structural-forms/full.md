# Skill — structural forms

## Rules

Model language structure as data first. A form earns syntax only when the data shape, parse shape, and emitted shape are all clear.

For NOTA and schema surfaces, positionality is the golden rule. Expected type plus position identifies every field, argument, and variant payload; use-site names are data, never slot labels.

Use closed positional forms when shape and count are fixed. If omission, reordering, defaults, or partial updates are real operations, model them as separate typed forms instead of adding keyword binding to a positional call.

Add syntax dimensions only when they separate real axes of meaning. Do not spend delimiters on emphasis, decoration, or nostalgia. Treat self-labeling adjacency such as `Name Value` or repeated heads such as `Vector Vector` as a design alarm.

Keep form vocabulary orthogonal: one delimiter or marker carries one meaning within a grammar layer. Capitalization is not runtime type inference; the expected type decides the value category.

When adding or pruning a form, update grammar, AST shape, examples, and round-trip tests together. A syntax rule without a data witness is incomplete.

Reject forms whose only justification is shorter prose; syntax protects structure.
