# Skill — structural forms

## Rules

Model language structure as data first. A form earns syntax only when the data
shape, parse shape, and emitted shape are all clear.

Use positional structs when field order is the meaning. Do not add labels just to
make a tuple feel safer; add labels only when order is not the primary contract.

Use named records when omission, reordering, defaults, or partial updates are
part of the contract. Do not force positional syntax onto unordered data.

Generics use pipe-parenthesis form when the parameters are type-level
participants in the form. Keep the generic list short enough that the reader can
see the base form first.

Traits and impl blocks use pipe-brace form when the body is a set of associated
items rather than data fields.

The dimensional principle: add a syntax dimension only when it separates a real
axis of meaning. Do not spend delimiters on emphasis, decoration, or historical
compatibility.

Streams and families are special forms, not ordinary structs. Preserve their
positional contract so the parser can distinguish repeated values, grouped
variants, and family membership without semantic lookahead.

Keep macro vocabulary orthogonal. One delimiter or marker should not mean both
"generic parameters" and "runtime list" in the same grammar layer.

The self-host boundary is a hard test: a form is stable only when the language
can describe that form without a parallel hand-written explanation.

When adding or pruning a form, update the grammar, AST shape, examples, and
round-trip tests together. A syntax rule without a data witness is not complete.

Delete tutorial examples once tests cover the form. Keep only the minimal example
that distinguishes the form from its nearest neighbor.

Reject forms whose only justification is that they make prose shorter; syntax
must protect structure, not save keystrokes.
