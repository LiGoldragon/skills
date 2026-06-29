# Skill — typed records over flags

When the system asks a yes/no question of a noun, ask whether the "yes"
carries data. If it does, the question wants a typed record, not a
boolean.

## The rule

**Boolean-on-a-noun is a code smell when the "yes" branch carries
data.** Replace `field: bool` with `field: Option<Record>` — the data
the "yes" carries becomes the record's payload. Readers go from
`if node.field` to `if let Some(record) = &node.field`.

The same generalises to enums: a unit-variant enum whose variants mean
more than their names is asking to carry data, or to become a struct of
`Option<T>`s.

## Why

A boolean field is a question with a hidden answer. `is_nix_cache: bool`
asks "is this node a Nix cache?" — but if yes, the consumer still needs
the endpoint, the signing key, the retention policy. In a boolean world,
every consumer reinvents that lookup with its own ad-hoc derivation
(magic-string URLs, hardcoded key paths). The derivation rules diverge,
the type system stops catching errors, and a node that should be a cache
but lacks a signing key type-checks fine — then fails at runtime.

When the boolean becomes `binary_cache: Option<BinaryCache>` and
`BinaryCache` carries `endpoint`, `signing_key`, `retention_policy`,
every property is in the type and every consumer reads it the same way.
Adding a property is one struct field; removing one breaks every consumer
that read it. The type **is** the contract, and an incomplete record
fails validation at proposal time, not deploy time.

## The three forms

Pick whichever fits.

### Form 1 — `Option<Record>` on a single noun

```rust
pub struct Node {
    pub binary_cache: Option<BinaryCache>,
}

pub struct BinaryCache {
    pub endpoint: BinaryCacheEndpoint,    // scheme, host, port, public_key
    pub signing_key: SecretReference,
    pub retention_policy: CacheRetentionPolicy,
}
```

Use when a node either is or isn't this thing; if it is, the record
carries the configuration. The capability sub-record is the default home.

### Form 2 — sum enum with data variants

```rust
pub enum WifiAuthentication {
    Wpa3Sae { password: SecretReference },
    EapTls { profile: CertificateProfileId },
}
```

Use when the noun is in one of several mutually-exclusive states, each
carrying its own data. A `eap_tls: bool` paired with a `wpa3_sae: bool`
is wrong: the values are exclusive and each carries different config. The
sum-with-data names both the exclusion and the per-variant payload.

### Form 3 — typed record replacing a multi-flag struct

```rust
pub enum NodePlacement {
    Metal(MetalPlacement),
    Contained(ContainedPlacement),
}
```

Use when several booleans are obviously a single closed-set choice
wearing a struct disguise. A `behaves_as { virtual_machine: bool,
bare_metal: bool, iso: bool }` triplet is one enum with three variants:
the struct form let `(true, true, false)` type-check though it was
illegal; the enum form makes the illegal state unrepresentable.

## What stays a boolean

Booleans whose "yes" branch carries no payload data are fine:

- `online: bool` — yes-or-no, no payload. The node is up or it isn't.
- `wants_printing: bool` — operator opt-in; the payload (the printer
  bundle) lives in the module that gates on the flag.

The diagnostic: if a `bool`'s value lets you derive the payload trivially
(`if x { default() }`), it can stay. If the payload requires authored
data (endpoints, keys, policies, references), the boolean is hiding a
record.

## Migration shape

1. **Add the typed record alongside the boolean** for a transition cycle.
2. **Derive the record from existing inputs in projection.** New
   proposals author the record directly; old proposals get a shimmed
   default.
3. **Migrate consumers one at a time**, each from `if node.flag` to a
   match on the record. The boolean's derivation can switch to
   `record.is_some()` once readers have migrated.
4. **Delete the boolean** once nothing reads it; the original flag-bundle
   struct shrinks and eventually disappears.
