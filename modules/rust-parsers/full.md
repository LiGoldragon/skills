# Skill — Rust parsers (no hand-rolled)

If a format has a name, there's a parser library — use it. Hand-rolled
string slicing for named formats is forbidden.

## The rule

When a value arrives as bytes you didn't author — JSON from an external
tool, a config file in a foreign format, an API response, a record from
a partner system — use a real parser library, not hand-rolled string
slicing.

```rust
// Wrong — hand-rolled JSON field extraction (also a free function;
// the real fix is method-on-type per skills/rust/methods.md)
impl KeyMaterial {
    pub fn from_external_json(json_bytes: &[u8]) -> Result<Self, Error> {
        let text = std::str::from_utf8(json_bytes)?;
        let needle = "\"PrivateKey\"";
        let start = text.find(needle).ok_or(...)?;
        let after_key = &text[start + needle.len()..];
        let after_colon = after_key.find(':').ok_or(...)?;
        let after_open = after_key[after_colon + 1..].find('"').ok_or(...)?;
        let value_start = after_colon + 1 + after_open + 1;
        let close = after_key[value_start..].find('"').ok_or(...)?;
        Ok(Self::new(after_key[value_start..value_start + close].to_string()))
    }
}

// Right — serde_json owns the parse; the verb is a method on the
// noun being constructed
impl KeyMaterial {
    pub fn from_external_json(json_bytes: &[u8]) -> Result<Self, Error> {
        let value: serde_json::Value = serde_json::from_slice(json_bytes)?;
        let private_key = value
            .get("PrivateKey")
            .and_then(|field| field.as_str())
            .ok_or(...)?;
        Ok(Self::new(private_key.to_string()))
    }
}
```

The hand-rolled version looks "minimal" but is fragile: it depends on
field-order, can't see nested structure, breaks on escaped quotes, gives
bad error messages, and gets re-debugged forever as the external tool's
output evolves. The library version is two lines, structurally correct,
and stays correct as long as the input is valid JSON.

If the format has a name (JSON, TOML, YAML, XML, INI, CSV, base64, hex,
PEM, ASN.1/DER, MIME, HTTP, …), there is a parser library for it. The
cost of a dependency is paid once; the cost of a hand-rolled parser is
paid every time the input edges into a corner case the parser silently
mishandled.

## When no library exists

Two paths, in order of preference:

1. **Find one.** Search crates.io, lib.rs, or
   `nix run nixpkgs#cargo -- search <format>`. Most external formats
   already have a Rust parser, often several.
2. **Write a real one in its own crate.** If the format is genuinely
   novel, the parser is its own concern — its own crate, grammar, tests,
   and version pin (the classic micro-component; see
   `skills/micro-components.md`). It exposes a typed API
   (`MyFormat::parse(bytes) -> Result<Doc, Error>`), not a hand-rolled
   `find()`/`split()` chain inside a downstream consumer.

## The two carve-outs

Single-character splits and direct integer parses are not "hand-rolled
parsing" — they're trivial primitives:

- `text.split(',').map(str::trim)` for comma-separated lists with no
  escaping or nesting.
- `text.parse::<u64>()` and `u32::from_str_radix(text, 16)` for bare
  numbers.
- `text.lines()` for newline-delimited lists with no continuation rules.

If the input has any of: nesting, escapes, quoting, indentation
significance, optional whitespace, multi-character delimiters,
keyword-vs-identifier ambiguity, or "documented in an RFC" — it has a
real grammar. Use a parser.

The same rule applies in any language: a hand-rolled regex parser in
Python is the same anti-pattern; the substrate just makes the wrong
thing easier to write quickly.

Writing a JSON / TOML / YAML / PEM / DER / HTTP / URL parser from
string-slicing primitives is forbidden under any circumstances. If the
right library isn't on the dependency list, add it — the Nix-managed
toolchain (`skills/nix-discipline.md`) makes it one `Cargo.toml` line
plus a `cargo update`.

## See also

- `skills/rust-discipline.md` — Rust discipline index.
- `skills/micro-components.md` — when a parser deserves its own crate.
