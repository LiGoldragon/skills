# RemoveSkillIndexGeneration Evidence

## Task And Scope

Implemented `RemoveSkillIndexGeneration` for SkillDoctrineV2 in the skills generator repo. The requested code scope was generator assembly and generation tests, with schema surfaces only if needed. No schema changes were needed.

Settled intent applied:

- Stop generating `skills/skills.nota`.
- Stop treating `skills/skills.nota` as an expected generated output.
- Remove dead `SkillIndex` and `ActiveSkill::index_record()` code.
- Keep generated skill files emitting.
- Keep stale-output behavior from treating old `skills.nota` as a normal expected output.

## Files Consulted

- `AGENTS.md`
- `skills.md`
- `ARCHITECTURE.md`
- `src/assembly.rs`
- `tests/generation.rs`
- `src/schema/assembly.rs`
- `src/error.rs`

## Changed Files

- `src/assembly.rs`
  - Removed the `GenerationJob::Rendered` job for `OutputPath::new("skills/skills.nota")`.
  - Removed `skills/skills.nota` from `GenerationConfiguration::expected_outputs()`.
  - Removed dead `SkillIndex`, `ActiveSkill::index_record()`, `SkillCategory::as_str()`, and `SkillTier::as_str()` code.
  - Removed now-unused `SkillCategory` and `SkillTier` imports from this module.

- `tests/generation.rs`
  - Updated the main skill generation test to assert `.agents` and `.claude` skill files are still reported and written.
  - Added assertions that `skills/skills.nota` is not reported and is absent in a clean generated workspace.
  - Removed `expected_empty_index()`.
  - Updated stale-output tests so an old `skills/skills.nota` fixture is ignored rather than modeled as a normal expected generated output.

## Verification

- `cargo fmt --check`: failed before formatting due to the new multiline assertion shape.
- `cargo test --test generation generation_writes_derived_skill_surfaces_with_roster_frontmatter check_mode_reports_stale_output_with_guidance check_mode_reports_archived_or_deleted_stale_skill_outputs`: failed because Cargo accepts only one test-name filter in that position.
- `cargo fmt`: passed.
- `cargo test --test generation`: passed, 15 tests.
- `cargo fmt --check`: passed.
- `cargo test`: passed, 15 integration tests; lib/main/doc tests had 0 tests.
- `nix flake check`: passed all checks for this host's compatible system set. Nix reported omitted incompatible systems: `aarch64-darwin`, `aarch64-linux`, `x86_64-darwin`.

## Observations

- The working copy also contains unrelated edits in `ARCHITECTURE.md`, `README.md`, and several `modules/*/full.md` files. They were present after my scoped edits and I did not modify or revert them.
- `skills/skills.nota` is no longer generated or expected, but this change does not intentionally prune an already-existing retired `skills/skills.nota` file from a target workspace.

## Follow-Up Requirements

- Generated consuming workspaces still need their old `skills/skills.nota` files removed through the appropriate generated-output or workspace update flow.
- Docs and module prose already have concurrent edits removing `skills.nota` references; those were outside this implementation scope and should be integrated by their owner.
