# Module - Rust discipline

## Rust Discipline Purpose

Rust discipline gives code writers and auditors the baseline shape expected in
workspace Rust. It is role composition, not a runtime lookup.

## Rust Baseline

Every non-test behavior is a method on a non-zero-sized data-bearing type or a
trait implementation. Avoid free helpers except `main` and required test
wrappers.

Use domain types for domain values. A string, integer, or bool is not enough
when the value has a role in the model.

Crate boundaries return the crate's typed `Error` enum. Use `thiserror` or the
repo's existing explicit enum shape. Do not expose `anyhow` or `eyre` as the
boundary contract.

Keep names as full English words. Do not prefix types with the crate name.
Encode direction in names when a type crosses a boundary.
