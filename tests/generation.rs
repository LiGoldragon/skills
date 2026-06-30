use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use nota::NotaSource;
use skills::{
    Error,
    schema::assembly::{
        ActiveOutputs, EmissionPolicy, GenerationMode, GenerationRequest, ManifestPath,
        ModuleDependencies, ModuleKind, ModuleLifecycle, RoleTargetSurface, SkillRoster,
        SourceRoot, TargetSurface, WorkspaceRoot,
    },
};
use tempfile::TempDir;

#[test]
fn generation_writes_derived_skill_surfaces_with_roster_frontmatter() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "---\nname: stale\n---\n\n# Skill — example\n\n## Rule\n\nKeep the prose.\n",
    );

    let report = fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");

    let generated_paths: Vec<&str> = report
        .payload()
        .payload()
        .iter()
        .map(|file| file.output_path.as_ref())
        .collect();
    assert!(generated_paths.contains(&".agents/skills/example/SKILL.md"));
    assert!(generated_paths.contains(&".claude/skills/example/SKILL.md"));
    assert!(!generated_paths.contains(&"skills/skills.nota"));

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert_eq!(
        generated,
        "---\nname: example\ndescription: 'Example skill.'\n---\n\n# Skill — example\n\n## Rule\n\nKeep the prose.\n"
    );
    assert_eq!(
        generated,
        fixture.read_workspace_file(".claude/skills/example/SKILL.md")
    );
    assert!(!fixture.workspace.path().join("skills/skills.nota").exists());
}

#[test]
fn generation_allows_fenced_frontmatter_examples_inside_modules() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill — example\n\n## Rule\n\n```markdown\n---\nname: example\n---\n```\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("fenced frontmatter example is ordinary markdown");

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert!(generated.starts_with("---\nname: example\ndescription: 'Example skill.'\n---\n\n"));
    assert!(generated.contains("```markdown\n---\nname: example\n---\n```"));
}

#[test]
fn generation_rejects_second_unfenced_frontmatter_delimiter_in_skill() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill — example\n\n## Rule\n\n---\n\nKeep the prose.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("only the leading frontmatter delimiter pair is allowed");

    assert!(
        matches!(error, Error::NestedFrontmatter { .. }),
        "{error:?}"
    );
}

#[test]
fn generation_does_not_rebase_link_syntax_inside_code_spans() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill — example\n\n## Rule\n\nUse `[text](url)` only as a literal example.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("code span link syntax is preserved");

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert!(generated.contains("`[text](url)`"));
}

#[test]
fn generation_fails_on_duplicate_headings() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill — example\n\n## Repeat\n\nFirst.\n\n## Repeat\n\nSecond.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("duplicate headings fail");

    assert!(matches!(error, Error::DuplicateHeading { .. }), "{error:?}");
}

#[test]
fn roster_model_covers_current_skills_without_entrypoint_extras() {
    let text = include_str!("../manifests/skills-roster.nota");
    let roster = NotaSource::new(text)
        .parse::<SkillRoster>()
        .expect("roster model parses");

    assert_eq!(roster.archive_root.as_ref(), "skills/archive");
    assert_eq!(roster.skill_modules.payload().len(), 66);

    let active_first_class_modules: Vec<_> = roster
        .skill_modules
        .payload()
        .iter()
        .filter(|module| {
            matches!(module.module_lifecycle, ModuleLifecycle::Active(_))
                && module.emission_policy == EmissionPolicy::FirstClassSkill
        })
        .collect();
    assert_eq!(active_first_class_modules.len(), 52);
    for module in active_first_class_modules {
        assert_eq!(
            module.target_surfaces.payload(),
            &[TargetSurface::AgentsSkill, TargetSurface::ClaudeSkill]
        );
    }

    let active_internal_modules: Vec<_> = roster
        .skill_modules
        .payload()
        .iter()
        .filter(|module| {
            matches!(module.module_lifecycle, ModuleLifecycle::Active(_))
                && module.emission_policy == EmissionPolicy::InternalOnly
        })
        .map(|module| module.module_name.payload())
        .collect();
    assert_eq!(
        active_internal_modules,
        [
            "architectural-truth-tests",
            "rust-discipline",
            "bead-weaver",
        ]
    );

    let archived_role_names = [
        "operator",
        "designer",
        "schema-designer",
        "system-operator",
        "system-maintainer",
        "poet",
        "editor",
        "assistant",
        "counselor",
    ];
    for role_name in archived_role_names {
        let module = roster
            .skill_modules
            .payload()
            .iter()
            .find(|module| module.module_name.payload() == role_name)
            .expect("role module modeled");
        assert!(matches!(
            module.module_lifecycle,
            ModuleLifecycle::Archived(_)
        ));
        assert_eq!(module.emission_policy, EmissionPolicy::NoEmission);
        assert!(module.target_surfaces.payload().is_empty());
    }

    for deleted_name in ["subagent-session-workflow", "keep-working"] {
        let deleted = roster
            .skill_modules
            .payload()
            .iter()
            .find(|module| module.module_name.payload() == deleted_name)
            .unwrap_or_else(|| panic!("{deleted_name} deleted module modeled"));
        assert_eq!(deleted.module_lifecycle, ModuleLifecycle::Deleted);
        assert_eq!(deleted.emission_policy, EmissionPolicy::NoEmission);
        assert!(deleted.target_surfaces.payload().is_empty());
    }

    assert!(
        roster.entry_points.payload().is_empty(),
        "no entrypoint command/prompt extras are currently generated"
    );
}

#[test]
fn active_manifest_and_module_index_cover_current_skills_and_roles() {
    let active_outputs = NotaSource::new(include_str!("../manifests/active-outputs.nota"))
        .parse::<ActiveOutputs>()
        .expect("active output manifest parses");
    let module_dependencies =
        NotaSource::new(include_str!("../manifests/module-dependencies.nota"))
            .parse::<ModuleDependencies>()
            .expect("module dependency index parses");

    let skill_count = active_outputs
        .payload()
        .iter()
        .filter(|output| matches!(output, skills::schema::assembly::ActiveOutput::Skill(_)))
        .count();
    let role_count = active_outputs
        .payload()
        .iter()
        .filter(|output| matches!(output, skills::schema::assembly::ActiveOutput::Role(_)))
        .count();

    assert_eq!(skill_count, 52);
    assert_eq!(role_count, 10);

    let dependency_modules: BTreeSet<&str> = module_dependencies
        .payload()
        .iter()
        .map(|dependency| dependency.module_identifier.as_ref())
        .collect();
    let module_kinds: BTreeMap<&str, ModuleKind> = module_dependencies
        .payload()
        .iter()
        .map(|dependency| {
            (
                dependency.module_identifier.as_ref(),
                dependency.module_kind,
            )
        })
        .collect();
    let role_composition_modules = [
        "agent-output-protocol",
        "edit-coordination-core",
        "code-implementation-core",
        "rust-core",
        "nix-core",
        "intent-core",
        "repo-scaffold-core",
        "repo-operation-core",
        "skill-source-core",
        "architectural-truth-tests",
        "rust-discipline",
        "bead-weaver",
    ];
    for module_identifier in role_composition_modules {
        assert_eq!(
            module_kinds.get(module_identifier),
            Some(&ModuleKind::RoleComposition),
            "{module_identifier} is generator-only role composition"
        );
    }
    let active_roles: BTreeMap<&str, _> = active_outputs
        .payload()
        .iter()
        .filter_map(|output| match output {
            skills::schema::assembly::ActiveOutput::Role(role) => {
                Some((role.output_identifier.as_ref(), role))
            }
            skills::schema::assembly::ActiveOutput::Skill(_) => None,
        })
        .collect();
    let expected_roles: &[(&str, &str, &[&str])] = &[
        (
            "intent-translator",
            "role-intent-translator",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "bead-weaver",
            ],
        ),
        (
            "scout",
            "role-scout",
            &["agent-output-protocol", "edit-coordination-core"],
        ),
        (
            "repo-scaffolder",
            "role-repo-scaffolder",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "repo-scaffold-core",
                "code-implementation-core",
                "rust-core",
                "nix-core",
            ],
        ),
        (
            "general-code-implementer",
            "role-general-code-implementer",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "code-implementation-core",
                "rust-core",
                "nix-core",
                "operating-system-operations",
            ],
        ),
        (
            "criomos-implementer",
            "role-criomos-implementer",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "code-implementation-core",
                "nix-core",
                "operating-system-operations",
            ],
        ),
        (
            "rust-auditor",
            "role-rust-auditor",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "rust-core",
                "architectural-truth-tests",
            ],
        ),
        (
            "nix-auditor",
            "role-nix-auditor",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "nix-core",
            ],
        ),
        (
            "skill-editor",
            "role-skill-editor",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "skill-source-core",
            ],
        ),
        (
            "intent-maintainer",
            "role-intent-maintainer",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "intent-core",
            ],
        ),
        (
            "repo-operator",
            "role-repo-operator",
            &[
                "agent-output-protocol",
                "edit-coordination-core",
                "repo-operation-core",
            ],
        ),
    ];

    assert_eq!(active_roles.len(), expected_roles.len());
    for (output_identifier, module_identifier, included_modules) in expected_roles {
        let role = active_roles
            .get(output_identifier)
            .unwrap_or_else(|| panic!("{output_identifier} role output modeled"));
        assert_eq!(role.module_identifier.as_ref(), *module_identifier);
        assert_eq!(
            role.included_modules
                .payload()
                .iter()
                .map(|module| module.as_ref())
                .collect::<Vec<_>>(),
            *included_modules
        );
        assert_eq!(
            role.role_target_surfaces.payload(),
            &[
                RoleTargetSurface::ClaudeAgent,
                RoleTargetSurface::CodexAgent,
                RoleTargetSurface::PiAgent,
            ]
        );
        assert!(dependency_modules.contains(module_identifier));
        assert_eq!(
            module_kinds.get(module_identifier),
            Some(&ModuleKind::RoleSource),
            "{module_identifier} is a role source module"
        );
        for included_module in *included_modules {
            assert!(dependency_modules.contains(included_module));
        }
    }
}

#[test]
fn role_generation_expands_dependencies_in_order_and_writes_harness_paths() {
    let fixture = Fixture::new();
    fixture.write_role_generation_sources();
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\nGenerated-file notices stay out.\n",
    );
    fixture.write_source_file(
        "modules/shared/full.md",
        "# Module - shared\n\n## Shared Rule\n\nDependency first.\n",
    );
    fixture.write_source_file(
        "modules/feature/full.md",
        "# Module - feature\n\n## Feature Rule\n\nDependent second.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("role generation succeeds");

    let claude = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude.starts_with("---\nname: worker\ndescription: 'Worker role.'\n---\n\n"));
    assert!(claude.contains("# Role - worker"));
    assert!(claude.contains("Module - shared"));
    assert!(claude.contains("Module - feature"));
    assert!(claude.find("# Role - worker") < claude.find("Module - shared"));
    assert!(claude.find("Module - shared") < claude.find("Module - feature"));
    assert_eq!(claude.matches("Module - shared").count(), 1);
    assert!(!claude.contains("@generated"));
    assert!(!claude.contains("generated by"));

    let codex = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex.contains("name = \"worker\""));
    assert!(codex.contains("description = \"Worker role.\""));
    assert!(codex.contains("developer_instructions = \"# Role - worker"));
    assert!(codex.contains("Module - shared"));
    assert!(codex.contains("Module - feature"));
    assert!(
        codex.contains(
            "Skill-read de-duplication: A pasted <skill ...>...</skill> block is complete"
        )
    );
    assert!(!claude.contains("Skill-read de-duplication"));

    let pi = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi.starts_with("---\nname: worker\ndescription: 'Worker role.'\n---\n\n"));
    assert!(!pi.contains("Skill-read de-duplication"));

    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    assert!(inventory.contains(".claude/agents/worker.md"));
    assert!(inventory.contains(".codex/agents/worker.toml"));
    assert!(inventory.contains(".pi/agents/worker.md"));
}

#[test]
fn generation_rejects_direct_module_dependency_cycle() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example modules/example/full.md [example] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("direct dependency cycle fails generation");

    assert!(
        matches!(
            error,
            Error::ModuleDependencyCycle {
                ref module_identifiers
            } if module_identifiers
                .iter()
                .map(String::as_str)
                .eq(["example", "example"])
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("example -> example"));
}

#[test]
fn generation_rejects_transitive_module_dependency_cycle() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example first Craft Topic [Example skill.] [AgentsSkill]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(first modules/first/full.md [second] RuntimeSkill) (second modules/second/full.md [third] RuntimeSkill) (third modules/third/full.md [second] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("transitive dependency cycle fails generation");

    assert!(
        matches!(
            error,
            Error::ModuleDependencyCycle {
                ref module_identifiers
            } if module_identifiers
                .iter()
                .map(String::as_str)
                .eq(["second", "third", "second"])
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("second -> third -> second"));
}

#[test]
fn generation_rejects_duplicate_role_output_paths_before_write() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [] [Worker role.] [ClaudeAgent ClaudeAgent]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker/full.md [] RoleSource)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("duplicate role output path fails before rendering");

    assert!(
        matches!(
            error,
            Error::DuplicateOutputPath {
                ref relative_path,
                ..
            } if relative_path == ".claude/agents/worker.md"
        ),
        "{error:?}"
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/agents/worker.md")
            .exists()
    );
}

#[test]
fn generation_rejects_role_composition_module_as_skill_output() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (edit-coordination-core edit-coordination-core Workflow Mechanism [Internal role component.] [AgentsSkill]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(edit-coordination-core modules/edit-coordination-core/full.md [] RoleComposition)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("role composition modules do not emit as skills");

    assert!(
        matches!(
            error,
            Error::InvalidModuleKind {
                ref module_identifier,
                ref expected,
                ref actual,
            } if module_identifier == "edit-coordination-core"
                && expected == "RuntimeSkill"
                && actual == "RoleComposition"
        ),
        "{error:?}"
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join(".agents/skills/edit-coordination-core/SKILL.md")
            .exists()
    );
}

#[test]
fn generation_rejects_runtime_module_as_role_source() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [] [Worker role.] [ClaudeAgent]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker/full.md [] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("role source modules are typed separately");

    assert!(
        matches!(
            error,
            Error::InvalidModuleKind {
                ref module_identifier,
                ref expected,
                ref actual,
            } if module_identifier == "worker"
                && expected == "RoleSource"
                && actual == "RuntimeSkill"
        ),
        "{error:?}"
    );
}

#[test]
fn write_mode_removes_only_inventory_owned_stale_role_outputs() {
    let fixture = Fixture::new();
    fixture.write_role_generation_sources();
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\nBody.\n",
    );
    fixture.write_source_file(
        "modules/shared/full.md",
        "# Module - shared\n\n## Shared Rule\n\nBody.\n",
    );
    fixture.write_source_file(
        "modules/feature/full.md",
        "# Module - feature\n\n## Feature Rule\n\nBody.\n",
    );
    fixture.write_workspace_file(
        "skills/generated-role-outputs.nota",
        "[.claude/agents/old.md]\n",
    );
    fixture.write_workspace_file(".claude/agents/old.md", "stale generated role\n");
    fixture.write_workspace_file(".claude/agents/human.md", "human-owned role\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("write mode prunes stale inventory-owned role path");

    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/agents/old.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".claude/agents/human.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".claude/agents/worker.md")
            .exists()
    );
}

#[test]
fn check_mode_reports_stale_output_with_guidance() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill — example\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/example/SKILL.md", "old\n");
    fixture.write_workspace_file(".claude/skills/example/SKILL.md", "old\n");
    fixture.write_workspace_file("skills/skills.nota", "old\n");

    let error = fixture
        .generate(GenerationMode::Check)
        .expect_err("stale output fails check mode");

    assert!(matches!(error, Error::StaleOutput { .. }), "{error:?}");
    assert!(!error.to_string().contains("skills.nota"));
    assert!(error.to_string().contains("generate-skills"));
    assert!(error.to_string().contains("check-skills"));
}

#[test]
fn generation_rejects_skill_with_oversized_serialized_block() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        &format!("# Skill — example\n\n## Rule\n\n{}\n", "x".repeat(33_000)),
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("oversized serialized skill block fails generation");

    assert!(
        matches!(
            error,
            Error::GeneratedSkillBlockTooLarge {
                ref skill_name,
                ref location,
                byte_count,
                limit,
            } if skill_name == "example"
                && location == ".agents/skills/example/SKILL.md"
                && byte_count > limit
                && limit == 32 * 1024
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("generated skill `example`"));
    assert!(error.to_string().contains("exceeding the 32768 byte limit"));
    assert!(
        !fixture
            .workspace
            .path()
            .join(".agents/skills/example/SKILL.md")
            .exists()
    );
}

#[test]
fn check_mode_reports_archived_or_deleted_stale_skill_outputs() {
    let fixture = Fixture::new();
    fixture.write_legacy_roster(
        "(skills/archive [(old modules/old/full.md Deleted NoEmission [])] [])\n",
    );
    fixture.write_workspace_file("skills/skills.nota", "old retired index\n");
    fixture.write_workspace_file(".agents/skills/old/SKILL.md", "stale\n");

    let error = fixture
        .generate_with_manifest(GenerationMode::Check, "manifests/skills-roster.nota")
        .expect_err("stale deleted output fails check mode");

    assert!(
        matches!(error, Error::StaleGeneratedOutput { .. }),
        "{error:?}"
    );
    assert!(!error.to_string().contains("skills.nota"));
    assert!(error.to_string().contains("archived/deleted"));
    assert!(error.to_string().contains("generate-skills"));
}

#[test]
fn check_mode_accepts_current_outputs_with_orphaned_retired_skill_index() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current generated outputs write to fixture workspace");
    fixture.write_workspace_file("skills/skills.nota", "old retired index\n");

    fixture
        .generate_from_repo(GenerationMode::Check)
        .expect("retired skill index is neither generated nor stale");

    assert_eq!(
        fixture.read_workspace_file("skills/skills.nota"),
        "old retired index\n"
    );
}

#[test]
fn write_mode_prunes_generated_skill_directories_before_writing() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill — example\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/old/SKILL.md", "stale\n");
    fixture.write_workspace_file(".claude/skills/old/SKILL.md", "stale\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("write mode prunes stale generated skill dirs");

    assert!(
        !fixture
            .workspace
            .path()
            .join(".agents/skills/old/SKILL.md")
            .exists()
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/skills/old/SKILL.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".agents/skills/example/SKILL.md")
            .exists()
    );
}

struct Fixture {
    source: TempDir,
    workspace: TempDir,
}

impl Fixture {
    fn new() -> Self {
        Self {
            source: TempDir::new().expect("source tempdir"),
            workspace: TempDir::new().expect("workspace tempdir"),
        }
    }

    fn write_default_roster(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill ClaudeSkill]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(example modules/example/full.md [] RuntimeSkill)]\n",
        );
    }

    fn write_legacy_roster(&self, text: &str) {
        self.write_source_file("manifests/skills-roster.nota", text);
    }

    fn write_role_generation_sources(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (worker worker [shared feature] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(worker roles/worker/full.md [] RoleSource) (shared modules/shared/full.md [] RoleComposition) (feature modules/feature/full.md [shared] RoleComposition)]\n",
        );
    }

    fn write_source_file(&self, path: &str, text: &str) {
        self.write_file(self.source.path(), path, text);
    }

    fn write_workspace_file(&self, path: &str, text: &str) {
        self.write_file(self.workspace.path(), path, text);
    }

    fn read_workspace_file(&self, path: &str) -> String {
        fs::read_to_string(self.workspace.path().join(path)).expect("read workspace file")
    }

    fn write_file(&self, root: &Path, path: &str, text: &str) {
        let full_path = root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(full_path, text).expect("write fixture file");
    }

    fn generate(
        &self,
        generation_mode: GenerationMode,
    ) -> Result<skills::schema::assembly::GenerationReport, Error> {
        GenerationRequest {
            source_root: SourceRoot::new(self.source.path().to_string_lossy().into_owned()),
            workspace_root: WorkspaceRoot::new(
                self.workspace.path().to_string_lossy().into_owned(),
            ),
            manifest_path: ManifestPath::new("manifests/active-outputs.nota"),
            generation_mode,
        }
        .generate()
    }

    fn generate_with_manifest(
        &self,
        generation_mode: GenerationMode,
        manifest_path: &str,
    ) -> Result<skills::schema::assembly::GenerationReport, Error> {
        GenerationRequest {
            source_root: SourceRoot::new(self.source.path().to_string_lossy().into_owned()),
            workspace_root: WorkspaceRoot::new(
                self.workspace.path().to_string_lossy().into_owned(),
            ),
            manifest_path: ManifestPath::new(manifest_path),
            generation_mode,
        }
        .generate()
    }

    fn generate_from_repo(
        &self,
        generation_mode: GenerationMode,
    ) -> Result<skills::schema::assembly::GenerationReport, Error> {
        GenerationRequest {
            source_root: SourceRoot::new(env!("CARGO_MANIFEST_DIR")),
            workspace_root: WorkspaceRoot::new(
                self.workspace.path().to_string_lossy().into_owned(),
            ),
            manifest_path: ManifestPath::new("manifests/active-outputs.nota"),
            generation_mode,
        }
        .generate()
    }
}
