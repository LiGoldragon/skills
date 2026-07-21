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
          generation-requests-use-active-manifest =
            pkgs.runCommand "skills-generation-requests-use-active-manifest" { }
              ''
                grep -F 'manifests/active-outputs.nota' ${cleanSource}/skills-check.nota >/dev/null
                grep -F 'manifests/active-outputs.nota' ${cleanSource}/skills-generate.nota >/dev/null
                if find ${cleanSource}/manifests -mindepth 2 -type f -name '*.nota' | grep .; then
                  echo "generation must be driven by the single active output manifest, not per-output manifests" >&2
                  exit 1
                fi
                touch "$out"
              '';
          management-source-is-shared = pkgs.runCommand "skills-management-source-is-shared" { } ''
            management=${cleanSource}/modules/management/full.md
            index=${cleanSource}/manifests/module-dependencies.nota
            insertions=${cleanSource}/manifests/target-module-insertions.nota
            test -f "$management"
            expected=$TMPDIR/management-expected.md
            printf '%s\n' \
              'Align with the psyche’s vision.' \
              'Ask the psyche *until the vision is clear.*' \
              'Ask one clear question at a time.' \
              'Use subagents for all task work; if delegation fails, stop.' \
              'Read relevant skills directly.' \
              'Run subagents asynchronously.' \
              'Keep observations, hypotheses, and unknowns separate.' \
              'Keep unknown causes unknown.' \
              'Seek disconfirming evidence.' \
              'Do not seed audits with suspected conclusions.' \
              'Do not treat repeated claims as independent evidence.' \
              'Before disruptive work, state exactly what will change and what can break.' \
              'Get psyche approval before disruptive work.' \
              'Get psyche approval before every skill edit.' \
              'A question authorizes an answer, not a change.' \
              > "$expected"
            cmp "$expected" "$management"
            grep -F '(management modules/management/full.md [] RuntimeSkill)' "$index" >/dev/null
            test ! -e ${cleanSource}/modules/claude-management
            ! grep -F '(management Claude' "$insertions"
            touch "$out"
          '';
          human-interaction-removed-from-active-and-generated =
            pkgs.runCommand "skills-human-interaction-removed-from-active-and-generated" { }
              ''
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
          skill-editor-source-of-truth-guardrails =
            pkgs.runCommand "skills-skill-editor-source-of-truth-guardrails" { }
              ''
                for source in               ${cleanSource}/modules/skill-editor/full.md               ${cleanSource}/roles/skill-editor/full.md               ${cleanSource}/modules/skill-source-core/full.md; do
                  grep -F 'explicit psyche approval' "$source" >/dev/null
                  grep -F 'generated runtime' "$source" >/dev/null
                  grep -F 'Generate and verify' "$source" >/dev/null
                done
                touch "$out"
              '';
          manager-doctrine-guardrails = pkgs.runCommand "skills-manager-doctrine-guardrails" { } ''
            management=${cleanSource}/modules/management/full.md
            safeguards=${cleanSource}/modules/manager-safeguards/full.md
            boundary=${cleanSource}/modules/manager-boundary/full.md
            intent=${cleanSource}/modules/manager-intent-classification/full.md
            index=${cleanSource}/manifests/module-dependencies.nota
            insertions=${cleanSource}/manifests/target-module-insertions.nota
            grep -F 'Use subagents for all task work; if delegation fails, stop.' "$management" >/dev/null
            grep -F 'A question authorizes an answer, not a change.' "$management" >/dev/null
            grep -F 'Require explicit psyche approval before a host reboot.' "$safeguards" >/dev/null
            grep -F 'Delegate investigation and operations.' "$boundary" >/dev/null
            grep -F 'Keep requested rules, mechanisms, and architecture as matter.' "$intent" >/dev/null
            grep -F '(management modules/management/full.md [] RuntimeSkill)' "$index" >/dev/null
            test ! -e ${cleanSource}/modules/claude-management
            ! grep -F '(management Claude' "$insertions"
            workspace=$TMPDIR/workspace
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
            ${skillsPackage}/bin/skills ${cleanSource}/skills-check.nota >/dev/null
            cmp "$workspace/.agents/skills/management/SKILL.md" "$workspace/.claude/skills/management/SKILL.md"
            for packet in "$workspace/.pi/agents/manager.md" "$workspace/.claude/agents/manager.md" "$workspace/.codex/agents/manager.toml"; do
              grep -F 'Use subagents for all task work; if delegation fails, stop.' "$packet" >/dev/null
              grep -F 'A question authorizes an answer, not a change.' "$packet" >/dev/null
              grep -F 'Require explicit psyche approval before a host reboot.' "$packet" >/dev/null
              ! grep -Ei '@generated|generated by' "$packet"
            done
            touch "$out"
          '';
          slim-role-composition = pkgs.runCommand "skills-slim-role-composition" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            if grep -F '(Role (' "$manifest" | grep -E '\[[^]]*(spirit-query|nota-design)[^]]*\]'; then
              echo "roles must not preload broad Spirit or NOTA runtime skills" >&2
              exit 1
            fi
            grep -F '(Role (intent-recorder role-intent-recorder [spirit-submission]' "$manifest" >/dev/null
            grep -F '[general-instructions]' ${cleanSource}/manifests/universal-role-modules.nota >/dev/null
            grep -F '(Role (manager role-manager [management manager-boundary manager-intent-classification manager-safeguards manager-dispatch manager-liveness manager-decisions manager-communication manager-synthesis psyche-facing-commitments protos-syntax]' "$manifest" >/dev/null
            touch "$out"
          '';
          role-profile-manifests = pkgs.runCommand "skills-role-profile-manifests" { } ''
            model_catalog=${cleanSource}/manifests/model-catalog.nota
            role_assignments=${cleanSource}/manifests/role-model-assignments.nota
            grep -F '(ChatGpt (gpt-5.6-sol openai-codex [(Medium 50) (High 60)]))' "$model_catalog" >/dev/null
            grep -F '(ChatGpt (gpt-5.6-terra openai-codex [(Medium 20) (High 30) (Xhigh 40)]))' "$model_catalog" >/dev/null
            grep -F '(Claude (fable-5 [(Medium 50) (High 60)]))' "$model_catalog" >/dev/null
            grep -F '(Claude (claude-opus-4-8 [(High 30) (Xhigh 40)]))' "$model_catalog" >/dev/null
            grep -F '(Claude (claude-sonnet-5 [(Medium 10)]))' "$model_catalog" >/dev/null
            grep -F '(manager (gpt-5.6-sol High) (claude-opus-4-8 High))' "$role_assignments" >/dev/null
            grep -F '(generalist (gpt-5.6-terra Xhigh) (claude-opus-4-8 High))' "$role_assignments" >/dev/null
            grep -F '(intent-translator (gpt-5.6-terra Xhigh) (claude-opus-4-8 Xhigh))' "$role_assignments" >/dev/null
            grep -F '(operating-system-implementer (gpt-5.6-terra Xhigh) (claude-opus-4-8 High))' "$role_assignments" >/dev/null
            grep -F '(skill-editor (gpt-5.6-terra Xhigh) (claude-opus-4-8 Xhigh))' "$role_assignments" >/dev/null
            grep -F '(intent-curator (gpt-5.6-terra Xhigh) (claude-opus-4-8 Xhigh))' "$role_assignments" >/dev/null
            grep -F '(intent-recorder (gpt-5.6-luna Medium) (claude-sonnet-5 Medium))' "$role_assignments" >/dev/null
            grep -F '(scout (gpt-5.6-luna Medium) (claude-sonnet-5 Medium))' "$role_assignments" >/dev/null
            grep -F '(repository-closeout (gpt-5.6-luna Medium) (claude-sonnet-5 Medium))' "$role_assignments" >/dev/null
            if grep -R -F 'claude-sonnet-4-6' ${cleanSource}/manifests; then
              echo "Claude Sonnet roles must not regress to Sonnet 4.6" >&2
              exit 1
            fi
            grep -F '(manager [intent-clarification context-handover helper-context-transfer])' ${cleanSource}/manifests/role-optional-skills.nota >/dev/null
            touch "$out"
          '';
          active-appellations = pkgs.runCommand "skills-active-appellations" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            index=${cleanSource}/manifests/module-dependencies.nota
            for required in component-architecture design-quality version-control work-tracking management manager generalist intent-recorder intent-curator repository-closeout tracker-weaver; do
              grep -F "$required" "$manifest" >/dev/null || {
                echo "$required must be present in active output manifest" >&2
                exit 1
              }
              grep -F "$required" "$index" >/dev/null || true
            done
            grep -F '(Skill (management management ' "$manifest" >/dev/null
            if grep -F '(Skill (orchestration ' "$manifest"; then
              echo "orchestration must not be an active skill output" >&2
              exit 1
            fi
            for retired in component-triad beauty 'Skill (jj ' 'Skill (beads ' human-interaction 'Role (orchestrator ' intent-maintainer repo-operator weave-operator; do
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
