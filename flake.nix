{
  description = "skills — generated skill surface assembler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nota-source = {
      url = "github:LiGoldragon/nota/ce7c564de0a0518eaa1938d55dccc460a67cadb4";
      flake = false;
    };
    schema-source = {
      url = "github:LiGoldragon/schema/f351f90d3b8898205cf3057f3c253a5e451180a9";
      flake = false;
    };
    schema-rust-source = {
      url = "github:LiGoldragon/schema-rust";
      flake = false;
    };
    signal-frame-source = {
      url = "github:LiGoldragon/signal-frame/bb86bef67e478ff52690a4dcceec8f22d2b005ad";
      flake = false;
    };
    triad-runtime-source = {
      url = "github:LiGoldragon/triad-runtime/0031b5519572f4571bf3895f78221de9404d4810";
      flake = false;
    };
    kameo-source = {
      url = "github:LiGoldragon/kameo/main";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-build,
      nota-source,
      schema-source,
      schema-rust-source,
      signal-frame-source,
      triad-runtime-source,
      kameo-source,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;

        skillSourceFilter =
          path: type:
          type == "directory"
          || pkgs.lib.hasSuffix ".md" path
          || pkgs.lib.hasSuffix ".nota" path
          || pkgs.lib.hasSuffix ".schema" path;

        cleanSource = rust.cleanSource {
          root = ./.;
          extraFilters = [ skillSourceFilter ];
        };

        src = pkgs.runCommand "skills-source-with-flake-input-patches" { } ''
            mkdir -p "$out"
            cp -R ${cleanSource}/. "$out"/
            chmod -R u+w "$out"
            mkdir -p "$out/vendor-sources"
            cp -R ${nota-source} "$out/vendor-sources/nota"
            cp -R ${schema-source} "$out/vendor-sources/schema"
            cp -R ${schema-rust-source} "$out/vendor-sources/schema-rust"
            cp -R ${signal-frame-source} "$out/vendor-sources/signal-frame"
            cp -R ${triad-runtime-source} "$out/vendor-sources/triad-runtime"
            cp -R ${kameo-source} "$out/vendor-sources/kameo"
            chmod -R u+w "$out/vendor-sources"
            cat >> "$out/Cargo.toml" <<'EOF'

          [patch."https://github.com/LiGoldragon/nota.git"]
          nota = { path = "vendor-sources/nota" }
          nota-derive = { path = "vendor-sources/nota/derive" }

          [patch."https://github.com/LiGoldragon/schema.git"]
          schema = { path = "vendor-sources/schema" }
          schema-cc = { path = "vendor-sources/schema/schema-cc" }

          [patch."https://github.com/LiGoldragon/schema-rust.git"]
          schema-rust = { path = "vendor-sources/schema-rust" }

          [patch."https://github.com/LiGoldragon/signal-frame.git"]
          signal-frame = { path = "vendor-sources/signal-frame" }
          signal-frame-macros = { path = "vendor-sources/signal-frame/macros" }

          [patch."https://github.com/LiGoldragon/triad-runtime.git"]
          triad-runtime = { path = "vendor-sources/triad-runtime" }

          [patch."https://github.com/LiGoldragon/kameo.git"]
          kameo = { path = "vendor-sources/kameo" }
          kameo_macros = { path = "vendor-sources/kameo/macros" }
          EOF
        '';

        patchedCargoLock = pkgs.runCommand "skills-patched-Cargo.lock" { } ''
          ${pkgs.python3}/bin/python3 - ${./Cargo.lock} "$out" <<'PYEOF'
          import re
          import sys

          path_dependency_names = {
              "kameo",
              "kameo_macros",
              "nota",
              "nota-derive",
              "schema",
              "schema-cc",
              "schema-rust",
              "signal-frame",
              "signal-frame-macros",
              "triad-runtime",
          }
          source_text = open(sys.argv[1]).read()
          blocks = source_text.split("[[package]]")
          header, entries = blocks[0], blocks[1:]

          def field(entry, name):
              found = re.search(r'^%s = "([^"]*)"' % name, entry, re.M)
              return found.group(1) if found else ""

          stripped = []
          for entry in entries:
              if field(entry, "name") in path_dependency_names:
                  entry = "\n".join(
                      line for line in entry.split("\n")
                      if not line.startswith('source = "git+https://github.com/LiGoldragon/')
                  )
              stripped.append(entry)

          open(sys.argv[2], "w").write(header + "".join("[[package]]" + entry for entry in stripped))
          PYEOF
        '';

        cargoVendorDirectory = craneLib.vendorCargoDeps {
          inherit src;
          cargoLock = patchedCargoLock;
        };

        commonArguments = {
          inherit src cargoVendorDirectory;
          cargoLock = patchedCargoLock;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
        skillsPackage = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });

        generatorApp =
          name: requestFile:
          let
            script = pkgs.writeShellApplication {
              inherit name;
              runtimeInputs = [ skillsPackage ];
              text = ''
                if [ "$#" -gt 1 ]; then
                  echo "usage: ${name} [workspace-root]" >&2
                  exit 2
                fi
                workspace_root="''${1:-$PWD}"
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$workspace_root"
                exec skills ${cleanSource}/${requestFile}
              '';
            };
          in
          {
            type = "app";
            program = "${script}/bin/${name}";
            meta.description = "Run ${name} against an explicit workspace root";
          };
      in
      rec {
        packages = {
          skills = skillsPackage;
          default = skillsPackage;
        };

        apps = rec {
          skills = {
            type = "app";
            program = "${skillsPackage}/bin/skills";
            meta.description = "Run the skills generator CLI";
          };
          generate-skills = generatorApp "generate-skills" "skills-generate.nota";
          check-skills = generatorApp "check-skills" "skills-check.nota";
          default = skills;
        };

        checks = rec {
          skills = skillsPackage;
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (
            commonArguments
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            }
          );
          no-hard-coded-generation-roots = pkgs.runCommand "skills-no-hard-coded-generation-roots" { } ''
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-check.nota >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-check.nota >/dev/null
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-generate.nota >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-generate.nota >/dev/null
            if grep -n -E '/(home|git)/' ${cleanSource}/skills-check.nota ${cleanSource}/skills-generate.nota; then
              echo "generation requests must not hard-code source or workspace roots" >&2
              exit 1
            fi
            touch "$out"
          '';
          check-request-is-non-writing = pkgs.runCommand "skills-check-request-is-non-writing" { } ''
            grep -F ' Check))' ${cleanSource}/skills-check.nota >/dev/null
            if grep -F ' Write))' ${cleanSource}/skills-check.nota; then
              echo "check request must not use Write mode" >&2
              exit 1
            fi
            touch "$out"
          '';
          generation-requests-use-active-manifest = pkgs.runCommand "skills-generation-requests-use-active-manifest" { } ''
            grep -F 'manifests/active-outputs.nota' ${cleanSource}/skills-check.nota >/dev/null
            grep -F 'manifests/active-outputs.nota' ${cleanSource}/skills-generate.nota >/dev/null
            if find ${cleanSource}/manifests -mindepth 2 -type f -name '*.nota' | grep .; then
              echo "generation must be driven by the single active output manifest, not per-output manifests" >&2
              exit 1
            fi
            touch "$out"
          '';
          stale-orchestration-aliases-removed = pkgs.runCommand "skills-stale-orchestration-aliases-removed" { } ''
            dash='-'
            old_prefix='intent-led'
            old_skill="$old_prefix$dash"'orchestration'
            generate_wrapper="generate-$old_skill"
            check_wrapper="check-$old_skill"
            generate_request="$old_skill-generate"
            check_request="$old_skill-check"
            if grep -R -F -e "$old_skill" -e "$generate_wrapper" -e "$check_wrapper" -e "$generate_request" -e "$check_request" ${cleanSource}/apps ${cleanSource}/manifests ${cleanSource}/modules ${cleanSource}/skills-check.nota ${cleanSource}/skills-generate.nota 2>/dev/null; then
              echo "obsolete narrow orchestration aliases or requests remain" >&2
              exit 1
            fi
            touch "$out"
          '';
          human-interaction-removed-from-active-and-generated = pkgs.runCommand "skills-human-interaction-removed-from-active-and-generated" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            index=${cleanSource}/manifests/module-dependencies.nota
            if grep -R -F 'human-interaction' "$manifest" "$index" ${cleanSource}/manifests/skills-roster.nota ${cleanSource}/modules; then
              echo "human-interaction must be deleted from source manifests and modules" >&2
              exit 1
            fi
            workspace=$TMPDIR/workspace
            mkdir -p "$workspace/.agents/skills/human-interaction" "$workspace/.claude/skills/human-interaction"
            printf 'stale\n' > "$workspace/.agents/skills/human-interaction/SKILL.md"
            printf 'stale\n' > "$workspace/.claude/skills/human-interaction/SKILL.md"
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
            test ! -e "$workspace/.agents/skills/human-interaction/SKILL.md"
            test ! -e "$workspace/.claude/skills/human-interaction/SKILL.md"
            touch "$out"
          '';
          skill-editor-source-of-truth-guardrails = pkgs.runCommand "skills-skill-editor-source-of-truth-guardrails" { } ''
            skill_module=${cleanSource}/modules/skill-editor/full.md
            role_module=${cleanSource}/roles/skill-editor/full.md
            source_core=${cleanSource}/modules/skill-source-core/full.md
            for source in "$skill_module" "$role_module" "$source_core"; do
              grep -F '`LiGoldragon/skills` as the canonical skills source' "$source" >/dev/null
              grep -F 'generated runtime targets' "$source" >/dev/null
              if grep -F 'generated runtime copies first' "$source"; then
                echo "skill-editor doctrine must not preserve old generated-copy-first wording" >&2
                exit 1
              fi
            done
            for source in "$skill_module" "$role_module"; do
              grep -F 'workspace skill and agent files' "$source" >/dev/null
              grep -F '.agents/skills' "$source" >/dev/null
              grep -F '.claude/skills' "$source" >/dev/null
              grep -F '.pi/agents' "$source" >/dev/null
              grep -F '.codex/agents' "$source" >/dev/null
            done
            touch "$out"
          '';
          orchestration-doctrine-guardrails = pkgs.runCommand "skills-orchestration-doctrine-guardrails" { } ''
            orchestration=${cleanSource}/modules/orchestration/full.md
            claude_orchestration=${cleanSource}/modules/claude-orchestration/full.md
            index=${cleanSource}/manifests/module-dependencies.nota
            target_insertions=${cleanSource}/manifests/target-module-insertions.nota
            grep -F 'refuses direct task work' "$orchestration" >/dev/null
            grep -F 'It does not inspect files, command output, links, status, or systems directly.' "$orchestration" >/dev/null
            grep -F 'read-only Spirit queries' "$orchestration" >/dev/null
            grep -F 'Do not record, clarify, supersede, retire, mutate, subscribe, or perform Spirit maintenance as orchestrator.' "$orchestration" >/dev/null
            grep -F 'Route candidate durable intent' "$orchestration" >/dev/null
            if grep -F 'Capture durable intent' "$orchestration"; then
              echo "orchestration must route candidate durable intent, not say the orchestrator captures it directly" >&2
              exit 1
            fi
            grep -F "Be curious about the psyche's design intent without turning curiosity into permission seeking." "$orchestration" >/dev/null
            grep -F 'Ask focused clarification questions when the desired end shape, authority boundary, risk, privacy boundary, or acceptance criterion is unclear' "$orchestration" >/dev/null
            grep -F 'During design, push back by naming contradictions, weaker assumptions, hidden constraints, design tension, and better end shapes.' "$orchestration" >/dev/null
            grep -F 'Act when the psyche gives a concrete, scoped, authorized next step.' "$orchestration" >/dev/null
            grep -F 'Small reversible scout, inspection, read-only research, or worker-dispatch steps do not need separate alignment or method approval.' "$orchestration" >/dev/null
            grep -F 'Pause for destructive, private, irreversible, high-blast-radius, out-of-scope, credentialed, substantial implementation, durable doctrine, or genuinely ambiguous actions.' "$orchestration" >/dev/null
            grep -F 'Questions must be single-focus and unambiguous; avoid bundled yes/no questions where a short answer could be ambiguous.' "$orchestration" >/dev/null
            grep -F 'Confirm suspected interpretation with the psyche instead of silently assuming.' "$orchestration" >/dev/null
            grep -F 'Brief by default in interactive turns: state the question, decision, blocker, worker return, or next action that matters now.' "$orchestration" >/dev/null
            grep -F 'When a worker returns while other relevant workers are still running, emit only an extremely short interim note' "$orchestration" >/dev/null
            grep -F 'Treat the psyche as authority, bottleneck, and limited attention.' "$orchestration" >/dev/null
            if grep -F 'Ask at least one before proposing method or dispatching workers' "$orchestration"; then
              echo "orchestration must not require ritual clarification before clear small actions" >&2
              exit 1
            fi
            if grep -F 'Require two explicit psyche approvals' "$orchestration"; then
              echo "orchestration must not require fixed two-approval gates" >&2
              exit 1
            fi
            grep -F 'Use a tracker-weaver or weaver when work needs multiple beads, multiple repos, multiple workers, an audit phase, or durable tracker state.' "$orchestration" >/dev/null
            grep -F 'Do not use a weaver for a single small bounded fix with one worker and no tracking value.' "$orchestration" >/dev/null
            grep -F 'Match worker model and thinking level to work intensity' "$orchestration" >/dev/null
            grep -F 'small, faster, low-thinking workers for mechanical checks, commits, grep verification, and small renames' "$orchestration" >/dev/null
            grep -F 'normal implementation workers for ordinary implementation with local tests' "$orchestration" >/dev/null
            grep -F 'strongest, high-thinking workers for architecture, doctrine, privacy, intent, security, cross-repo plans, or ambiguous decisions' "$orchestration" >/dev/null
            grep -F 'Honor deliberate psyche-requested session or worker setup; when a lane intentionally requests a matching model, workers may use it.' "$orchestration" >/dev/null
            if grep -F 'never dispatch a worker on the `fable5` model' "$orchestration" "$claude_orchestration"; then
              echo "orchestration must not carry a global fable5 worker ban" >&2
              exit 1
            fi
            grep -F 'Use a separate auditor for substantial completed work, with strength matched to risk' "$orchestration" >/dev/null
            grep -F 'Keep context-handover separate and manual-load only' "$orchestration" >/dev/null
            grep -F '(orchestration modules/orchestration/full.md [spirit-query nota-design] RuntimeSkill)' "$index" >/dev/null
            grep -F 'Do not paste fixed commit or push protocols' "$orchestration" >/dev/null
            grep -F 'generated role packet already embeds the required doctrine' "$orchestration" >/dev/null
            grep -F '(claude-orchestration modules/claude-orchestration/full.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(orchestration ClaudeSkill [claude-orchestration])' "$target_insertions" >/dev/null
            grep -F '(orchestration ClaudeAgent [claude-orchestration])' "$target_insertions" >/dev/null
            grep -F 'Ask clarification in ordinary chat text instead of multiple-choice, picker, or' "$claude_orchestration" >/dev/null
            if grep -F 'Brief by default in interactive turns' "$claude_orchestration"; then
              echo "generic reply shape belongs in shared orchestration" >&2
              exit 1
            fi
            if grep -F 'When a worker returns while other relevant workers are still running' "$claude_orchestration"; then
              echo "generic interim worker return guidance belongs in shared orchestration" >&2
              exit 1
            fi
            if grep -F 'Claude orchestration surfaces' "$claude_orchestration"; then
              echo "Claude overlay should read naturally in generated surfaces" >&2
              exit 1
            fi
            if grep -F 'multiple-choice, picker, or' "$orchestration"; then
              echo "Claude UI preference belongs in the Claude target overlay, not shared orchestration" >&2
              exit 1
            fi
            touch "$out"
          '';
          role-composition-spirit-query = pkgs.runCommand "skills-role-composition-spirit-query" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            index=${cleanSource}/manifests/module-dependencies.nota
            grep -F '(spirit-query modules/spirit-query/full.md [nota-design] RuntimeSkill)' "$index" >/dev/null
            grep -F '(Skill (spirit-query spirit-query Meta Topic' "$manifest" >/dev/null
            for role in intent-translator scout repo-scaffolder general-code-implementer operating-system-implementer rust-auditor nix-auditor skill-editor intent-curator tracker-weaver; do
              grep -E "\\(Role \\($role [^]]*spirit-query[^]]*nota-design" "$manifest" >/dev/null || {
                echo "$role role must embed read-only Spirit query and NOTA design doctrine" >&2
                exit 1
              }
            done
            grep -E "\\(Role \\(repository-closeout [^]]*nota-design" "$manifest" >/dev/null || {
              echo "repository-closeout role must embed NOTA design doctrine" >&2
              exit 1
            }
            if grep -F '(Role (repository-closeout ' "$manifest" | grep -F 'spirit-query'; then
              echo "repository-closeout remains the mechanical closeout exemption and must not embed spirit-query" >&2
              exit 1
            fi
            touch "$out"
          '';
          active-appellations = pkgs.runCommand "skills-active-appellations" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            index=${cleanSource}/manifests/module-dependencies.nota
            for required in component-architecture design-quality version-control work-tracking intent-curator repository-closeout tracker-weaver; do
              grep -F "$required" "$manifest" >/dev/null || {
                echo "$required must be present in active output manifest" >&2
                exit 1
              }
              grep -F "$required" "$index" >/dev/null || true
            done
            for retired in component-triad beauty 'Skill (jj ' 'Skill (beads ' human-interaction intent-maintainer repo-operator weave-operator; do
              if grep -F "$retired" "$manifest"; then
                echo "$retired must not be an active output appellation" >&2
                exit 1
              fi
            done
            for retired_title in 'Repo Operator' 'Weave Operator' 'Intent Maintainer'; do
              if grep -R -F "$retired_title" ${cleanSource}/roles ${cleanSource}/modules; then
                echo "$retired_title must not appear as active current-destination prose" >&2
                exit 1
              fi
            done
            touch "$out"
          '';
          default = test;
        };

        devShells.default = pkgs.mkShell {
          name = "skills";
          packages = [
            pkgs.jujutsu
            toolchain
          ];
        };
      }
    );
}
