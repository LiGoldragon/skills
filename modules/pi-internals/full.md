# Skill — pi internals

## Rules

Inspect installed Pi files, pinned Pi source, docs, and examples when Pi behavior matters and the active role permits direct inspection. In delegation-only roles, dispatch inspection instead of reading directly.

Treat installed Pi packages, generated runtime files, profile symlinks, `$HOME/.pi/agent/bin/pi`, `$HOME/.local/share/criomos/pi/package`, and runtime `dist` as read-only evidence. Do not replace managed symlinks, patch installed runtime output, add ad hoc dependency symlinks, shadow profile commands, or make copied installed source the effective Pi system.

Make durable Pi package, prompt, skill, extension, theme, settings, and harness changes through CriomOS-home/Nix patches or declarative package and configuration surfaces. Commit source, update portable inputs, build or check the owning Nix surface, and redeploy.

Read-only inspection, byte-for-byte backups for evidence preservation, and isolated repro copies are allowed when the active role permits them. They must not become effective runtime, profile, or system behavior, and they are not closeout fixes.

Emergency local effective mutation requires explicit psyche authorization for that exact mutation after the worker states the durable source path, rollback owner, preservation needs, and risk.

Preserve active role and action-space restrictions. Do not use Pi internals to bypass management delegation, read-only Spirit boundaries, or repository closeout.

Keep package inputs portable through flake inputs, committed patches, and lock files. Validate the narrow Nix surface that owns the change. Closeout is blocked when Pi behavior depends on uncommitted runtime edits, PATH shims, replaced managed symlinks, or copied installed source.
