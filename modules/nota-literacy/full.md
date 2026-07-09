# Skill — NOTA literacy

## Rules

NOTA is strict structured communication. The schema, help projection, or concrete examples supplied in the prompt give the expression its expected type and meaning.

Use the provided schema/help projection and examples as authoritative. For API-like calls, do not assume a runtime help tool is available; answer from the prompt's supplied contract.

Reply with only the requested NOTA expression unless the prompt explicitly provides a diagnostic or prose escape hatch. If such a hatch exists and the contract is unclear, use it to say what is unclear and what should be improved.

Parenthesized records and variants are positional. Field order matters, values do not carry field names, and every positional component appears in the expression. No extra slots, no missing slots.

Read field names in schema/help text as position labels, not as keys to emit. Do not turn records into maps unless the expected type is a map.

Use bare atoms for canonical strings, variants, identities, and stable names when the value is valid as an atom. Capitalization does not infer type state: a bare capitalized atom may be a string when the expected type is `String`, and enum slots decode by exact variant match.

Represent optionality exactly as typed data in the supplied shape. Do not omit positional fields; choose the explicit variant, option record, or required sentinel shape the schema provides.

Do not wrap the answer in markdown fences, JSON, YAML, comments, explanations, or surrounding prose. Do not invent double-quoted strings, field names, maps, or alternate delimiters.

Treat pseudo-NOTA documentation as a reader aid. Concrete schema/help projections and round-trip examples own the response shape.

## Examples

If the prompt says `Entry` is `{ Domains Kind Description Certainty Importance Privacy Referents }`, emit values in that order:

```nota
{ [(Technology All)] Principle [|Use the canonical codec.|] High High Zero [codec] }
```

If the prompt asks for `(Decision <kind> <reason>)`, emit the record without labels:

```nota
(Decision Accept [|The evidence satisfies the requested rule.|])
```

## Anti-Patterns

- inferring meaning from delimiters alone;
- changing positional records into keyed maps;
- omitting fields because they are optional in ordinary prose;
- treating pseudo-NOTA placeholders as wire truth;
- putting machine-readable data in comments;
- returning opaque `MeaningUnclear` when the prompt permits a diagnostic explanation.
