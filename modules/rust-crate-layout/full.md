# Skill — Rust crate layout

## CLIs are daemon clients

A CLI in this workspace is a client. When a tool needs durable state,
supervision, subscriptions, long-lived actors, or shared runtime
context, that state lives in a daemon and the CLI talks to it. The
"one-shot CLI owns the runtime" shape is not an option unless the user
explicitly asks to break this rule.

- the daemon owns the root actor, durable database, subscriptions, and
  runtime lifecycle;
- the CLI parses one input object, sends a typed request, waits for one
  typed reply, renders it, and exits;
- tests may use in-process harnesses for speed, but production
  architecture stays daemon-first.

Every non-contract stateful component or daemon exposes a thin CLI
control surface, even when it is not user-facing — it is the test and
operations boundary. It does not own durable state, open the component
database directly, or bypass the daemon's actor/message path.

Read-only inspection CLIs are the narrow exception: a component may ship
an explicitly named inspection client that opens the component's Sema
database to render artifacts or operational state. It must not mutate
state, allocate identity, drive effects, or become the production
request path; effect-bearing commands still go through the daemon.

Contract crates are also excepted — they are libraries of typed wire
vocabulary. Their tests are round-trip, schema, and compile-time
witnesses; they need a CLI only if they deliberately ship a generator
or inspection tool.

Example: `mind` is a thin client to the long-lived `persona-mind`
daemon. The daemon owns `MindRoot` and `mind.redb`; the CLI owns
argv/env decoding and reply rendering.

## One Rust crate per repo

Rust crates live in their own dedicated repos and are consumed via flake
inputs. Don't inline a Rust crate inside a non-Rust repo (e.g. under a
NixOS-platform repo's `packages/`). A Rust crate has its own toolchain
pin, Cargo lockfile, test surface, release cadence, and style
obligations; inlining one inside a heterogeneous repo couples those
concerns to the host repo's churn for no gain.

A workspace of related Rust crates (e.g. lib + cli) belongs in **one**
repo together. The split is per *project*, not per crate.

Cross-crate `Cargo.toml` deps use `git = "..."`, never `path =
"../..."`. A `path` dependency on a sibling repo makes the repo
non-portable: fresh clones don't reproduce, `Cargo.lock` doesn't pin the
rev, and `nix flake check` can't fetch through the sandbox.

See `skills/micro-components.md` for the `Cargo.toml` dependency rule and
lore's `rust/style.md` for toolchain conventions and pin strategy.

## Tests live in separate files

Unit tests do **not** go in a `#[cfg(test)] mod tests` block at the
bottom of the source file. They live in a sibling file under `tests/` at
the crate root, named for the module they exercise.

```
src/
├── cert.rs
├── tree.rs
└── error.rs
tests/
├── cert.rs      # integration tests for Cert
└── tree.rs      # integration tests for Tree
```

This keeps the source file focused on behavior, lets the test file grow
without bloating the source, and forces tests through the public API.
Integration tests can't reach private items — the right pressure: if
something is hard to test from outside, the API needs work, not the
test. Private-helper tests are rare; put them in a small
`tests_internal` module with a clear boundary. Reaching for many is a
signal the helper wants to be its own type with a public constructor.

One test file per source file. Don't collect tests from multiple modules
into a single `tests/common.rs` unless the shared fixtures genuinely
apply to more than one module.

## Module layout

One concern per file. Typical crate:

```
src/
├── lib.rs        # re-exports + crate-level doc (//!)
├── error.rs      # Error enum + impls
├── types.rs      # domain newtypes + small structs
├── <thing>.rs    # one file per major type / subsystem
└── main.rs       # binary crates only; contains only fn main()
```

Impls live in the same file as the type they're for. Don't split a type
and its impls across files.

### Split traits into their own files when they accumulate

When a file grows past ~300 lines because traits have piled up on a
type, split each trait impl into its own file. The type's file holds the
definition plus inherent impls; each separate file holds one trait impl,
named for the trait.

```
src/cert/
├── mod.rs              # type definition + inherent impls (Cert::new, fields)
├── from_str.rs         # impl FromStr for Cert
├── display.rs          # impl Display for Cert
├── try_from_pem.rs     # impl TryFrom<Pem> for Cert
└── serde_impls.rs      # impl Serialize + Deserialize for Cert (paired traits)
```

Explicit code is fine; long files are not. Splitting keeps any single
file readable and makes the type's surface discoverable from the
directory listing. Don't pre-split a type with two trait impls — split
when a file is becoming hard to navigate.

## Documentation

Doc comments are impersonal, timeless, precise. Document the contract;
don't restate the signature.

```rust
impl Cert {
    /// Issue a server certificate against this CA.
    ///
    /// The CA's signing key must be an Ed25519 key resolvable via the
    /// local GPG agent. The server keypair is ECDSA P-256, generated fresh.
    pub fn issue_server(&self, request: ServerCertRequest) -> Result<Self, Error> { … }
}
```

Module-level docs go in `//!` at the top of `lib.rs`, or `///` at the top
of a single-purpose module file. Skip docs on obvious boilerplate:
getters, `From` impls, internal helpers. No examples unless the API is
non-obvious. Present indicative only — no personal voice, no future
tense.

## See also

- `skills/rust-discipline.md` — Rust discipline index.
- `skills/rust/methods.md` — what goes inside the source files.
- `skills/micro-components.md` — one capability per crate per repo.
