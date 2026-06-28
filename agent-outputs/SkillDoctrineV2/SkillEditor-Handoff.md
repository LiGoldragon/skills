# SkillDoctrineV2 Role Bundle Heading Collision Handoff

## Task And Scope

Task: fix the role bundle heading collision blocking SkillDoctrineV2 generation/check after V2 role bundle and doctrine edits.

Scope held narrow:

- identify why `intent-translator` generated packets could receive a duplicate generic `Purpose` heading;
- fix source instruction prose, not generated output by hand;
- regenerate primary outputs through the configured generator;
- run `skills-check.nota`;
- run focused stale-reference checks for active runtime-discovery language.

## Files Consulted

- `skills.md`
- `AGENTS.md`
- `manifests/active-outputs.nota`
- `manifests/module-dependencies.nota`
- `roles/intent-translator/full.md`
- `modules/agent-output-protocol/full.md`
- `modules/worker-output-core/full.md`
- `modules/workspace-context-core/full.md`
- `modules/bead-weaver/full.md`
- `/home/li/primary/.claude/agents/intent-translator.md`

## Observed Facts

- `intent-translator` includes `agent-output-protocol`, `worker-output-core`, `bead-weaver`, and `workspace-context-core`.
- `modules/agent-output-protocol/full.md` had a generic `## Purpose` section.
- The other newly included core modules already use scoped purpose headings:
  `## Output Core Purpose` and workspace-specific headings.
- Generated role packets demote included-module headings, so the generic source
  heading became `### Purpose` inside every generated role packet.
- Before regeneration with environment variables set, `skills-check.nota`
  failed on stale `/home/li/primary/.claude/agents/intent-translator.md`.
- A raw `cargo run -- skills-generate.nota` without environment variables
  exits before generation because `SKILLS_SOURCE_ROOT` and
  `SKILLS_WORKSPACE_ROOT` are required by the request file.

## Change Made

Changed one source heading:

- `modules/agent-output-protocol/full.md`: `## Purpose` became
  `## Output Protocol Purpose`.

The body text was unchanged. This keeps the output protocol content intact while
making the included module heading unique and module-scoped in role bundles.

Generated primary role packets were then regenerated through the generator. The
reconciled generated surfaces include configured role outputs under:

- `/home/li/primary/.claude/agents/`
- `/home/li/primary/.codex/agents/`
- `/home/li/primary/.pi/agents/`

The generator also checked all configured active skill outputs. No generated
files were edited by hand.

## Commands And Results

- `cargo run -- skills-generate.nota`
  - Result: failed before generation because `SKILLS_SOURCE_ROOT` was unset:
    `skills: environment variable SKILLS_SOURCE_ROOT must name a generation root`.

- `SKILLS_SOURCE_ROOT=$PWD SKILLS_WORKSPACE_ROOT=/home/li/primary cargo run -- skills-check.nota`
  - Result before source fix/regeneration: failed with stale generated output:
    `/home/li/primary/.claude/agents/intent-translator.md`.

- `SKILLS_SOURCE_ROOT=$PWD SKILLS_WORKSPACE_ROOT=/home/li/primary cargo run -- skills-generate.nota`
  - Result after source fix: passed and generated all configured active outputs,
    including role packets and `skills/generated-role-outputs.nota`.

- `SKILLS_SOURCE_ROOT=$PWD SKILLS_WORKSPACE_ROOT=/home/li/primary cargo run -- skills-check.nota`
  - Result after regeneration: passed.

- `rg -n "runtime skill-index|skill-index discovery|perform runtime skill-index discovery|Do not perform runtime skill-index discovery|skills\\.nota|runtime-discovery|runtime discovery" /home/li/primary/.agents/skills /home/li/primary/.claude/skills /home/li/primary/.claude/agents /home/li/primary/.codex/agents /home/li/primary/.pi/agents; printf 'exit=%s\n' $?`
  - Result: no matches, exit `1`.

- `rg -n "^### Purpose$|^## Purpose$" /home/li/primary/.claude/agents /home/li/primary/.pi/agents /home/li/primary/.codex/agents; printf 'exit=%s\n' $?`
  - Result: no bare `Purpose` headings remain in generated role packets, exit
    `1`.

- `rg -n "Output Protocol Purpose|^### Purpose$|^## Purpose$" /home/li/primary/.claude/agents/intent-translator.md /home/li/primary/.pi/agents/intent-translator.md /home/li/primary/.codex/agents/intent-translator.toml`
  - Result: `intent-translator` role packets show `Output Protocol Purpose`;
    no bare `Purpose` heading is present.

- `rg -n "^---$" modules/agent-output-protocol/full.md roles/intent-translator/full.md modules/worker-output-core/full.md modules/workspace-context-core/full.md`
  - Result: no harness frontmatter delimiters in the checked source files, exit
    `1`.

## Status

Primary generated outputs are reconciled for the current skills repo working
copy: `skills-check.nota` passes after generator regeneration.

No commit or push was performed, per task instruction.

## Notes For Next Agent

The skills repo already had a dirty working copy from the broader
SkillDoctrineV2 work before this narrow fix. This handoff only claims the
additional source fix in `modules/agent-output-protocol/full.md` and the
generator reconciliation that followed it.
