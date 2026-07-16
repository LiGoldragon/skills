use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use nota::NotaSource;
use skills::{
    Error,
    schema::assembly::{
        ActiveOutputs, EmissionPolicy, GenerationMode, GenerationRequest, ManifestPath,
        ModelCatalog, ModuleDependencies, ModuleKind, ModuleLifecycle, RoleModelAssignments,
        RoleOptionalSkills, RoleTargetSurface, SkillRoster, SourceRoot, TargetModuleInsertions,
        TargetSurface, UniversalRoleModules, WorkspaceRoot,
    },
    trunk_guard::{TrunkDescendantGuard, TrunkDivergence},
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
        "---\nname: example\ndescription: 'Example skill.'\n---\n\n# example\n\n## Rule\n\nKeep the prose.\n"
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
    assert_eq!(roster.skill_modules.payload().len(), 76);

    let active_first_class_modules: Vec<_> = roster
        .skill_modules
        .payload()
        .iter()
        .filter(|module| {
            matches!(module.module_lifecycle, ModuleLifecycle::Active(_))
                && module.emission_policy == EmissionPolicy::FirstClassSkill
        })
        .collect();
    assert_eq!(active_first_class_modules.len(), 62);
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
        let archived_source = fs::read_to_string(format!("skills/archive/{role_name}.md"))
            .unwrap_or_else(|error| panic!("{role_name} archive source is readable: {error}"));
        assert!(
            archived_source.contains("Deprecated: this archived prior-workflow appellation is not a current handoff role or subagent destination."),
            "{role_name} archive source marks the appellation deprecated"
        );
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
    let target_module_insertions =
        NotaSource::new(include_str!("../manifests/target-module-insertions.nota"))
            .parse::<TargetModuleInsertions>()
            .expect("target module insertion index parses");
    let universal_role_modules =
        NotaSource::new(include_str!("../manifests/universal-role-modules.nota"))
            .parse::<UniversalRoleModules>()
            .expect("universal role module manifest parses");
    let model_catalog = NotaSource::new(include_str!("../manifests/model-catalog.nota"))
        .parse::<ModelCatalog>()
        .expect("model catalog parses");
    let role_model_assignments =
        NotaSource::new(include_str!("../manifests/role-model-assignments.nota"))
            .parse::<RoleModelAssignments>()
            .expect("role model assignments parse");
    let role_optional_skills =
        NotaSource::new(include_str!("../manifests/role-optional-skills.nota"))
            .parse::<RoleOptionalSkills>()
            .expect("role optional skills parse");

    // These hardcoded generation expectations intentionally catch membership drift.
    // Update them when module membership, role includes, or universal role modules change.
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

    assert_eq!(skill_count, 64);
    assert_eq!(role_count, 14);
    assert_eq!(model_catalog.payload().len(), 5);
    assert_eq!(role_model_assignments.payload().len(), role_count);
    assert_eq!(role_optional_skills.payload().len(), role_count);

    let model_catalog_source = include_str!("../manifests/model-catalog.nota");
    let role_model_assignments_source = include_str!("../manifests/role-model-assignments.nota");
    assert!(model_catalog_source.contains("(Claude (claude-sonnet-5 [Medium]))"));
    for sonnet_role in ["intent-recorder", "scout", "repository-closeout"] {
        assert!(
            role_model_assignments_source.contains(&format!(
                "({sonnet_role} (gpt-5.6-luna Medium) (claude-sonnet-5 Medium))"
            )),
            "{sonnet_role} uses Claude Sonnet 5"
        );
    }
    assert!(!model_catalog_source.contains("claude-sonnet-4-6"));
    assert!(!role_model_assignments_source.contains("claude-sonnet-4-6"));

    let active_skill_identifiers: BTreeSet<&str> = active_outputs
        .payload()
        .iter()
        .filter_map(|output| match output {
            skills::schema::assembly::ActiveOutput::Skill(skill) => {
                Some(skill.output_identifier.as_ref())
            }
            skills::schema::assembly::ActiveOutput::Role(_) => None,
        })
        .collect();
    for required_skill in [
        "component-architecture",
        "design-quality",
        "version-control",
        "work-tracking",
        "repository-publication",
        "pi-extension-updates",
        "nota-shape-checklist",
        "management",
    ] {
        assert!(
            active_skill_identifiers.contains(required_skill),
            "{required_skill} active skill uses approved appellation"
        );
    }
    for deprecated_skill in [
        "component-triad",
        "beauty",
        "jj",
        "beads",
        "human-interaction",
        "agent-feedback-loop",
        "orchestration",
    ] {
        assert!(
            !active_skill_identifiers.contains(deprecated_skill),
            "{deprecated_skill} active skill appellation stays retired or removed"
        );
    }

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
        "agent-feedback-loop",
        "edit-coordination-core",
        "editing-closeout",
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
        "return-to-manager",
        "spirit-submission",
    ];
    for module_identifier in role_composition_modules {
        assert_eq!(
            module_kinds.get(module_identifier),
            Some(&ModuleKind::RoleComposition),
            "{module_identifier} is generator-only role composition"
        );
    }
    assert_eq!(
        module_kinds.get("spirit-query"),
        Some(&ModuleKind::RuntimeSkill),
        "spirit-query remains a first-class read-only skill"
    );
    assert!(
        !dependency_modules.contains("human-interaction"),
        "human-interaction is deleted from the dependency index"
    );
    let spirit_query_dependency = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "spirit-query")
        .expect("spirit-query dependency indexed");
    assert_eq!(
        spirit_query_dependency
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["nota-design"]
    );
    let management_dependency = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "management")
        .expect("management dependency indexed");
    assert!(
        management_dependency
            .dependency_modules
            .payload()
            .is_empty()
    );
    for nota_module in ["nota-design", "nota-schema-design", "nota-literacy"] {
        let dependency = module_dependencies
            .payload()
            .iter()
            .find(|dependency| dependency.module_identifier.as_ref() == nota_module)
            .unwrap_or_else(|| panic!("{nota_module} dependency indexed"));
        assert!(
            dependency
                .dependency_modules
                .payload()
                .iter()
                .any(|module| module.as_ref() == "nota-shape-checklist"),
            "{nota_module} includes nota-shape-checklist"
        );
    }
    assert!(
        !management_dependency
            .dependency_modules
            .payload()
            .iter()
            .any(|module| module.as_ref() == "context-handover"),
        "context-handover remains separate/manual-load only"
    );
    assert_eq!(
        target_module_insertions
            .payload()
            .iter()
            .map(|insertion| (
                insertion.module_identifier.as_ref(),
                insertion.output_surface,
                insertion
                    .included_modules
                    .payload()
                    .iter()
                    .map(|module| module.as_ref())
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>(),
        [
            (
                "management",
                skills::schema::assembly::OutputSurface::ClaudeSkill,
                vec!["claude-management"]
            ),
            (
                "management",
                skills::schema::assembly::OutputSurface::ClaudeAgent,
                vec!["claude-management"]
            ),
        ]
    );
    assert_eq!(
        universal_role_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["agent-feedback-loop", "return-to-manager"]
    );

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
        ("manager", "role-manager", &["management"]),
        (
            "generalist",
            "role-generalist",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
            ],
        ),
        (
            "intent-recorder",
            "role-intent-recorder",
            &["spirit-submission"],
        ),
        (
            "intent-translator",
            "role-intent-translator",
            &["edit-coordination-core", "bead-weaver"],
        ),
        ("scout", "role-scout", &["edit-coordination-core"]),
        (
            "repo-scaffolder",
            "role-repo-scaffolder",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "repo-scaffold-core",
                "code-implementation-core",
            ],
        ),
        (
            "general-code-implementer",
            "role-general-code-implementer",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
            ],
        ),
        (
            "operating-system-implementer",
            "role-operating-system-implementer",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
                "nix-core",
                "operating-system-operations",
                "nixos-vm-testing",
            ],
        ),
        (
            "rust-auditor",
            "role-rust-auditor",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "rust-core",
                "architectural-truth-tests",
            ],
        ),
        (
            "nix-auditor",
            "role-nix-auditor",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "nix-core",
                "nixos-vm-testing",
            ],
        ),
        (
            "skill-editor",
            "role-skill-editor",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "skill-source-core",
            ],
        ),
        (
            "intent-curator",
            "role-intent-curator",
            &["edit-coordination-core", "editing-closeout", "intent-core"],
        ),
        (
            "repository-closeout",
            "role-repository-closeout",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "repo-operation-core",
            ],
        ),
        (
            "tracker-weaver",
            "role-tracker-weaver",
            &["edit-coordination-core", "editing-closeout", "bead-weaver"],
        ),
    ];

    assert_eq!(active_roles.len(), expected_roles.len());
    for deprecated_role in ["intent-maintainer", "repo-operator", "weave-operator"] {
        assert!(
            !active_roles.contains_key(deprecated_role),
            "{deprecated_role} active role appellation stays retired"
        );
    }
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
fn human_interaction_is_removed_and_context_handover_stays_manual_load() {
    let manifest_text = include_str!("../manifests/active-outputs.nota");
    let index_text = include_str!("../manifests/module-dependencies.nota");

    assert!(!manifest_text.contains("human-interaction"));
    assert!(!index_text.contains("human-interaction"));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("modules/human-interaction/full.md")
            .exists(),
        "human-interaction source module is deleted, not archived"
    );

    let module_dependencies = NotaSource::new(index_text)
        .parse::<ModuleDependencies>()
        .expect("module dependency index parses");
    let management = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "management")
        .expect("management dependency indexed");
    assert!(management.dependency_modules.payload().is_empty());
    assert!(manifest_text.contains("(Skill (context-handover context-handover Meta Mechanism"));
    assert!(
        !management
            .dependency_modules
            .payload()
            .iter()
            .any(|module| module.as_ref() == "context-handover")
    );
}

#[test]
fn skill_editor_doctrine_names_canonical_source_and_generated_targets() {
    let skill_module = include_str!("../modules/skill-editor/full.md");
    let role_module = include_str!("../roles/skill-editor/full.md");
    let skill_source_core = include_str!("../modules/skill-source-core/full.md");

    for source_text in [skill_module, role_module] {
        assert!(source_text.contains("`LiGoldragon/skills` as the canonical skills source"));
        assert!(source_text.contains("source modules"));
        assert!(source_text.contains("role source"));
        assert!(source_text.contains("generation data"));
        for generated_target in [
            ".agents/skills",
            ".claude/skills",
            ".pi/agents",
            ".codex/agents",
        ] {
            assert!(
                source_text.contains(generated_target),
                "skill-editor source identifies {generated_target} as generated"
            );
        }
        assert!(source_text.contains("generated runtime targets"));
        assert!(!source_text.contains("generated runtime copies first"));
    }

    assert!(skill_source_core.contains("`LiGoldragon/skills` as the canonical skills source"));
    assert!(skill_source_core.contains("generator inputs"));
    assert!(skill_source_core.contains("source modules"));
    assert!(skill_source_core.contains("role source modules"));
    assert!(skill_source_core.contains("generated runtime targets"));
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ForkStatus {
    FullyAbsorbed,
    PartiallyAbsorbed,
    StillAbsent,
    DeliberatelyDivergent,
    Unknown,
}

impl ForkStatus {
    fn parse(value: &str) -> Self {
        match value {
            "fully absorbed" => Self::FullyAbsorbed,
            "partially absorbed" => Self::PartiallyAbsorbed,
            "still absent" => Self::StillAbsent,
            "deliberately divergent" => Self::DeliberatelyDivergent,
            "unknown" => Self::Unknown,
            _ => panic!("unknown fork status: {value}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ForkDecision {
    Rebase,
    Reimplement,
    Drop,
    Escalate,
}

impl ForkDecision {
    fn parse(value: &str) -> Self {
        match value {
            "rebase" => Self::Rebase,
            "reimplement" => Self::Reimplement,
            "drop" => Self::Drop,
            "escalate" => Self::Escalate,
            _ => panic!("unknown fork decision: {value}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ForkDecisionState {
    Final,
    Provisional,
}

impl ForkDecisionState {
    fn parse(value: &str) -> Self {
        match value {
            "final" => Self::Final,
            "provisional" => Self::Provisional,
            _ => panic!("unknown fork decision state: {value}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WitnessTree {
    Pristine,
    Reconciled,
}

impl WitnessTree {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pristine => "pristine",
            Self::Reconciled => "reconciled",
        }
    }
}

#[derive(Debug)]
struct ParsedWitness {
    name: String,
    result: BTreeMap<String, String>,
}

#[derive(Debug)]
struct ForkDeltaRecord {
    delta: String,
    status: ForkStatus,
    decision: ForkDecision,
    pristine: ParsedWitness,
    reconciled: ParsedWitness,
    state: ForkDecisionState,
}

impl ForkDeltaRecord {
    fn parse(row: &str) -> Self {
        let columns: Vec<_> = row.trim_matches('|').split('|').map(str::trim).collect();
        assert_eq!(
            columns.len(),
            10,
            "delta row keeps the required ten fields: {row}"
        );

        let delta = inline_code(columns[0]).to_owned();
        let rationale = inline_code(columns[1]);
        assert_eq!(
            rationale.len(),
            40,
            "delta rationale uses an immutable commit: {row}"
        );
        assert!(
            rationale
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
            "delta rationale is hexadecimal: {row}"
        );
        assert!(
            !columns[2].is_empty(),
            "delta records implementation location: {row}"
        );

        let status = ForkStatus::parse(columns[3]);
        let decision = ForkDecision::parse(columns[4]);
        let pristine = parse_witness(columns[5], columns[6], WitnessTree::Pristine);
        let reconciled = parse_witness(columns[7], columns[8], WitnessTree::Reconciled);
        assert_eq!(
            pristine.name, reconciled.name,
            "witness pair must name the same delta gate: {row}"
        );
        match decision {
            ForkDecision::Drop => assert_eq!(
                pristine.result, reconciled.result,
                "a dropped absorbed delta remains unchanged: {row}"
            ),
            ForkDecision::Reimplement => assert_ne!(
                pristine.result, reconciled.result,
                "a reimplemented delta must record changed results: {row}"
            ),
            ForkDecision::Rebase | ForkDecision::Escalate => {}
        }

        Self {
            delta,
            status,
            decision,
            pristine,
            reconciled,
            state: ForkDecisionState::parse(columns[9]),
        }
    }
}

fn inline_code(value: &str) -> &str {
    value
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .filter(|value| !value.is_empty() && !value.contains('`'))
        .unwrap_or_else(|| panic!("expected one non-empty inline-code value: {value}"))
}

fn parse_witness(command_cell: &str, result_cell: &str, tree: WitnessTree) -> ParsedWitness {
    const SCRIPT: &str = "packages/pi-subagents/reconciliation/verify-0.34.0.sh";

    let command = inline_code(command_cell);
    let tokens: Vec<_> = command.split_whitespace().collect();
    assert_eq!(
        tokens.len(),
        4,
        "witness command has exactly script, subcommand, witness, and tree: {command}"
    );
    assert_eq!(tokens[0], SCRIPT, "witness uses the retained executable");
    assert_eq!(tokens[1], "witness", "witness uses the witness subcommand");
    assert!(
        !tokens[2].is_empty()
            && tokens[2]
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-'),
        "witness name is a canonical command argument: {command}"
    );
    assert_eq!(
        tokens[3],
        tree.as_str(),
        "witness command carries the explicit tree argument"
    );

    let mut result = BTreeMap::new();
    for component in inline_code(result_cell).split("; ") {
        let (key, value) = component
            .split_once('=')
            .unwrap_or_else(|| panic!("witness result component must be key=value: {component}"));
        assert!(
            !key.is_empty()
                && key
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character == '-'),
            "witness result key is canonical: {key}"
        );
        assert!(
            !value.is_empty()
                && (value.chars().all(|character| character.is_ascii_digit()) || value == "pass"),
            "witness result value is an exit/count or pass token: {value}"
        );
        assert!(
            result.insert(key.to_owned(), value.to_owned()).is_none(),
            "witness result key appears once: {key}"
        );
    }
    assert_eq!(
        result.get("exit").map(String::as_str),
        Some("0"),
        "retained witness command itself exits successfully"
    );
    assert!(
        result.len() >= 2,
        "witness records command exit plus at least one component result"
    );

    ParsedWitness {
        name: tokens[2].to_owned(),
        result,
    }
}

fn canonical_ledger_path() -> (PathBuf, bool) {
    const LEDGER_RELATIVE_PATH: &str = "packages/pi-subagents/fork-delta-ledger.md";

    if let Some(path) = env::var_os("PI_SUBAGENTS_CANONICAL_LEDGER") {
        let path = PathBuf::from(path);
        assert!(
            path.ends_with(LEDGER_RELATIVE_PATH),
            "PI_SUBAGENTS_CANONICAL_LEDGER must name the owning package ledger"
        );
        return (path, true);
    }

    let sibling = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("skills checkout has a parent")
        .join("CriomOS-home")
        .join(LEDGER_RELATIVE_PATH);
    if sibling.is_file() {
        return (sibling, true);
    }

    (
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pi-subagents-canonical-ledger.md"),
        false,
    )
}

fn sha256_file(path: &Path) -> String {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run sha256sum for {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "sha256sum failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("sha256sum output is UTF-8")
        .split_whitespace()
        .next()
        .expect("sha256sum emits a digest")
        .to_owned()
}

#[test]
fn pi_extension_update_protocol_covers_fork_reconciliation_and_real_fixture() {
    let protocol = include_str!("../modules/pi-extension-updates/full.md");
    for required in [
        "maintained-fork reconciliation",
        "primary/live sources and recent upstream activity",
        "mechanisms and tests from upstream",
        "local compatibility",
        "semantic Jujutsu patch stack",
        "must never blind-merge",
        "pristine target and reconciled result",
        "Push the producer revision before updating a consumer pin",
        "upstream, drop, carry/reimplement, or escalate",
        "fork retires",
        "Re-audit whenever upstream activity",
    ] {
        assert!(
            protocol.contains(required),
            "missing Pi extension update rule: {required}"
        );
    }

    let fixture = include_str!("fixtures/pi-subagents-0.31.0-to-0.34.0.md");
    for required in [
        "## Canonical ledger",
        "CriomOS-home/packages/pi-subagents/fork-delta-ledger.md",
        "## Immutable candidate",
        "e4f06282d0c95856b36b7ec2893f4fd294ebfefe",
        "8a6c5b154f7df63b65c6027ba41ea7c6496d60db",
        "12a157d2a70b2f4cbc004c020c5f9213b6d8eea8",
        "## Delta records",
        "## Applicability evidence",
        "patch --dry-run --forward --batch --verbose",
        "Reversed notices are never counted as application.",
        "## Evidence gates",
        "108 passed, 0 failed",
        "981 total, 978 passed, 3 failed",
        "985 total, 982 passed, same 3 failed",
        "Nix candidate package build",
        "Nix package-content witness",
        "Nix Pi RPC extension-load witness",
        "## Decision status",
        "No decision is final.",
        "not a psyche authority, privacy, or value decision",
    ] {
        assert!(
            fixture.contains(required),
            "fixture missing evidence semantic: {required}"
        );
    }

    let ledger_digest = fixture
        .lines()
        .find_map(|line| line.strip_prefix("- SHA-256: `")?.strip_suffix("`."))
        .expect("fixture carries canonical ledger digest");
    assert_eq!(ledger_digest.len(), 64);
    assert!(
        ledger_digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );

    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pi-subagents-canonical-ledger.md");
    assert_eq!(
        sha256_file(&snapshot_path),
        ledger_digest,
        "retained canonical snapshot drift requires a deliberate fixture digest update"
    );
    let (canonical_path, uses_owning_checkout) = canonical_ledger_path();
    assert_eq!(
        sha256_file(&canonical_path),
        ledger_digest,
        "canonical ledger drift requires a deliberate fixture and snapshot update"
    );
    if uses_owning_checkout {
        let repository_root = canonical_path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("canonical ledger has packages/pi-subagents ancestry");
        let verifier =
            repository_root.join("packages/pi-subagents/reconciliation/verify-0.34.0.sh");
        assert!(verifier.is_file(), "canonical witness executable exists");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(
                fs::metadata(&verifier)
                    .expect("canonical witness metadata is readable")
                    .permissions()
                    .mode()
                    & 0o111,
                0,
                "canonical witness is executable"
            );
        }
    }

    let expected_deltas = BTreeMap::from([
        ("acceptance-read-only-evidence.patch", "read-only-evidence"),
        ("agent-chain-clarify-opt-in.patch", "clarify"),
        ("async-runner-stderr.patch", "stderr-compaction"),
        ("detached-runner-peer-isolation.patch", "peer-isolation"),
        ("full-child-extension-bridge.patch", "child-extension"),
        ("slim-parent-skill.patch", "compact-skill"),
    ]);
    let records: Vec<_> = fixture
        .lines()
        .filter(|line| line.starts_with("| `") && line.contains(".patch` |"))
        .map(ForkDeltaRecord::parse)
        .collect();
    assert_eq!(records.len(), expected_deltas.len());

    let mut observed_deltas = BTreeSet::new();
    let mut commands = BTreeSet::new();
    let mut status_counts = BTreeMap::new();
    let mut decision_counts = BTreeMap::new();
    let mut state_counts = BTreeMap::new();
    for record in &records {
        let expected_witness = expected_deltas
            .get(record.delta.as_str())
            .unwrap_or_else(|| panic!("unexpected delta row: {}", record.delta));
        assert!(
            observed_deltas.insert(record.delta.as_str()),
            "duplicate delta row: {}",
            record.delta
        );
        assert_eq!(&record.pristine.name, expected_witness);
        assert_eq!(&record.reconciled.name, expected_witness);
        assert!(
            commands.insert((
                record.pristine.name.as_str(),
                WitnessTree::Pristine.as_str()
            )),
            "duplicate pristine witness: {}",
            record.pristine.name
        );
        assert!(
            commands.insert((
                record.reconciled.name.as_str(),
                WitnessTree::Reconciled.as_str()
            )),
            "duplicate reconciled witness: {}",
            record.reconciled.name
        );
        *status_counts.entry(record.status).or_insert(0) += 1;
        *decision_counts.entry(record.decision).or_insert(0) += 1;
        *state_counts.entry(record.state).or_insert(0) += 1;
    }
    assert_eq!(observed_deltas.len(), expected_deltas.len());
    assert_eq!(commands.len(), expected_deltas.len() * 2);
    assert_eq!(
        status_counts,
        BTreeMap::from([
            (ForkStatus::PartiallyAbsorbed, 3),
            (ForkStatus::StillAbsent, 2),
            (ForkStatus::FullyAbsorbed, 1),
        ])
    );
    assert_eq!(
        decision_counts,
        BTreeMap::from([(ForkDecision::Reimplement, 5), (ForkDecision::Drop, 1)])
    );
    assert_eq!(
        state_counts,
        BTreeMap::from([(ForkDecisionState::Provisional, 6)])
    );

    assert!(fixture.contains("the four originally identified remainder-analysis deltas"));
    assert!(fixture.contains("baseline-equivalent failures remain failing gates"));
    assert!(fixture.contains("best-effort post-close compaction"));
    assert!(!fixture.contains("live 64 KiB bound is proven"));
}

#[test]
fn management_doctrine_contains_required_rules() {
    let management = include_str!("../modules/management/full.md");
    let manager_role = include_str!("../roles/manager/full.md");
    for required in [
        "doubt about intent, authority, safety, or privacy",
        "reflection and confirmation are not ritual gates.",
        "Direct known work goes to one specialist.",
        "Unfamiliar non-trivial work goes first to a fast, cheap",
        "Tightly coupled cross-specialty work goes to one accountable Generalist.",
        "Independent work goes to peer specialists in parallel.",
        "A Generalist may use subagents when useful",
        "Do not impose a rigid one-level delegation limit.",
        "does not inspect repositories, commands, links, or systems",
        "It never records or mutates Spirit.",
        "load only the optional skills listed in its generated role packet",
        "return or feedback protocols already present in role packets.",
        "The manager never spawns a blocking agent.",
        "Every manager-dispatched agent runs",
        "in the background. Never use a foreground agent call",
        "Never use a foreground agent call or wait synchronously for",
        "defer its dispatch until completion",
        "notification arrives while keeping psyche chat available for redirection.",
        "Dispatch workers without `turnBudget`, `toolBudget`, `timeoutMs`, or",
        "hypothetical runaway risk do not justify limits.",
        "concrete external constraint requires it,",
        "disclose that constraint before dispatch.",
        "Do not interrupt or terminate a worker for turn count or silence",
        "Inspect concrete evidence of blockage first.",
        "do not fail a read-only Scout for lacking",
        "changed-file evidence.",
        "When asked why, lead with the",
        "causal mechanism.",
        "Do not substitute apology, self-judgment, or a promise for the",
        "Treat every tool result as psyche-visible.",
        "inspect concise status first.",
        "request the smallest tail that resolves it.",
        "Do not narrate repeated availability checks.",
        "The synthesis gate binds from first dispatch until the outstanding-worker set is",
        "Follow-up dispatches, lane extensions, and resumed workers re-close the",
        "an interim return earns at most a brief factual note",
        "never a synthesis installment, a",
        "the manager does not volunteer elaboration early.",
        "Deliver the full consolidated synthesis exactly once, after the final worker",
        "returns, in ordinary English",
        "Make every psyche-facing question or decision request self-contained.",
        "in enough substance to answer from chat alone.",
        "psyche opens a report or recalls a prior session.",
        "Speak the psyche's own vocabulary, not the agents'.",
        "a name is never an explanation.",
        "let compression outrun the psyche's model:",
        "When the psyche signals lost understanding, stop advancing and re-ground before",
        "in the psyche's own terms.",
        "Name the Session and Lane in PascalCase alphanumeric;",
        "a hyphenated name forces a translation",
    ] {
        assert!(
            management.contains(required),
            "missing management rule: {required}"
        );
    }
    for required in [
        "Never spawn a blocking agent.",
        "Run every dispatched agent in the background;",
        "defer dependent dispatch until completion notification",
        "remain available for psyche redirection.",
    ] {
        assert!(
            manager_role.contains(required),
            "missing manager role rule: {required}"
        );
    }
    assert!(!management.contains("orchestrator"));
    assert!(!management.contains("orchestration"));
    for operational_detail in [
        "deploy",
        "lojix",
        "launcher",
        "profile",
        "Home Manager",
        "rollback",
    ] {
        assert!(
            !management
                .to_lowercase()
                .contains(&operational_detail.to_lowercase()),
            "management contains operational detail: {operational_detail}"
        );
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
    assert!(claude.starts_with(
        "---\nname: worker\ndescription: 'Worker role.'\nmodel: claude-test\neffort: high\n---\n\n"
    ));
    assert!(claude.contains("# worker"));
    assert!(claude.contains("## shared"));
    assert!(claude.contains("## feature"));
    assert!(!claude.contains("Role - worker"));
    assert!(!claude.contains("Module - shared"));
    assert!(claude.find("# worker") < claude.find("## shared"));
    assert!(claude.find("## shared") < claude.find("## feature"));
    assert_eq!(claude.matches("## shared").count(), 1);
    assert_eq!(claude.matches("Dependency first.").count(), 1);
    assert!(!claude.contains("@generated"));
    assert!(!claude.contains("generated by"));

    let codex = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex.contains("name = \"worker\""));
    assert!(codex.contains("description = \"Worker role.\""));
    assert!(codex.contains("model = \"gpt-test\""));
    assert!(codex.contains("model_reasoning_effort = \"high\""));
    assert!(codex.contains("developer_instructions = \"# worker"));
    assert!(codex.contains("## shared"));
    assert!(codex.contains("## feature"));
    assert!(
        codex.contains(
            "Skill-read de-duplication: A pasted <skill ...>...</skill> block is complete"
        )
    );
    assert!(!claude.contains("Skill-read de-duplication"));

    let pi = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi.starts_with("---\nname: worker\ndescription: 'Worker role.'\nmodel: 'openai-codex/gpt-test'\nthinking: high\n---\n\n"));
    assert!(!pi.contains("Skill-read de-duplication"));

    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    assert!(inventory.contains(".claude/agents/worker.md"));
    assert!(inventory.contains(".codex/agents/worker.toml"));
    assert!(inventory.contains(".pi/agents/worker.md"));
}

#[test]
fn pi_manager_dispatch_roster_equals_generated_pi_agents_without_manager_or_non_pi_roles() {
    let fixture = Fixture::new();
    let children = [
        "generalist",
        "intent-recorder",
        "intent-translator",
        "scout",
        "repo-scaffolder",
        "general-code-implementer",
        "operating-system-implementer",
        "rust-auditor",
        "nix-auditor",
        "skill-editor",
        "intent-curator",
        "repository-closeout",
        "tracker-weaver",
    ];
    let mut active_outputs =
        vec!["(Role (manager manager [] [Manager root.] [PiAgent]))".to_owned()];
    active_outputs.extend(
        children
            .iter()
            .map(|role| format!("(Role ({role} {role} [] [Role {role}.] [PiAgent]))")),
    );
    active_outputs
        .push("(Role (claude-only claude-only [] [Non Pi witness.] [ClaudeAgent]))".to_owned());
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        &format!("[{}]\n", active_outputs.join(" ")),
    );

    let mut dependencies = Vec::new();
    for role in children.iter().chain(["manager", "claude-only"].iter()) {
        dependencies.push(format!("({role} roles/{role}/full.md [] RoleSource)"));
        fixture.write_source_file(
            &format!("roles/{role}/full.md"),
            &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
        );
    }
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        &format!("[{}]\n", dependencies.join(" ")),
    );
    let metadata_roles = children
        .iter()
        .chain(["manager", "claude-only"].iter())
        .copied()
        .collect::<Vec<_>>();
    fixture.write_role_metadata(&metadata_roles);

    let report = fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");
    let manager = fixture.read_workspace_file(".pi/agents/manager.md");
    let roster = manager
        .lines()
        .filter_map(|line| {
            line.strip_prefix("- `")
                .and_then(|line| line.split('`').next())
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let generated_pi_agents = report
        .payload()
        .payload()
        .iter()
        .filter_map(|file| file.output_path.as_ref().strip_prefix(".pi/agents/"))
        .filter_map(|path| path.strip_suffix(".md"))
        .filter(|role| *role != "manager")
        .map(str::to_owned)
        .collect::<Vec<_>>();

    assert_eq!(roster, generated_pi_agents);
    assert_eq!(roster, children);
    assert_eq!(roster.len(), 13);
    assert!(!roster.contains(&"manager".to_owned()));
    assert!(!roster.contains(&"claude-only".to_owned()));
    assert!(manager.contains("runtime validation handles unknown or disabled names"));
}

#[test]
fn role_profiles_and_optional_skills_render_without_preloading_skill_bodies() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill ClaudeSkill])) (Role (worker worker [] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example modules/example/full.md [] RuntimeSkill) (worker roles/worker/full.md [] RoleSource)]\n",
    );
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill - example\n\n## Example Rule\n\nThis body must not be preloaded.\n",
    );
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\nRole body.\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [example])]\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("profiled role with optional skill generates");

    let claude = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude.contains("model: claude-test\neffort: high"));
    assert!(claude.contains("## optional skills"));
    assert!(claude.contains("- `example`"));
    assert!(!claude.contains("This body must not be preloaded."));

    let pi = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi.contains("model: 'openai-codex/gpt-test'\nthinking: high\nskills: example"));
    assert!(pi.contains("## optional skills"));
    assert!(!pi.contains("This body must not be preloaded."));

    let codex = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex.contains("model = \"gpt-test\""));
    assert!(codex.contains("model_reasoning_effort = \"high\""));
    assert!(codex.contains("## optional skills"));
    assert!(!codex.contains("This body must not be preloaded."));
}

#[test]
fn role_model_assignments_reject_missing_duplicate_stale_and_duplicate_catalog_entries() {
    let missing = Fixture::new();
    missing.write_role_generation_sources();
    missing.write_source_file("manifests/role-model-assignments.nota", "[]\n");
    let error = missing
        .generate(GenerationMode::Write)
        .expect_err("missing assignment fails");
    assert!(
        matches!(error, Error::MissingRoleModelAssignment { .. }),
        "{error:?}"
    );

    let duplicate = Fixture::new();
    duplicate.write_role_generation_sources();
    duplicate.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(worker (gpt-test High) (claude-test High)) (worker (gpt-test High) (claude-test High))]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate assignment fails");
    assert!(
        matches!(error, Error::DuplicateRoleModelAssignment { .. }),
        "{error:?}"
    );

    let stale = Fixture::new();
    stale.write_role_generation_sources();
    stale.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(worker (gpt-test High) (claude-test High)) (retired-role (gpt-test High) (claude-test High))]\n",
    );
    let error = stale
        .generate(GenerationMode::Write)
        .expect_err("stale assignment fails");
    assert!(
        matches!(error, Error::StaleRoleModelAssignment { .. }),
        "{error:?}"
    );

    let duplicate_catalog = Fixture::new();
    duplicate_catalog.write_role_generation_sources();
    duplicate_catalog.write_source_file(
        "manifests/model-catalog.nota",
        "[(ChatGpt (gpt-test openai-codex [High])) (ChatGpt (gpt-test openai-codex [High])) (Claude (claude-test [High]))]\n",
    );
    let error = duplicate_catalog
        .generate(GenerationMode::Write)
        .expect_err("duplicate catalog entry fails");
    assert!(
        matches!(error, Error::DuplicateModelCatalogEntry { .. }),
        "{error:?}"
    );
}

#[test]
fn role_model_assignments_reject_unsupported_effort_and_family_mismatch() {
    let unsupported = Fixture::new();
    unsupported.write_role_generation_sources();
    unsupported.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(worker (unknown-model High) (claude-test High))]\n",
    );
    let error = unsupported
        .generate(GenerationMode::Write)
        .expect_err("unknown model fails");
    assert!(
        matches!(error, Error::UnsupportedRoleModel { .. }),
        "{error:?}"
    );

    let effort = Fixture::new();
    effort.write_role_generation_sources();
    effort.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(worker (gpt-test Xhigh) (claude-test High))]\n",
    );
    let error = effort
        .generate(GenerationMode::Write)
        .expect_err("unsupported effort fails");
    assert!(
        matches!(error, Error::UnsupportedRoleModelEffort { .. }),
        "{error:?}"
    );

    let family = Fixture::new();
    family.write_role_generation_sources();
    family.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(worker (claude-test High) (gpt-test High))]\n",
    );
    let error = family
        .generate(GenerationMode::Write)
        .expect_err("family mismatch fails");
    assert!(
        matches!(error, Error::RoleModelFamilyMismatch { .. }),
        "{error:?}"
    );
}

#[test]
fn optional_skill_metadata_rejects_missing_duplicate_stale_and_inactive_references() {
    let missing = Fixture::new();
    missing.write_role_generation_sources();
    missing.write_source_file("manifests/role-optional-skills.nota", "[]\n");
    let error = missing
        .generate(GenerationMode::Write)
        .expect_err("missing optional metadata fails");
    assert!(
        matches!(error, Error::MissingRoleOptionalSkills { .. }),
        "{error:?}"
    );

    let duplicate = Fixture::new();
    duplicate.write_role_generation_sources();
    duplicate.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker []) (worker [])]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate optional metadata fails");
    assert!(
        matches!(error, Error::DuplicateRoleOptionalSkills { .. }),
        "{error:?}"
    );

    let stale = Fixture::new();
    stale.write_role_generation_sources();
    stale.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker []) (retired-role [])]\n",
    );
    let error = stale
        .generate(GenerationMode::Write)
        .expect_err("stale optional metadata fails");
    assert!(
        matches!(error, Error::StaleRoleOptionalSkills { .. }),
        "{error:?}"
    );

    let inactive = Fixture::new();
    inactive.write_role_generation_sources();
    inactive.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [renamed-skill])]\n",
    );
    let error = inactive
        .generate(GenerationMode::Write)
        .expect_err("inactive optional skill fails");
    assert!(
        matches!(error, Error::MissingOptionalSkill { .. }),
        "{error:?}"
    );
}

#[test]
fn optional_skill_metadata_rejects_duplicate_and_target_incompatible_skills() {
    let duplicate = Fixture::new();
    duplicate.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill ClaudeSkill])) (Role (worker worker [] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    duplicate.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example modules/example/full.md [] RuntimeSkill) (worker roles/worker/full.md [] RoleSource)]\n",
    );
    duplicate.write_role_metadata(&["worker"]);
    duplicate.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [example example])]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate optional skill fails");
    assert!(
        matches!(error, Error::DuplicateOptionalSkill { .. }),
        "{error:?}"
    );

    let incompatible = Fixture::new();
    incompatible.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [ClaudeSkill])) (Role (worker worker [] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    incompatible.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example modules/example/full.md [] RuntimeSkill) (worker roles/worker/full.md [] RoleSource)]\n",
    );
    incompatible.write_role_metadata(&["worker"]);
    incompatible.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [example])]\n",
    );
    let error = incompatible
        .generate(GenerationMode::Write)
        .expect_err("target-incompatible skill fails");
    assert!(
        matches!(error, Error::TargetIncompatibleOptionalSkill { .. }),
        "{error:?}"
    );
}

#[test]
fn universal_role_modules_expand_into_every_role_packet_without_per_role_manifest_entries() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [feature] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker/full.md [] RoleSource) (universal modules/universal/full.md [] RoleComposition) (feature modules/feature/full.md [] RoleComposition)]\n",
    );
    fixture.write_source_file("manifests/universal-role-modules.nota", "[universal]\n");
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\nRole body.\n",
    );
    fixture.write_source_file(
        "modules/universal/full.md",
        "# Module - universal\n\n## Universal Rule\n\nUniversal doctrine.\n",
    );
    fixture.write_source_file(
        "modules/feature/full.md",
        "# Module - feature\n\n## Feature Rule\n\nPer-role doctrine.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("universal role modules generate");

    let claude = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude.contains("Universal doctrine."));
    assert!(claude.contains("Per-role doctrine."));
    assert!(claude.find("Role body.") < claude.find("Universal doctrine."));
    assert!(claude.find("Universal doctrine.") < claude.find("Per-role doctrine."));
    assert_eq!(claude.matches("Universal doctrine.").count(), 1);
    assert_eq!(
        fixture
            .read_workspace_file(".pi/agents/worker.md")
            .matches("Universal doctrine.")
            .count(),
        1
    );
    assert_eq!(
        fixture
            .read_workspace_file(".codex/agents/worker.toml")
            .matches("Universal doctrine.")
            .count(),
        1
    );
}

#[test]
fn generation_strips_source_maintenance_notes_from_runtime_surfaces() {
    let fixture = Fixture::new();
    fixture.write_default_roster();
    fixture.write_source_file(
        "modules/example/full.md",
        "# Skill - example\n\n## Rule\n\nGenerated.\n\n## Source Maintenance Notes\n\nMaintainer-only synchronization steps.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("source maintenance notes stay source-only");

    let agents_skill = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert!(agents_skill.contains("# example"));
    assert!(agents_skill.contains("Generated."));
    assert!(!agents_skill.contains("Skill - example"));
    assert!(!agents_skill.contains("Source Maintenance Notes"));
    assert!(!agents_skill.contains("Maintainer-only synchronization steps"));
}

#[test]
fn target_module_insertions_apply_only_to_matching_generated_surfaces() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (management management Meta Mechanism [Management skill] [AgentsSkill ClaudeSkill])) (Role (worker worker [management] [Worker role] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker/full.md [] RoleSource) (management modules/management/full.md [] RuntimeSkill) (claude-management modules/claude-management/full.md [] RuntimeSkill)]\n",
    );
    fixture.write_source_file(
        "manifests/target-module-insertions.nota",
        "[(management ClaudeSkill [claude-management]) (management ClaudeAgent [claude-management])]\n",
    );
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\nRole body.\n",
    );
    fixture.write_source_file(
        "modules/management/full.md",
        "# Skill - management\n\n## Shared Rule\n\nShared management.\n",
    );
    fixture.write_source_file(
        "modules/claude-management/full.md",
        "# Module - Target reply surface\n\n## Clarification UI\n\nTarget overlay.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("target insertions generate");

    let agents_skill = fixture.read_workspace_file(".agents/skills/management/SKILL.md");
    assert!(agents_skill.contains("Shared management."));
    assert!(!agents_skill.contains("Target overlay."));

    let claude_skill = fixture.read_workspace_file(".claude/skills/management/SKILL.md");
    assert!(claude_skill.contains("Shared management."));
    assert!(claude_skill.contains("Target overlay."));

    let claude_role = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude_role.contains("Shared management."));
    assert!(claude_role.contains("Target overlay."));

    let codex_role = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex_role.contains("Shared management."));
    assert!(!codex_role.contains("Target overlay."));

    let pi_role = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi_role.contains("Shared management."));
    assert!(!pi_role.contains("Target overlay."));
}

#[test]
fn role_generation_rejects_retired_current_destination_prose() {
    for phrase in [
        "Repo Operator",
        "Weave Operator",
        "Intent Maintainer",
        "workspace essence",
        "workspace intent",
    ] {
        let fixture = Fixture::new();
        fixture.write_role_generation_sources();
        fixture.write_source_file(
            "roles/worker/full.md",
            &format!(
                "# Role - worker\n\n## Contract\n\nDo not assign current closeout to {phrase}.\n"
            ),
        );
        fixture.write_source_file(
            "modules/shared/full.md",
            "# Module - shared\n\n## Shared Rule\n\nDependency first.\n",
        );
        fixture.write_source_file(
            "modules/feature/full.md",
            "# Module - feature\n\n## Feature Rule\n\nDependent second.\n",
        );

        let error = fixture
            .generate(GenerationMode::Write)
            .expect_err("retired title-case current-destination prose fails role generation");

        assert!(
            matches!(
                error,
                Error::RetiredCurrentDestinationProse { phrase: ref found, .. } if found == phrase
            ),
            "{error:?}"
        );
    }
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
    fixture.write_role_metadata(&["worker"]);
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
    fixture.write_role_metadata(&["worker"]);
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
fn generation_rejects_role_required_module_missing_from_dependency_index() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [spirit-query] [Worker role.] [ClaudeAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker/full.md [] RoleSource)]\n",
    );
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\nBody.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("role-required modules must resolve before packet generation");

    assert!(
        matches!(
            error,
            Error::MissingModule {
                ref module_identifier,
            } if module_identifier == "spirit-query"
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

#[test]
fn write_mode_prunes_removed_or_renamed_skill_and_role_outputs() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (new-skill new-skill Craft Topic [New skill.] [AgentsSkill ClaudeSkill])) (Role (new-worker new-worker [] [New worker.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_role_metadata(&["new-worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(new-skill modules/new-skill/full.md [] RuntimeSkill) (new-worker roles/new-worker/full.md [] RoleSource)]\n",
    );
    fixture.write_source_file(
        "modules/new-skill/full.md",
        "# Skill — new-skill\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_source_file(
        "roles/new-worker/full.md",
        "# Role - new-worker\n\n## Contract\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/old-skill/SKILL.md", "stale skill\n");
    fixture.write_workspace_file(".claude/skills/old-skill/SKILL.md", "stale skill\n");
    fixture.write_workspace_file(
        "skills/generated-role-outputs.nota",
        "[.claude/agents/old-worker.md .codex/agents/old-worker.toml .pi/agents/old-worker.md]\n",
    );
    fixture.write_workspace_file(".claude/agents/old-worker.md", "stale role\n");
    fixture.write_workspace_file(".codex/agents/old-worker.toml", "stale role\n");
    fixture.write_workspace_file(".pi/agents/old-worker.md", "stale role\n");
    fixture.write_workspace_file(".claude/agents/human-owned.md", "human-owned role\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("write mode prunes removed or renamed generated outputs");

    for stale_path in [
        ".agents/skills/old-skill/SKILL.md",
        ".claude/skills/old-skill/SKILL.md",
        ".claude/agents/old-worker.md",
        ".codex/agents/old-worker.toml",
        ".pi/agents/old-worker.md",
    ] {
        assert!(
            !fixture.workspace.path().join(stale_path).exists(),
            "{stale_path} is pruned"
        );
    }
    for active_path in [
        ".agents/skills/new-skill/SKILL.md",
        ".claude/skills/new-skill/SKILL.md",
        ".claude/agents/new-worker.md",
        ".codex/agents/new-worker.toml",
        ".pi/agents/new-worker.md",
        ".claude/agents/human-owned.md",
    ] {
        assert!(
            fixture.workspace.path().join(active_path).exists(),
            "{active_path} remains or is generated"
        );
    }
    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    assert!(!inventory.contains("old-worker"));
    assert!(inventory.contains("new-worker"));
}

#[test]
fn trunk_guard_passes_source_without_jujutsu_working_copy() {
    let source = TempDir::new().expect("source tempdir");

    TrunkDescendantGuard::new(source.path())
        .verify()
        .expect("an immutable source with no Jujutsu working copy is inherently safe");
}

#[test]
fn trunk_divergence_permits_regeneration_when_no_trunk_commits_are_unreached() {
    let divergence = TrunkDivergence::from_revset_output("\n  \n");

    assert!(
        !divergence.requires_refusal(),
        "a descendant working copy leaves no trunk commit unreached"
    );
}

#[test]
fn trunk_divergence_refuses_regeneration_when_trunk_has_unreached_commits() {
    let divergence = TrunkDivergence::from_revset_output("oxxluyzymxmv\nrlkyomtvabcd\n");

    assert!(
        divergence.requires_refusal(),
        "a sibling or behind working copy leaves trunk commits unreached and must refuse"
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
        self.write_role_metadata(&["worker"]);
    }

    fn write_role_metadata(&self, role_identifiers: &[&str]) {
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-test openai-codex [Medium High])) (Claude (claude-test [Medium High]))]\n",
        );
        let assignments = role_identifiers
            .iter()
            .map(|role| format!("({role} (gpt-test High) (claude-test High))"))
            .collect::<Vec<_>>()
            .join(" ");
        self.write_source_file(
            "manifests/role-model-assignments.nota",
            &format!("[{assignments}]\n"),
        );
        let optional_skills = role_identifiers
            .iter()
            .map(|role| format!("({role} [])"))
            .collect::<Vec<_>>()
            .join(" ");
        self.write_source_file(
            "manifests/role-optional-skills.nota",
            &format!("[{optional_skills}]\n"),
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
