# Skill — Rust discipline (index)

Entry point for writing Rust in this workspace: behavior lives on
types, domain values are typed, boundaries take and return one
object, errors are hand-written enums.

## The rules in one sentence

Every function is a method on a non-zero-sized data-bearing type or
a trait impl. Domain values are typed. Boundaries take and return
one object. Errors are enums you implement by hand.

## Sub-files — read before authoring

The substance lives in five focused sub-files under `skills/rust/`.
Read the relevant one(s) at the authoring moment.

| Sub-file | Covers |
|---|---|
| `skills/rust/methods.md` | methods on non-zero-sized data-bearing types only; no free functions outside `fn main()` / `#[cfg(test)]`; no ZST namespace holders; domain newtypes, one-type-per-concept, no string typification, one-object-in/out, constructors, trait domains, direction-encoded names |
| `skills/rust/errors.md` | typed `Error` enum per crate via `thiserror` |
| `skills/rust/storage-and-wire.md` | redb + rkyv durable state and binary wire (signaling, NOTA projection, sema-family) |
| `skills/rust/parsers.md` | no hand-rolled parsers; use a real library |
| `skills/rust/crate-layout.md` | CLIs as daemon clients, one crate per repo, tests in separate files, module layout, documentation |

## Toolchain authority

The interactive Rust toolchain is
`CriomOS-home.packages.<system>.rust-toolchain` (defined at
CriomOS-home's `packages/rust-toolchain/default.nix`, pinned by its
`flake.lock`). It provides `cargo`, `rustfmt`/`cargo fmt`, `clippy`,
`rust-analyzer`, and `rust-src`. User profiles install that package,
not bare `pkgs.cargo`/`pkgs.rustc`/`pkgs.rustfmt`. A Rust app repo
may pin its own build toolchain through its flake when
reproducibility requires it; that build pin is not the profile
toolchain authority.

## Target directory hygiene

Do not delete whole `target/` directories as the normal cleanup path:
that throws away the newest useful local build artifacts. Prefer
`cargo-sweep` with a per-repo size cap, which removes older Cargo
artifacts while preserving the most recent builds:

```sh
nix run github:NixOS/nixpkgs/nixpkgs-unstable#cargo-sweep -- sweep --recursive --maxsize 4GB /git/github.com/LiGoldragon
```

Use `--dry-run` first when deciding a new cap or scope. For a single
large active repo, run from that repo and choose a larger cap when
needed, for example:

```sh
nix run github:NixOS/nixpkgs/nixpkgs-unstable#cargo-sweep -- sweep --dry-run --maxsize 8GB .
```

The durable Nix-side prevention is still the shared `rust-build`
source cleaner: Rust flakes should use `rust-build.cleanSource` rather
than raw `craneLib.filterCargoSources` / `cleanCargoSource`, so Nix
source construction prunes `target`, `.git`, `.jj`, `.direnv`, and
`node_modules` before Crane or repo-specific filters run.

## Naming — full English words

Every name reads as an English word; `self` stays as the implicit
receiver. The cross-language rule, offender table, and permitted
exceptions live in `skills/naming.md`.

```rust
// Wrong — cryptic in-group dialect
let mut lex = Lexer::new(input);
let tok = lex.next_tok()?;
let ctx = ParseCtx::new(&tok.kind());

// Right — every name reads as English
let mut lexer = Lexer::new(input);
let token = lexer.next_token()?;
let context = ParseContext::new(&token.kind());
```

### No crate-name prefix on types

Never prefix a type with its crate name (the Rust API Guidelines
call this **C-CRATE-PREFIX**). The standard library is the model:
`Vec`, `HashMap`, `Arc`, `Mutex` — never `StdVec`, `StdArc`.
Workspace pattern: `signal::Request`, `chroma::Error`; never
`SignalRequest` or `ChromaError`.

## Actors: logical units with kameo

When a Rust component is a daemon, state engine, router, watcher,
delivery engine, database owner, or long-lived service, follow
`skills/actor-systems.md` and `skills/kameo.md`. Reach for an actor
for **logical cohesion**, not performance: it is the unit for a
coherent plane of logic with owned state, a typed message protocol,
and a defined lifecycle. Rust-side enforcement:

- Actor type carries data fields (Kameo's `Self IS the actor`); no
  public ZST actor nouns.
- One `impl Message<Verb> for Actor` per verb; no monolithic `Msg`
  enum, no untyped channels.
- One actor per file when the actor is durable enough to name.
- Handlers do not block. Use `DelegatedReply<R>` or a dedicated
  blocking-plane actor.
- Never `tell` a handler whose `Reply = Result<_, _>` unless
  `on_panic` is overridden.
- No `Arc<Mutex<T>>` between actors — send a message to whoever owns
  the state.
- Errors at component boundaries are the crate's typed `Error` enum,
  never `anyhow`/`eyre`.
- The default public consumer surface is `ActorRef<MyActor>`; domain
  wrappers earn their place per `skills/kameo.md`.

Plain sync code is fine for stateless one-shot CLIs, build tools,
and library crates with no concurrent state. A CLI that needs
durable state, supervision, subscriptions, or shared runtime context
is a daemon client (see `skills/rust/crate-layout.md`).

## See also

- `skills/abstractions.md` — cross-language methods-on-types rule.
- `skills/enum-contact-points.md` — where two enums meet under
  `match`, name the contact point instead of scattering the matrix
  across `if` chains and string predicates.
- `skills/kameo.md` — Kameo framework usage.
