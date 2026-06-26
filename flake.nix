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
      url = "github:LiGoldragon/nota-next";
      flake = false;
    };
    schema-source = {
      url = "github:LiGoldragon/schema-next";
      flake = false;
    };
    schema-rust-source = {
      url = "github:LiGoldragon/schema-rust-next";
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
            chmod -R u+w "$out/vendor-sources"
            cat >> "$out/Cargo.toml" <<'EOF'

          [patch."https://github.com/LiGoldragon/nota-next.git"]
          nota = { path = "vendor-sources/nota" }
          nota-derive = { path = "vendor-sources/nota/derive" }

          [patch."https://github.com/LiGoldragon/schema-next.git"]
          schema = { path = "vendor-sources/schema" }
          schema-cc = { path = "vendor-sources/schema/schema-cc" }

          [patch."https://github.com/LiGoldragon/schema-rust-next.git"]
          schema-rust = { path = "vendor-sources/schema-rust" }
          EOF
        '';

        patchedCargoLock = pkgs.runCommand "skills-patched-Cargo.lock" { } ''
          ${pkgs.python3}/bin/python3 - ${./Cargo.lock} "$out" <<'PYEOF'
          import re
          import sys

          path_dependency_names = {"nota", "nota-derive", "schema", "schema-cc", "schema-rust"}
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
          generate-intent-led-orchestration = generatorApp "generate-intent-led-orchestration" "skills-generate.nota";
          check-intent-led-orchestration = generatorApp "check-intent-led-orchestration" "skills-check.nota";
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
          all-skill-manifests-configured = pkgs.runCommand "skills-all-skill-manifests-configured" { } ''
            while IFS= read -r manifest; do
              manifest_path="''${manifest#${cleanSource}/}"
              case "$manifest_path" in
                manifests/migration/*) continue ;;
              esac
              grep -F "$manifest_path" ${cleanSource}/skills-check.nota >/dev/null
              grep -F "$manifest_path" ${cleanSource}/skills-generate.nota >/dev/null
            done < <(find ${cleanSource}/manifests -type f -name '*.nota' | sort)
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
