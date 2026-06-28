# Rust Auditor Generator Review

Task: audit SkillDoctrineV2 generator/test changes focused on removal of generated/runtime `skills/skills.nota`, dead `SkillIndex` assumptions, stale-output behavior, primary workspace deletion handling, and Rust/style issues.

Scope inspected:

- `/git/github.com/LiGoldragon/skills/agent-outputs/RemoveSkillIndexGeneration/GeneralCodeImplementer-Evidence.md`
- `/git/github.com/LiGoldragon/skills/src/assembly.rs`
- `/git/github.com/LiGoldragon/skills/tests/generation.rs`
- Targeted references in `src`, `tests`, `manifests`, `README.md`, `ARCHITECTURE.md`, `skills.md`
- Primary workspace presence/status for `/home/li/primary/skills/skills.nota`

## Findings

### High: Current generation test suite fails after manifest/module changes

File: `/git/github.com/LiGoldragon/skills/tests/generation.rs:208`

Risk: `cargo test --test generation` currently fails because `active_manifest_and_module_index_cover_current_skills_and_roles` still expects `module_dependencies.payload().len() == 77`, while the current manifest parses 87 module dependency records. This leaves the generator test suite red in the audited working copy even though the SkillIndex removal itself compiles under focused tests.

Expected correction: update the sentinel count to the current manifest count, or replace the brittle total-count assertion with a more direct witness for the role-bundle/module identities that must exist. If the count is intentionally a drift sentinel, set it to 87 with the same care as the skill and role counts above it.

Evidence:

- `cargo test --test generation` failed: 14 passed, 1 failed.
- Failure: `tests/generation.rs:208:5`, left `87`, right `77`.

### Medium: Retired `skills.nota` ignore behavior lacks a direct positive witness

Files:

- `/git/github.com/LiGoldragon/skills/tests/generation.rs:407`
- `/git/github.com/LiGoldragon/skills/tests/generation.rs:429`

Risk: the updated stale-output tests place an old `skills/skills.nota` next to another stale output, then assert that the reported error does not mention `skills.nota`. That catches accidental restoration of the old rendered `skills.nota` job because that job would fail before skill-file checks. It does not prove the intended standalone behavior: a workspace whose generated outputs are current but still contains the retired `skills/skills.nota` should pass `Check` if deletion is handled outside generator pruning.

A future change could add a `StaleOutputScan::require_absent("skills/skills.nota")` tombstone after the normal stale skill checks and these two tests would still pass, because both fixtures already fail earlier on `.agents/skills/.../SKILL.md`.

Expected correction: add a dedicated test that writes current expected skill outputs, adds an old `skills/skills.nota`, then runs `GenerationMode::Check` and expects success. That witnesses the settled behavior that the retired file is neither generated, expected, nor generator-pruned. If the intended policy changes to generator-owned deletion, encode that explicitly instead.

## Non-Findings

- No live `SkillIndex`, `ActiveSkill::index_record`, or `skills/skills.nota` generation path remains in `src/assembly.rs`.
- `GenerationConfiguration::expected_outputs()` no longer inserts `skills/skills.nota`; it now covers skill manifests, role manifests, active-manifest role inventory, and legacy entrypoint extras.
- `WorkspacePruner::prune()` still removes generated skill directories, legacy extras, and inventory-owned stale role outputs only. It does not remove `skills/skills.nota`, matching the settled direction that primary workspace deletion is handled outside generator pruning.
- `/home/li/primary/skills/skills.nota` is absent on disk. `jj file list skills/skills.nota` in `/home/li/primary` reported no matching entry, so the file is not visible as a tracked primary workspace file in the current revision state.
- Rust shape/style in the audited removal is acceptable: behavior remains on data-bearing types, errors remain typed through the crate `Error`, NOTA parsing still uses the NOTA codec, and no new ZST namespace holder was introduced.

## Checks Run

- `sed -n '1,240p' agent-outputs/RemoveSkillIndexGeneration/GeneralCodeImplementer-Evidence.md`
  - Result: read implementer evidence and verification claims.
- `nl -ba src/assembly.rs | sed -n '1,1245p'`
  - Result: inspected generator source, job construction, expected outputs, rendered output checks, role inventory, pruner, and stale-output scan.
- `nl -ba tests/generation.rs | sed -n '1,577p'`
  - Result: inspected generation tests and fixtures.
- `jj status` in `/git/github.com/LiGoldragon/skills`
  - Result: working copy is dirty with many concurrent non-audit changes; audited Rust paths are `src/assembly.rs` and `tests/generation.rs`.
- `jj diff -- src/assembly.rs tests/generation.rs`
  - Result: confirmed removal of `skills/skills.nota` rendered job, expected-output entry, `SkillIndex`, `index_record`, and old index fixture helper.
- `rg -n "SkillIndex|skills\\.nota|index_record|expected_empty_index|SkillCategory|SkillTier|expected_outputs|StaleOutput|Pruner|generated-role-outputs" src tests manifests modules README.md ARCHITECTURE.md`
  - Result: no live `SkillIndex`/`index_record`; `skills.nota` references in audited source are limited to tests asserting non-generation/ignored stale fixture behavior.
- `rg -n "skills/skills\\.nota|skills\\.nota|SkillIndex|index_record" src tests manifests README.md ARCHITECTURE.md skills.md`
  - Result: in the generator repo active docs/source scope, `skills.nota` appears only in tests.
- `test ! -e /home/li/primary/skills/skills.nota`
  - Result: passed; printed `absent`.
- `jj file list skills/skills.nota` in `/home/li/primary`
  - Result: warning reported no matching entries.
- `cargo test --test generation generation_writes_derived_skill_surfaces_with_roster_frontmatter`
  - Result: passed; verifies normal generation omits `skills/skills.nota` from report and clean workspace.
- `cargo test --test generation check_mode_reports_stale_output_with_guidance`
  - Result: passed.
- `cargo test --test generation check_mode_reports_archived_or_deleted_stale_skill_outputs`
  - Result: passed.
- `cargo fmt --check`
  - Result: passed.
- `cargo test --test generation`
  - Result: failed; 14 passed, 1 failed at `tests/generation.rs:208` due module dependency count mismatch, 87 actual vs 77 expected.
- `cargo test --test generation check_mode_reports_stale_output_with_guidance check_mode_reports_archived_or_deleted_stale_skill_outputs`
  - Result: failed due invalid Cargo command shape; Cargo accepts only one test-name filter in that position. This is not a code failure.

## Residual Risks

- I did not run `nix flake check` because the narrower Rust generation test suite is currently red.
- I did not modify code or tests. The count fix at `tests/generation.rs:208` is small, but this audit stayed read-only per the assignment.
