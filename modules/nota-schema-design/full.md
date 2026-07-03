# Skill — NOTA schema design

## Rules

Schema specifies NOTA types, source syntax, and codec contracts. Raw NOTA parses first; schema lowering assigns type meaning after the structural parse succeeds.

Design one explicit type shape for each value shape. Use positional structs when there is one payload shape, and named enum variants when a position can carry multiple alternatives.

Struct fields are positional in authored schema source. Use `TypeName` when the field role derives from the type name, `role.TypeName` when the role differs, and `role.(Composite TypeName)` for parenthesized references such as `role.(Optional TypeName)`.

Use current reference heads such as `Vector`, `Map`, `Optional`, `ScopeOf`, and `(Bytes N)` according to the schema source grammar. Avoid retired pair forms and editor-tolerance aliases in authoritative schema.

Optional named struct fields are legal when absence differs from an empty value. Optional enum payloads and disappearing positional fields are wrong; use explicit variants or named optional fields instead.

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

Use an explicit role when the field role differs from the type:

```nota
VerbatimQuote {
  QuoteText
  optionalAntecedent.(Optional Antecedent)
}
```

Model optional variant payloads as explicit alternatives:

```nota
Decision [
  (Accepted Reason)
  (Rejected Reason)
  NeedsClarification
]
```

## Anti-Patterns

- mixing schema source truth with pseudo-NOTA documentation;
- encoding field names into positional values;
- using `(Optional T)` as an enum payload or positional field that can disappear;
- preserving retired pair syntax in new schema;
- hand-rendering help text outside the schema/codec projection.
