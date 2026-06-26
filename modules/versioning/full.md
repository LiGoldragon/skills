# Skill — versioning

Apply whenever you change code, generated code, wire contracts,
storage schemas, package metadata, deployment wrappers, or any logic
that affects a running component. A binary still reporting `0.3.0`
after a week of behavioral change lies to every agent, report, test,
and deploy wrapper reasoning about it.

## The rule

Every change that alters component behavior bumps that component's
version in the same change set. The bump is part of the
implementation, never a later cleanup.

Changes that only edit reports, skills, comments, or prose docs do
not bump a version — unless that prose is packaged as the
component's runtime-visible help or public documentation surface.

## Semver before 1.0

Use `major.minor.patch`.

- **Patch** (`0.3.0` → `0.3.1`): compatible bug fixes, internal
  logic changes, behavior corrections, non-breaking improvements.
- **Minor** (`0.3.x` → `0.4.0`): new operation roots, new public
  behavior, new wire fields with compatibility decoding, new deploy
  slots, forward-managed storage migrations. Before 1.0, breaking
  public changes also bump the minor at minimum and are called out
  explicitly in the commit/report.
- **Major**: requires explicit psyche authorization. Crossing to
  `1.0` and every later first-digit bump is the psyche's call. An
  agent may *propose* a major in a report or commit with reasoning,
  but must get the go before making it. Patch and minor an agent
  makes on its own; major is reserved.

## Distinguish the version surfaces

Do not blur these into one word — bump the surface that changed.
When a daemon consumes a changed signal contract, bump the daemon
too.

- **Component release version** — the package/binary version users
  and wrappers see.
- **Wire contract version** — the signal contract crate's semver and
  any protocol handshake value.
- **Storage schema version** — the redb/rkyv schema guard and
  migration ladder.
- **Deployment slot version** — the versioned wrapper or Home/Nix
  profile slot the unsuffixed command points at.

## Where to bump

For Rust: the `Cargo.toml` package version of the changed daemon/CLI
crate and every changed signal contract crate. If the crate exposes
`--version` or a versioned binary name, make that surface read from
the bumped version.

For Nix-packaged tools: any Nix package version, flake input name,
versioned wrapper, or slot mapping that names the old version; update
downstream `flake.lock` files when the component is consumed through
a flake input.

For storage changes: update the schema-version guard and add the
migration step in the same change.

## Commit and report discipline

The commit message names the version move:

```text
persona-spirit: bump to 0.3.1 for privacy-filter fixes
```

When a bump spans repos, the report or final status lists each repo
and each old → new version.

Never report branch work as deployed. A version is:

- **implemented** when source and tests are on the branch;
- **landed** when it is on main;
- **available** when the package can be built from the pin;
- **deployed** only when the running profile or service points at
  the new version.

## Final check

Before finishing a code change: inspect the package metadata
carrying the version; run the relevant tests; if a binary exists,
verify the built or live version surface; update downstream locks
when the changed component is consumed elsewhere; say clearly
whether the version is only landed or also deployed.

If version semantics are unclear — patch vs minor vs wire vs storage
vs deploy-slot — ask the psyche rather than choosing silently.

## See also

- `skills/contract-repo.md` — wire-contract semver.
- `skills/rust/storage-and-wire.md` — storage schema guards and
  migrations.
- `skills/nix-discipline.md` — flake and lock hygiene.
