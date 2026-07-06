# Skill — pi internals

## Rules

Inspect installed Pi files, pinned Pi source, docs, and examples when Pi behavior matters and the active role permits direct inspection. In delegation-only roles, dispatch inspection instead of reading directly.

Treat installed copies and derivation outputs as read-only evidence. Do not edit `/nix/store`.

Make durable Pi package, prompt, skill, extension, theme, settings, and harness changes through CriomOS-home/Nix patches or declarative package and configuration surfaces.

Avoid mutable Pi state as the durable fix. Use `$HOME/.pi` only for temporary probes; commit the fix to Nix-managed source.

Preserve active role and action-space restrictions. Do not use Pi internals to bypass orchestration delegation, read-only Spirit boundaries, or repository closeout.

Keep package inputs portable through flake inputs, committed patches, and lock files. Validate the narrow Nix surface that owns the change.
