# Skill — disk hygiene

## Rules

Use this under disk pressure or as periodic host hygiene when reclaiming disk
space on the real host. It is a cleanup protocol, not a Lojix or os-operations
verb — os-operations exposes deploy, activate, and rollback only and has no
garbage-collection verb. Measure before and after, reclaim the two safe
categories yourself, and ask the psyche before touching anything else.

## Measure first

Baseline with `df -h`. Then size real directories with `du` — home, `repos/`,
`private-repos/`, worktrees, and caches. Never `du`, `find`, or walk
`/nix/store`; it is a hard boundary and it misses the point, because the store
lives on the nix filesystem and its reclaim is invisible to `du`. Measure nix
reclaim by `df` deltas only. On a real sweep, `du` over `/home` would have
missed a 196 GB store-GC win entirely.

## Nix generations — the largest, most counterintuitive win

Delete old generations from user-owned profiles first to release their store
roots, then run one garbage collection:

```sh
nix-env --profile ~/.local/state/nix/profiles/home-manager --delete-generations old
nix-env --profile ~/.local/state/nix/profiles/profile --delete-generations old
nix-collect-garbage -d
```

A user-invoked `nix-collect-garbage -d` triggers a global store GC through the
root daemon and reclaims the bulk of store garbage without sudo. On a real host
this single GC freed 196 GB of a 215 GB sweep.

The system profile needs sudo, so a background or non-interactive agent cannot
do it — hand it to whoever holds sudo. Booted is not current: resolve
`readlink /run/booted-system` against `readlink /run/current-system` and
preserve both generations. One host booted `system-136` while `system-142` was
activated; both had to survive. On the system profile run `nix-collect-garbage`
without `-d` — bare `-d` collapses to current-only and drops the booted
generation's symlink. A `booted-system` gcroot protects the running closure
regardless, but keeping the generation keeps rollback intact.

Lojix records that reference home-manager or system store paths are historical,
not GC roots, so store GC does not corrupt lojix. After deleting a
system-profile generation, refresh stale boot-menu entries with a lojix deploy,
never a manual `nixos-rebuild`.

## Rust target directories — safe, high confidence

`rm -rf <dir>/target` equals `cargo clean` and never touches source. Guard
first: confirm the directory is a build directory — `CACHEDIR.TAG` present, or a
`debug/` or `release/` subdirectory. These concentrate in throwaway worktrees,
so there is no rebuild cost worth keeping. About 12 GB on a real sweep.

## Safe versus ask first

Self-authorize only two categories: old nix generations, always keeping both the
current and the booted generation, and rust `target/` directories.

Ask the psyche before deleting anything else — media such as Audiobooks, Music,
and Documents; Downloads; browser profile data under `.config` and `.cache`
chrome; the `.cargo` registry cache; `~/tmp`; and any large path outside the two
safe categories.

## Close the loop

Re-measure with `df -h` and report space reclaimed overall and per category.
