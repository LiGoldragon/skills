# Skill — NOTA schema docs

## Rules

Document a NOTA record with a pseudo-NOTA form when a reader needs field names without reading Rust schema source.

Use angle-bracket placeholders and `?` for optional fields:

```nota
;; Bug: concrete defect.
(Bug <title> <description> <severity> <incident-at?> <reproduction?>)
;;   title        : ShortText
;;   description  : TextBody
;;   severity     : Catastrophic | High | Normal | Low
;;   incident-at? : Timestamp
;;   reproduction?: TextBody
```

Rules:

- placeholders use spelled-out kebab-case;
- optional fields end in `?` in the form and field list;
- closed enums use `|` between variants;
- lists wrap the inner type in square brackets;
- nested records use the same form;
- `;;` lines are NOTA comments and may name field types or non-obvious constraints.

Pseudo-NOTA is documentation, not the authoritative wire shape. The schema and round-trip examples own truth.
