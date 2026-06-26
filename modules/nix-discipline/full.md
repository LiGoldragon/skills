# Skill — nix discipline

This is the *discipline* — which input form to pick, which command
shape, which test runner, and why. The CLI reference lives in lore
(`nix/basic-usage.md`, `nix/flakes.md`, `rust/nix-packaging.md`).

## Services are NixOS modules, not OCI workloads

Every service on a CriomOS host — directly on the host or inside a
`Contained` node — is a NixOS service module: `services.<name>` from
nixpkgs, a CriomOS module under `services/`, or a typed
`systemd.services.<name>`.

OCI / Docker workloads (`virtualisation.oci-containers`,
`docker-compose` imports, image-tag pins) are not a peer choice; they
are transitional debt. They form a second deployment language — image
tags don't flow from the cluster proposal, image contents don't flow
from the flake, and the operator manages a parallel update cadence.
The whole point of CriomOS is that the flake describes the entire
system; one OCI escape hatch erodes that until reasoning about "what
is on this host" requires joining a store walk with a Docker image
inventory. It is a cost trap too: each OCI service makes eventual
native packaging more expensive, because operational habits come to
depend on the OCI shape.

When upstream ships only an OCI image, package it natively —
`mkYarnPackage`, pnpm overlays, `uv2nix`, crane, `buildGoModule`;
every mainstream stack has a Nix path. "But the official install is
Docker Compose" doesn't override this: the official install is one
reference implementation; the CriomOS implementation is a NixOS
module reading the same upstream sources.

OCI is acceptable only with all three: an explicit transitional bead
with a sunset date; image and tag pinned through a typed cluster
record (no floating `:latest`, no host Compose YAML the cluster
doesn't see); secrets and state declared exactly as for a native
service.

The same holds inside contained nodes. A `Contained { substrate:
NixosContainer }` node runs CriomOS, which runs NixOS service
modules. There is no separate "workload substrate" axis — placement
(`Metal`, `Contained`) names how the node exists; the workload it
runs is just Nix.

## Flake inputs — choosing the form

This is the canonical home for detailed rules about Nix source refs,
flake input forms, `--override-input`, and remote-vs-local testing
discipline. Other skills may mention the rule, but they point back here
rather than restating the full policy.

Default to `github:<owner>/<repo>` for any sibling repo you consume.
It is portable (any machine resolves it identically), reproducible
(the lock pins to a commit, not a working-tree state), and survives
history (if the committing machine disappears, the input still
resolves).

```nix
inputs.nota-codec.url = "github:LiGoldragon/nota-codec";
```

| Form | When | Notes |
|---|---|---|
| `github:<owner>/<repo>` | **Default** for sibling-repo deps | Portable, reproducible, history-stable. |
| `github:<owner>/<repo>?ref=<branch>` | Track a non-default branch | Re-pinned by `nix flake update`. |
| `git+ssh://`, `git+https://` | Repos not on github | Same shape as `github:`, explicit transport. |
| `path:...` | **Forbidden for flake inputs and overrides** | Local filesystem inputs are not a workspace test/deploy surface. |
| `git+file:///...` | **Forbidden** | Same local-filesystem problem as `path:`. |

`flake = false` is for sources without their own `flake.nix` (you
only need the source tree for `import` or a build script). If the
input *does* have a `flake.nix`, omit it — you want its outputs.

### Don't commit `git+file://`

`git+file://` points at a local checkout on the committing
machine. The committed flake then references a path that exists on no
other machine and behaves differently depending on uncommitted
changes. It stops being reproducible (resolves to a different working
tree per host, nothing on the rest) and loses history meaning — it
doesn't pin the dep's commit in `flake.lock` the way `github:` does.

### Testing against a pushed ref — `--override-input`

To point a `github:` input at a feature branch without changing the
committed `flake.nix`, commit and push the dependency repo first, then
override to the remote ref:

```sh
nix flake lock --override-input nota-codec github:LiGoldragon/nota-codec?ref=operator/my-feature
```

This rewrites only the `flake.lock` entry; verify in the `locked`
block under the input. When done, `nix flake update nota-codec` to
re-pin to the intended remote commit. `--override-input ... path:...`
and `git+file://` are forbidden: they make Nix copy local checkout
state, including huge ignored build directories when filtering is
wrong, and they are not reproducible by another agent.

### Multi-repo remote stack checks — ephemeral overrides

For central integration tests that rebuild several sibling repos
together, prefer an ephemeral check over rewriting the lock. The refs
must be remote:

```sh
nix flake check \
  --override-input nota-next-source github:LiGoldragon/nota-next?ref=operator/my-feature \
  --override-input schema-next-source github:LiGoldragon/schema-next?ref=operator/my-feature
```

This keeps `flake.nix` portable while consuming the pushed feature
refs. Committed flakes that support this expose sibling source
inputs with clear names (`nota-next-source`); the build patches Cargo
git deps to those input sources inside the Nix builder. Never use
`path:` or `git+file://` for these overrides.

When the flake has many sibling `*-source` inputs, use the workspace helper
instead of hand-writing the override list:

```sh
tools/nix-local-stack build --target github:LiGoldragon/spirit#default
tools/nix-local-stack check --target github:LiGoldragon/spirit
tools/nix-local-stack build --target github:LiGoldragon/spirit#default --ref operator/my-feature
```

The helper reads the target flake, maps each `*-source` input to the matching
`github:LiGoldragon/<repo>?ref=<ref>` remote, and adds those overrides
ephemerally. It rejects local `path:` and `git+file:` refs.

## Build, run, and deploy from the remote — never a local checkout

When you `nix build`, `nix run`, or **deploy** a workspace repo, name
the remote `github:<owner>/<repo>` (optionally `?ref=<branch>` or a
pinned rev) — never a local `path:` or `git+file://`.

```sh
# Right — commit + push, then build/run/deploy the pushed ref
jj git push --bookmark my-feature --allow-new
nix build github:<owner>/<repo>?ref=my-feature
nix run   github:<owner>/<repo> -- <args>
```

Do not run `nix build` against a local path flake ref.

A `path:` build consumes your uncommitted working tree. What you
ship then depends on local state that exists on no other machine and
vanishes at the next edit — not reproducible, impossible to hand off.
This matters most for deploys: a cluster deploy must build from a
pushed ref so the closure is reproducible and the deploy is re-runnable
by another operator. If you reach for `path:` only because the change
is uncommitted, that is the signal to commit and push — not to bypass
the remote. There is no sanctioned local-path override for workspace
integration builds.

## Compiled artefacts at build time, never JIT

When a derivation builds a config/script/module for a runtime with
its own JIT or AOT compilation (Emacs Lisp `.eln`, Python bytecode,
Common Lisp FASLs, TypeScript declaration emit, sass/SCSS), produce
the compiled artefacts at Nix build time inside the derivation. Do
not let them appear lazily at first use.

A Nix-built artefact is content-hashed and store-shipped. If
compilation happens at runtime, the compiled cache lives outside the
store (`~/.emacs.d/eln-cache/` or similar) and invalidates on every
rebuild — the next `home-manager switch` produces a new source hash,
the runtime cache misses, JIT runs again. The regression is invisible
until the user feels the slowness on first cold start after each
rebuild. Build-time compilation puts the `.eln` / `.pyc` / `.fasl` in
the store next to its source, so the cache hits on every cold start
regardless of rebuild cycle.

Smell test: if the Nix-built tool starts equally fast on every fresh
store entry, the derivation compiles at build time. If first use after
rebuild is slow and subsequent uses are fast, runtime JIT is doing the
work — the regression this forbids. (Worked example: `CriomOS-home`'s
`initElCompiled` byte- and native-compiles `init.el` in one
`pkgs.runCommand` with `emacs --batch`, ships `init.elc` +
`init-<hash>.eln`, and a generated `early-init.el` prepends the
store-path `eln-cache/` to `native-comp-eln-load-path`.)

## Lock-side pinning — never a hash in `flake.nix`

Keep `flake.nix` generic; record the exact rev in `flake.lock`.

```nix
# flake.nix — generic, no hash
inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
```

```sh
# pinning happens via the CLI
nix flake lock --override-input nixpkgs github:<org>/nixpkgs/<rev>
```

`flake.lock` is machine-generated; never hand-edit it — hand edits
drift silently and the next `nix flake lock` overwrites them. To fix
a wrong entry: `nix flake update` (all inputs), `nix flake update
<input>` (one), or `nix flake lock --override-input <name> <url>`
(pin to a URL/rev). Commit the lock after, naming what changed
(`update nota-codec to <short-sha>`).

To reuse a rev another flake already pins, use `--inputs-from` with the
sibling's pushed remote ref — resolves matching inputs from that
sibling's locked entries, no hash typed by hand.

For the workspace fenix lockstep (every Rust crate's fenix lock
copied from a canonical source so the workspace shares one rustc
store path), see lore's `rust/nix-packaging.md`.

## Cargo git deps in crane flakes — never `outputHashes`

When a Rust crate consumes sibling crates as `git = "..."` deps
(`nota-codec`, `nota-derive`, `horizon-lib`), don't declare
`cargoVendorDir.outputHashes = { ... }` in `flake.nix`. Modern crane
fetches git deps directly from `Cargo.lock`'s git-source metadata —
the rev already in the lock is enough. A redundant `outputHashes`
block re-pins the same hash in `flake.nix`, violating the
no-hashes-in-`flake.nix` rule and creating two places to update.

Right shape — `Cargo.lock` is the source of truth:

```nix
craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
src = craneLib.cleanCargoSource ./.;
commonArgs = { inherit src; strictDeps = true; };
cargoArtifacts = craneLib.buildDepsOnly commonArgs;
# No outputHashes block.
```

Wrong shape — a `cargoVendorDir.outputHashes` block listing
`"git+https://.../horizon-rs#<rev>" = "sha256-<hash>";` entries.
Older flakes that pre-date crane's lock-only git-dep handling still
carry it; drop the block.

Mixing rev pins across `flake.nix` and `Cargo.lock` produces
two-place updates (forgetting one fails the build), hash-mismatch
theatre (paste sha, rebuild, next mismatch, repeat — cargo-cult
typing), and drift between consumers vendoring the same crate.

### Bumping a Rust git dep — Cargo.lock-only

```sh
nix run nixpkgs#cargo -- update -p <crate>   # new rev in Cargo.lock
nix build .# -L                              # crane fetches via lock
```

No `flake.nix` edit, no hash to rotate. To pin to a specific rev (to
keep compatibility with another consumer):

```sh
nix run nixpkgs#cargo -- update -p nota-codec --precise <rev>
```

Trap: bare `cargo update` (no `-p <pkg>`) bumps every git-source
crate to its branch tip. Sibling crates sharing a transitive dep
(nota-codec here) then lock at the latest tip, which may be
API-incompatible with a sibling still on the older rev. Pin with
`--precise <rev>` to the rev the other sibling consumes when the bump
is breaking.

## Don't reference raw `/nix/store/<hash>-<name>` paths

Store hashes change on every rebuild, so any recorded path goes stale
within minutes — long, noisy, silently wrong by tomorrow.

- Name a binary by plain name (`dolt`, `bd`, `jq`) or profile path
  (`~/.nix-profile/bin/dolt`), never the resolved `/nix/store/...`.
- When tool output (`ps`, `env`, `ls`) contains store paths, refer to
  the thing by package name rather than quoting the path back.
- If a store path is genuinely load-bearing for the point ("two
  `dolt` versions coexist"), say so explicitly — don't paste the hash
  and call it documentation.
- A `/nix/store/...` literal in a commit message or report freezes
  one build's hash into history forever and reads as archaeological
  junk after the next build.

Capture in a shell variable when a store path is needed for a
one-shot operation:

```sh
result=$(nix build .#some-output --print-out-paths --no-link)
ls "$result"/bin   # local to this shell; nothing freezes into history
```

## Use `nix run nixpkgs#<pkg>` for missing tools

When a tool isn't on `PATH` (`rustfmt`, `clippy`, `jq`, `ripgrep`),
invoke it via Nix:

```sh
nix run nixpkgs#<package> -- <args>
```

Don't reach for `cargo install`, `pip install`, `npm install -g`,
distro package managers, or hand-written shell substitutes. The setup
is Nix-managed end-to-end; out-of-Nix installs pollute the
environment, are non-reproducible, and bypass the system's
invariants. Nix caches the build, so repeat use in a session is free.
Don't fall back to a bespoke Python/sed/awk substitute "for speed"
while nix fetches — it's almost always faster than debugging a
hand-rolled substitute. Reserve writing a script substitute for cases
where no upstream tool exists.

For one-shot invocations of this workspace's flake outputs, prefer
`nix run .#<attr> -- <args>`. Reach for `nix build` only when the
store path itself is load-bearing (closure introspection, `nix copy`)
— and even then, capture the path in a shell variable.

## `nix flake check` is the canonical pre-commit runner

Every Rust crate (ideally every flake) exposes its test suite as
`checks.default`. Use `nix flake check` as the pre-commit test
runner, not bare `cargo test`. It pins the toolchain to the flake's
`fenix` component (no host-rustc drift), resolves deps from the
committed `Cargo.lock` / `flake.lock` (no "works on my machine"), and
makes the invocation self-documenting (any checkout reproduces the
exact suite). `cargo test` skips those guarantees — fine for a tight
inner loop, but treat `nix flake check` as the gate before pushing.

## See also

- lore's `rust/nix-packaging.md` — canonical crane + fenix flake
  layout and the workspace fenix lockstep.
- `skills/testing.md` — pure, stateful, and chained test surfaces
  through Nix.
- `skills/jj.md` — push before building, so the input is reachable
  from the lock.
