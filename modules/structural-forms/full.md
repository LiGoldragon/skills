# Skill — structural forms

## Rules

Model language structure as data first. A form earns syntax only when the data shape, parse shape, and emitted shape are all clear.

Use positional structs only when field order is the meaning. Use named records when omission, reordering, defaults, or partial updates are part of the contract.

Add syntax dimensions only when they separate real axes of meaning. Do not spend delimiters on emphasis, decoration, or nostalgia.

Keep form vocabulary orthogonal: one delimiter or marker carries one meaning within a grammar layer.

When adding or pruning a form, update grammar, AST shape, examples, and round-trip tests together. A syntax rule without a data witness is incomplete.

Reject forms whose only justification is shorter prose; syntax protects structure.
