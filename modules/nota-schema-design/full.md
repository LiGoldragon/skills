# Skill — NOTA schema design

## Rules

Schema specifies NOTA types, source syntax, and codec contracts. Raw NOTA parses first; schema lowering assigns type meaning after the structural parse succeeds.

At every correct schema boundary, the expected type is already known. Schema, help, examples, and codecs must not rely on parser guessing, capitalization, or per-call labels to decide value category.

Design one explicit type shape for each value shape. Use positional structs when there is one payload shape, and named enum variants when a position can carry multiple alternatives. The known shape fixes slot count: no extra slots, no missing slots, and no disappearing positional optionals.

Struct fields, arguments, generic parameters, and variant payloads are positional. Field names in schema identify positions for authors and generated help; values never bind by field name, keyword argument, or named generic argument. Multi-parameter generics apply positionally.

Use closed typed variants and meta-types for generic definitions. Do not force distinct cases into one parameter soup, and do not create kinds merely by arity. Put arity in delimited payload data when it is real data.

Target schema design uses dotted carrying/application syntax: `Head.Payload`, `Head.(...)`, and data-carrying variants as `Variant.Payload`. When editing deployed schema source, use the grammar it accepts until the target lands; do not present legacy parenthesized applications or named-brace generic binding as the design goal.

Use current reference heads such as `Vector`, `Map`, `Optional`, `ScopeOf`, and `(Bytes N)` according to the deployed schema source grammar. Structural lowering uses the generic definition or meta-type, not hard-coded name tests for `Vector`, `Map`, `Optional`, or any other generic. Name-specific editorial projection is legitimate only when definition data or kind defaults carry it.

Represent optionality as typed data in a known position. Optional enum payloads, disappearing fields, and omitted slots are wrong; use explicit variants, option records, or required sentinel shapes.

Maps are known by expected map type. Prefer atom keys unless the map type requires a richer key; if dotted map-entry syntax is used, split at the first top-level dot.

Keep pseudo-NOTA docs separate from schema truth. Pseudo-NOTA may help humans read field names in markdown, but schema source, generated help, and round-trip examples own the contract.

Prefer canonical schema, codec, source, and help projection APIs over hand parsing or rendering. Do not create parallel per-type parsers, printers, or help languages.

When authoring prompts for models that must answer in NOTA, include the relevant schema/help projection or concrete examples in the prompt. Do not rely on the model calling a help tool during the API-like turn.

For judge-style prompts, provide an explicit diagnostic option when ambiguity should be explainable. The diagnostic branch may allow ordinary prose; normal NOTA replies stay expression-only unless the schema says otherwise.

## Examples

Use positional field references in schema source:

```nota
Entry {
  Domains
  Kind
  Description
  Certainty
  Importance
  Privacy
  Referents
}
```

Use target dotted carrying syntax for a role-specific field:

```nota
VerbatimQuote {
  QuoteText
  optionalAntecedent.Optional.Antecedent
}
```

Model optional variant payloads as explicit alternatives:

```nota
Decision [
  Accepted.Reason
  Rejected.Reason
  NeedsClarification
]
```

## Anti-Patterns

- mixing schema source truth with pseudo-NOTA documentation;
- encoding field names into positional values;
- binding fields or generic arguments by name at a use site;
- using optionality to make an enum payload, field, or positional slot disappear;
- hard-coding generic names instead of using generic definitions or meta-types;
- normalizing self-labeling forms such as `Vector Vector` or `Name Value`;
- presenting legacy parenthesized applications or named-brace binding as new schema design;
- hand-rendering help text outside the schema/codec projection.
