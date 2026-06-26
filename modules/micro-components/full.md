# Skill — micro-components (one capability, one crate, one repo)

Each functional capability lives in its own repo, sized so the whole
component fits in a single LLM context window.

## When to apply

At the moment of "should I add this here, or start a new crate?" That
is where bundling decay begins. The default answer is **a new repo**,
not a new module in an existing crate. A `Cargo.toml`, a `flake.nix`,
and a few minutes of plumbing are paid once; bundling costs months of
future friction.

## The shape

Every capability — state engine, code emitter, executor, store,
parser, schema, transport — lives in an independent repository with
its own `Cargo.toml`, `flake.nix`, and tests. Components talk only
through typed protocols, never shared mutable state. Each is sized so
the *entire component including tests* fits comfortably in one LLM
context window.

This is **source organization, not deployment**: components may
compile into one binary, many binaries, or talk over a network. The
workspace is the assemblage; no component knows more than its
protocol-typed neighbors. The boundary is filesystem-enforced —
module boundaries inside one crate decay under deadline pressure into
shared internals (the "modular monolith" failure mode).

## The rules

1. **One capability, one crate, one repo.** If you can name the new
   functionality with a noun, it gets its own `Cargo.toml` and git
   history — never a new `mod` in an existing crate.

2. **A component fits in a single LLM context window.** A crate of
   ~3k–10k lines (~30k–80k tokens including tests) can be reasoned
   about end-to-end. Above that ceiling, split. This is the
   operational gate for AI-assisted editing, not aesthetics.

3. **Components communicate only through typed protocols.** No shared
   mutable state, no leaked internals via `pub use`, no cross-crate
   `unsafe`. The protocol is the contract; the type-checker enforces
   it.

4. **Every component builds, tests, and is replaceable on its own.**
   `cargo build` and `nix flake check` must succeed inside the
   component's own repo with no workspace-level helpers. If they
   don't, the boundary is fiction.

5. **Depend on protocols, not implementations.** A consumer names the
   trait/schema crate, never the engine crate. That is what makes a
   component swappable without touching its callers.

6. **Adding a feature defaults to a new crate.** The burden of proof
   is on whoever wants to grow a crate: they must justify why the new
   behavior is part of the *same capability*, not a new one.

7. **No component owns more than one bounded context.** When a crate's
   vocabulary forks — "session" meaning two things, "build" used for
   both verb and artifact — split along that seam.

## Why

Monolith collapse converges on five structural failures, each closed
by per-capability decomposition:

- **Cognitive load.** No one holds the whole picture, so changes ride
  partial mental models. Per-capability components mean no one has to.
- **Change blast radius.** A fix in module A breaks module Z through a
  hidden coupling. Information hiding (Parnas, 1972) is the only known
  antidote; independent crates enforce it.
- **Dependency knots.** Circular and transitive dependencies make
  build/test order brittle. Independent crates turn cycles into
  compile errors, not runtime bugs.
- **Deployment coupling.** In a monolith one bug blocks all releases.
  Even when components link into one binary, the source boundary keeps
  each capability releasable on its own schedule.
- **Test fragility.** Integration tests dominate monoliths and unit
  tests lose meaning because units aren't isolated. Per-capability
  components have meaningful unit tests because the unit is the
  boundary.

The record is unambiguous: Twitter's Ruby monolith became
un-deployable and took years to rewrite into JVM services; Facebook's
PHP monolith was so large the response was a new compiler (HHVM)
rather than decomposition; Healthcare.gov collapsed because
integration was discovered at launch; COBOL bank/government systems
persist because they cannot be modified once institutional knowledge
of the whole retires.

## The LLM-context argument

Frontier context windows are 200k–1M tokens. A monolith of millions
of lines cannot be loaded; the agent operates on partial views and
produces changes that violate invariants it cannot see. The fix is
*not* a larger window — codebases grow faster than windows. The fix
is components small enough that the whole component fits. A Rust crate
of ~3k–10k lines fits in ~30k–80k tokens and can be loaded and
reasoned about in full — that is the operational definition of
LLM-context-sized. McIlroy's 1978 Unix crate-size advice and a 2026
frontier context window land on the same number.

## How

1. **Name the capability as a noun.** If you can't, you don't yet
   understand what you're adding.
2. **Check existing crates.** Does the new noun already match an
   existing crate's stated capability? Only then add to that crate.
3. **Default to a new repo.** A permanent boundary the build system
   enforces, for the cost of a few minutes of plumbing.
4. **If the capability is stateful, default to the triad shape**
   (`component-triad.md`): a runtime repo with a long-lived daemon +
   thin CLI client, and a separate `signal-<component>` contract repo
   for the typed wire vocabulary. The triad gives a stateful
   component a subscribable surface, a debug bridge, and a typed
   boundary peers can speak directly.
5. **Define the protocol crate first** when the component will have
   multiple consumers. Implementations depend on the protocol crate,
   not on each other.
6. **Each component carries its own `ARCHITECTURE.md`, `AGENTS.md`,
   and `skills.md`** at its repo root.

## Cargo.toml dependencies — named `git =` refs, never `path = "../"`

A `Cargo.toml` must not depend on a sibling repo via
`path = "../sibling"`. Cross-repo dependencies use
`git = "https://github.com/..."` with a named reference (branch,
bookmark-as-branch, or tag) or a published crates.io version. The
manifest says which API lane the consumer follows; `Cargo.lock`
records the exact resolved commit for a reproducible build.

```toml
# Wrong — assumes a filesystem layout the consumer's machine lacks
nota-codec = { path = "../nota-codec" }

# Right — portable; the named ref is the API lane, Cargo.lock pins the commit
nota-codec = { git = "https://github.com/LiGoldragon/nota-codec.git", branch = "main" }
```

Choose the reference by intent: `branch = "main"` to track the
current development API; a named compatibility-lane branch while the
next API settles; `tag = "vX.Y.Z"` for a stable release or wire cut;
a crates.io version when published. Do **not** write raw
`rev = "<sha>"` merely to feel pinned — it hides the semantic
contract behind an opaque hash. If a commit matters, point a named
reference at it. Raw revs are acceptable only as a short local
diagnostic override while bisecting, never committed as the normal
shape.

The discriminator: **does the path stay inside the repo's own working
tree?** Intra-repo paths (`path = "lib"` in a Cargo workspace) are
fine — they travel with `git clone`. Any `..` crosses repo boundaries
and breaks the independently-buildable invariant.

Three failures the rule prevents:

1. **Fresh clones don't reproduce.** Cloning the consumer alone gets
   `cargo build` failing with *"could not find Cargo.toml at
   ../sibling"*.
2. **Cargo.lock drifts silently.** A `path` dep records no upstream
   identity; Cargo resolves whatever the local sibling has. A
   `git = "..."` dep names the lane and records the resolved commit.
3. **`nix flake check` can't fetch.** The build sandbox isolates from
   the host filesystem; `path = "../..."` can't cross it.

For local fast iteration without touching the committed `Cargo.toml`,
use Cargo's `[patch."https://github.com/..."]` in a gitignored
user-local `.cargo/config.toml`.

## Distinctions

- **Microservices** — runtime processes over a network. A different
  layer; micro-components is source organization and
  deployment-agnostic.
- **Microkernel** — OS design. A different domain.
- **Modular monolith** — one deployable unit with internal modules.
  Right intent, wrong enforcement: without filesystem boundaries,
  "explicit module boundaries" decay.

The axis micro-components occupies and the others miss:
**source-level, filesystem-enforced decomposition that is
deployment-agnostic.**

## When you're tempted to grow a crate

Stop and ask:

- Can I name this behavior with a noun distinct from the crate's
  current capability?
- Would a fresh reader of the result think "this crate does one
  thing"?
- Does the new behavior introduce vocabulary the crate doesn't
  already use?

If any answer is "yes," start a new crate.

## See also

- `component-triad.md` — the shape every *stateful* component takes:
  daemon + thin CLI + `signal-*` contract.
- `abstractions.md` — every reusable verb belongs to a noun; the same
  discipline at the type level.
- `skill-editor.md` — every component's `skills.md` follows the same
  conventions.
