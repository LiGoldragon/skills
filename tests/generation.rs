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
        ActiveOutputs, EffortLevel, EmissionPolicy, GenerationMode, GenerationRequest,
        ManifestPath, ModelCatalog, ModuleDependencies, ModuleKind, ModuleLifecycle,
        NestedRoleRelations, RoleModelAssignments, RoleOptionalSkills, RoleTargetSurface,
        SkillRoster, SourceRoot, TargetModuleInsertions, TargetSurface, UniversalRoleModules,
        WorkspaceRoot,
    },
    trunk_guard::{TrunkDescendantGuard, TrunkDivergence},
};
use tempfile::TempDir;

#[derive(Debug, Eq, PartialEq)]
struct ParsedProjectRoleContract {
    project_role_identity: String,
    project_role_dispatch_kind: String,
    allowed_child_role_names: Vec<String>,
}

fn flat_frontmatter(packet: &str) -> BTreeMap<String, String> {
    let block = packet
        .strip_prefix("---\n")
        .and_then(|packet| packet.split_once("\n---\n"))
        .map(|(frontmatter, _)| frontmatter)
        .expect("packet has frontmatter");
    block
        .lines()
        .map(|line| {
            let (key, value) = line.split_once(':').expect("flat frontmatter field");
            let value = value.trim();
            let value = value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .or_else(|| {
                    value
                        .strip_prefix('"')
                        .and_then(|value| value.strip_suffix('"'))
                })
                .unwrap_or(value);
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn project_role_contract(packet: &str, runtime_role_name: &str) -> ParsedProjectRoleContract {
    let frontmatter = flat_frontmatter(packet);
    let identity = frontmatter
        .get("projectRoleIdentity")
        .expect("projectRoleIdentity exists");
    assert_eq!(identity, runtime_role_name);
    let dispatch_kind = frontmatter
        .get("projectRoleDispatchKind")
        .expect("projectRoleDispatchKind exists");
    assert!(matches!(
        dispatch_kind.as_str(),
        "manager" | "nested" | "leaf"
    ));
    let allowed_child_role_names = frontmatter
        .get("allowedChildRoleNames")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if dispatch_kind == "nested" {
        assert!(frontmatter.contains_key("allowedChildRoleNames"));
    } else {
        assert!(!frontmatter.contains_key("allowedChildRoleNames"));
    }
    for incompatible_key in [
        "delegation-role-classification",
        "allowed-child-role-identifiers",
    ] {
        assert!(!frontmatter.contains_key(incompatible_key));
    }
    ParsedProjectRoleContract {
        project_role_identity: identity.clone(),
        project_role_dispatch_kind: dispatch_kind.clone(),
        allowed_child_role_names,
    }
}

fn frontmatter_block(packet: &str) -> &str {
    let end = packet.find("\n---\n").expect("frontmatter closes") + "\n---\n".len();
    &packet[..end]
}

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
    let nested_role_relations =
        NotaSource::new(include_str!("../manifests/nested-role-relations.nota"))
            .parse::<NestedRoleRelations>()
            .expect("nested role relations parse");

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

    assert_eq!(skill_count, 65);
    assert_eq!(role_count, 14);
    assert_eq!(model_catalog.payload().len(), 6);
    assert_eq!(nested_role_relations.payload().len(), 3);
    assert_eq!(role_model_assignments.payload().len(), role_count);
    assert_eq!(role_optional_skills.payload().len(), role_count);

    let model_catalog_source = include_str!("../manifests/model-catalog.nota");
    let role_model_assignments_source = include_str!("../manifests/role-model-assignments.nota");
    assert!(model_catalog_source.contains("(Claude (claude-sonnet-5 [(Medium 10)]))"));
    assert!(
        model_catalog_source
            .contains("(ChatGpt (gpt-5.6-sol openai-codex [(Medium 50) (High 60)]))")
    );
    assert!(
        model_catalog_source
            .contains("(ChatGpt (gpt-5.6-terra openai-codex [(Medium 20) (High 30) (Xhigh 40)]))")
    );
    assert!(model_catalog_source.contains("(Claude (fable-5 [(Medium 50) (High 60)]))"));
    assert!(model_catalog_source.contains("(Claude (claude-opus-4-8 [(High 30) (Xhigh 40)]))"));
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
        "general-instructions",
        "psyche-facing-commitments",
        "codex-skill-loading",
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
        "spirit-submission",
        "manager-boundary",
        "manager-intent-classification",
        "manager-safeguards",
        "manager-dispatch",
        "manager-liveness",
        "manager-decisions",
        "manager-communication",
        "manager-synthesis",
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
    assert_eq!(
        management_dependency
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["spirit-query", "nota-design"]
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
                "skill-editor",
                skills::schema::assembly::OutputSurface::AgentsSkill,
                vec!["harness-placement"]
            ),
            (
                "skill-editor",
                skills::schema::assembly::OutputSurface::ClaudeSkill,
                vec!["harness-placement"]
            ),
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
            (
                "general-instructions",
                skills::schema::assembly::OutputSurface::CodexAgent,
                vec!["codex-skill-loading"]
            ),
        ]
    );
    assert_eq!(
        universal_role_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["general-instructions"]
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
        (
            "manager",
            "role-manager",
            &[
                "management",
                "manager-boundary",
                "manager-intent-classification",
                "manager-safeguards",
                "manager-dispatch",
                "manager-liveness",
                "manager-decisions",
                "manager-communication",
                "manager-synthesis",
                "psyche-facing-commitments",
                "protos-syntax",
            ],
        ),
        (
            "generalist",
            "role-generalist",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
                "non-ideal-registry",
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
                "non-ideal-registry",
            ],
        ),
        (
            "general-code-implementer",
            "role-general-code-implementer",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
                "non-ideal-registry",
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
                "non-ideal-registry",
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
                "non-ideal-registry",
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
                "non-ideal-registry",
            ],
        ),
        (
            "skill-editor",
            "role-skill-editor",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "skill-source-core",
                "harness-placement",
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
        let expected_surfaces: &[RoleTargetSurface] = &[
            RoleTargetSurface::ClaudeAgent,
            RoleTargetSurface::CodexAgent,
            RoleTargetSurface::PiAgent,
        ];
        assert_eq!(role.role_target_surfaces.payload(), expected_surfaces);
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
    assert_eq!(
        management
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["spirit-query", "nota-design"]
    );
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
fn repository_visibility_doctrine_defaults_public_without_weakening_privacy() {
    let publication = include_str!("../modules/repository-publication/full.md");
    for required in [
        "Repositories are public by default.",
        "Do not ask or repeatedly seek visibility permission absent such a conflict.",
        "Public-by-default visibility never authorizes publishing private information, secrets, credentials, or unreviewed private material.",
        "Before creation, inspect configured remotes and query the canonical owner/name on the forge.",
        "Create a repository only when no remote repository already exists.",
    ] {
        assert!(
            publication.contains(required),
            "missing repository-publication rule: {required}"
        );
    }

    let management = include_str!("../modules/repository-management/full.md");
    for required in [
        "treat public visibility as the default",
        "use `repository-publication` for remote discovery, creation, and privacy gates",
        "Ask about visibility only when a concrete privacy or safety conflict applies.",
        "ask about the project boundary before creation.",
    ] {
        assert!(
            management.contains(required),
            "missing repository-management rule: {required}"
        );
    }

    let dependencies = NotaSource::new(include_str!("../manifests/module-dependencies.nota"))
        .parse::<ModuleDependencies>()
        .expect("module dependency index parses");
    for role_module in ["repo-scaffold-core", "repo-operation-core"] {
        let dependency = dependencies
            .payload()
            .iter()
            .find(|dependency| dependency.module_identifier.as_ref() == role_module)
            .unwrap_or_else(|| panic!("{role_module} dependency indexed"));
        assert_eq!(
            dependency
                .dependency_modules
                .payload()
                .iter()
                .map(|module| module.as_ref())
                .collect::<Vec<_>>(),
            ["repository-publication"],
            "{role_module} carries repository visibility doctrine into role packets"
        );
    }
}

#[test]
fn skill_editor_doctrine_names_canonical_source_and_generated_targets() {
    let skill_module = include_str!("../modules/skill-editor/full.md");
    let role_module = include_str!("../roles/skill-editor/full.md");
    let skill_source_core = include_str!("../modules/skill-source-core/full.md");
    let harness_placement = include_str!("../modules/harness-placement/full.md");
    let skills_repo = include_str!("../skills.md");

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
    for required in [
        "Classify each instruction before placing it.",
        "General doctrine describes behavior\nindependent of a runtime API.",
        "target-module-insertions.nota",
        "When no matching target-specific surface exists, omit the rule",
    ] {
        assert!(
            harness_placement.contains(required),
            "harness placement teaches {required}"
        );
    }
    for required in [
        "## Harness placement",
        "manifests/active-outputs.nota",
        "manifests/module-dependencies.nota",
        "manifests/target-module-insertions.nota",
        "AgentsSkill",
        "ClaudeSkill",
        "ClaudeAgent",
        "CodexAgent",
        "PiAgent",
        "It has no active `PiAgent` insertion.",
        "The generator expands dependencies, then appends matching target modules",
        "When the required\nsurface does not exist, omit the rule",
    ] {
        assert!(
            skills_repo.contains(required),
            "skills repository harness-placement documentation includes {required}"
        );
    }
}

#[test]
fn harness_api_fields_do_not_leak_into_general_management_doctrine() {
    let fields = ["turnBudget", "toolBudget", "timeoutMs", "maxRuntimeMs"];
    for (name, source) in [
        ("management", include_str!("../modules/management/full.md")),
        (
            "manager-boundary",
            include_str!("../modules/manager-boundary/full.md"),
        ),
        (
            "manager-intent-classification",
            include_str!("../modules/manager-intent-classification/full.md"),
        ),
        (
            "manager-safeguards",
            include_str!("../modules/manager-safeguards/full.md"),
        ),
        (
            "manager-dispatch",
            include_str!("../modules/manager-dispatch/full.md"),
        ),
        (
            "manager-liveness",
            include_str!("../modules/manager-liveness/full.md"),
        ),
        (
            "manager-decisions",
            include_str!("../modules/manager-decisions/full.md"),
        ),
        (
            "manager-communication",
            include_str!("../modules/manager-communication/full.md"),
        ),
        (
            "manager-synthesis",
            include_str!("../modules/manager-synthesis/full.md"),
        ),
    ] {
        for field in fields {
            assert!(
                !source.contains(field),
                "general {name} doctrine leaks harness API field {field}"
            );
        }
    }

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("harness-placement profile generates");
    for path in [
        ".agents/skills/management/SKILL.md",
        ".claude/skills/management/SKILL.md",
        ".pi/agents/manager.md",
        ".claude/agents/manager.md",
        ".codex/agents/manager.toml",
    ] {
        let output = fixture.read_workspace_file(path).replace("\\n", "\n");
        for field in fields {
            assert!(
                !output.contains(field),
                "general generated output {path} leaks harness API field {field}"
            );
        }
    }
    for path in [
        ".agents/skills/skill-editor/SKILL.md",
        ".claude/skills/skill-editor/SKILL.md",
        ".pi/agents/skill-editor.md",
        ".claude/agents/skill-editor.md",
        ".codex/agents/skill-editor.toml",
    ] {
        assert!(
            fixture
                .read_workspace_file(path)
                .contains("Classify each instruction before placing it."),
            "{path} trains the placement test"
        );
    }
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
fn management_doctrine_preserves_base_and_composes_manager_safeguards() {
    let management = include_str!("../modules/management/full.md");
    let claude_management = include_str!("../modules/claude-management/full.md");
    let manager_role = include_str!("../roles/manager/full.md");
    let manager_modules = [
        (
            "boundary",
            include_str!("../modules/manager-boundary/full.md"),
            "Outside this action space, every investigation and operation goes to a subagent.",
        ),
        (
            "intent classification",
            include_str!("../modules/manager-intent-classification/full.md"),
            "Matter does not become intent because it is broad, durable, emphatic, or directly",
        ),
        (
            "safeguards",
            include_str!("../modules/manager-safeguards/full.md"),
            "A host reboot is forbidden by default.",
        ),
        (
            "dispatch",
            include_str!("../modules/manager-dispatch/full.md"),
            "Tightly coupled cross-specialty work goes to one accountable Generalist.",
        ),
        (
            "liveness",
            include_str!("../modules/manager-liveness/full.md"),
            "Report a worker as running only on fresh positive evidence",
        ),
        (
            "decisions",
            include_str!("../modules/manager-decisions/full.md"),
            "Psyche responses carry graded states, not one yes or no:",
        ),
        (
            "communication",
            include_str!("../modules/manager-communication/full.md"),
            "Make every psyche-facing question or decision request self-contained.",
        ),
        (
            "synthesis",
            include_str!("../modules/manager-synthesis/full.md"),
            "The synthesis gate binds from first dispatch until the outstanding-worker set is",
        ),
    ];

    for heading in [
        "Rules",
        "Psyche Boundary",
        "Inputs",
        "Action Space",
        "Curiosity",
        "Gates",
        "Planning And Dispatch",
        "Synthesis",
    ] {
        assert!(
            management.contains(&format!("## {heading}")),
            "historical base preserves {heading}"
        );
    }
    for required in [
        "The manager is an intent-only lane.",
        "Treat problem reports and frustration as context, not dispatch authority.",
        "Use a read-only Spirit query only when existing intent would resolve a material ambiguity",
        "Use CamelCase Session names and task-specific Lane names.",
        "Never block it on background work.",
        "Use a separate auditor only for substantial or consequence-gated completed work",
    ] {
        assert!(
            management.contains(required),
            "historical rule preserved: {required}"
        );
    }
    assert!(claude_management.contains("# Module — Management reply surface"));
    assert!(claude_management.contains("## Clarification UI"));
    assert!(manager_role.contains("Apply the preloaded management modules together."));
    assert!(
        include_str!("../modules/manager-dispatch/full.md").contains(
            "the historical CamelCase wording means the\ndaemon-compatible PascalCase alphanumeric form"
        ),
        "manager dispatch clarifies the preserved historical session/lane wording"
    );
    for (name, source, required) in manager_modules {
        assert!(
            source.contains(required),
            "{name} retains its manager safeguard"
        );
    }
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("modules/management/full.md")
            .exists(),
        "management source remains active"
    );
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("modules/claude-management/full.md")
            .exists(),
        "management companion source remains active"
    );
    for retired_source in [
        "modules/orchestration/full.md",
        "modules/claude-orchestration/full.md",
    ] {
        assert!(
            !Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(retired_source)
                .exists(),
            "former identity source is absent: {retired_source}"
        );
    }

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("restored management profile generates");
    for path in [
        ".pi/agents/manager.md",
        ".claude/agents/manager.md",
        ".codex/agents/manager.toml",
    ] {
        let packet = fixture.read_workspace_file(path).replace("\\n", "\n");
        for required in [
            "The manager is an intent-only lane.",
            "Use `spirit` for read-only intent queries before an intent-grounded judgment",
            "A host reboot is forbidden by default.",
            "Report a worker as running only on fresh positive evidence",
            "the historical CamelCase wording means the\ndaemon-compatible PascalCase alphanumeric form",
            "Psyche responses carry graded states, not one yes or no:",
            "field name is\nillegal Protos, full stop",
            "The synthesis gate binds from first dispatch until the outstanding-worker set is",
        ] {
            assert!(packet.contains(required), "{path} retains {required}");
        }
        assert!(
            !packet.contains("@generated"),
            "{path} has no generated notice"
        );
        assert!(
            !packet.to_lowercase().contains("generated by"),
            "{path} has no generated notice"
        );
    }
    assert!(
        fixture
            .read_workspace_file(".claude/agents/manager.md")
            .contains(
                "Ask clarification in ordinary chat text instead of multiple-choice, picker, or"
            )
    );
    let management_skill = fixture.read_workspace_file(".agents/skills/management/SKILL.md");
    assert!(management_skill.contains("name: management"));
    assert!(management_skill.contains("# management"));
    assert!(management_skill.contains("The manager is an intent-only lane."));
    for retired_output in [
        ".agents/skills/orchestration",
        ".claude/skills/orchestration",
    ] {
        assert!(
            !fixture.workspace.path().join(retired_output).exists(),
            "former identity output is pruned: {retired_output}"
        );
    }
}

#[test]
fn generated_manager_and_recorder_packets_preserve_matter_not_intent_classification() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current manager and recorder packets generate");

    let matter_not_intent_clauses = [
        "Matter does not become intent because it is broad, durable, emphatic, or directly\nspoken by the psyche.",
        "Requested rules, defaults, prohibitions, authorization\nboundaries, mechanisms, architecture, and guidance edits remain matter; “we need\nto forbid X” routes to operational guidance.",
        "Only explicitly expressed orienting\naims, values, or beliefs qualify, never one inferred from a mechanism.",
    ];
    for role in ["manager", "intent-recorder"] {
        for path in [
            format!(".pi/agents/{role}.md"),
            format!(".claude/agents/{role}.md"),
            format!(".codex/agents/{role}.toml"),
        ] {
            let packet = fixture.read_workspace_file(&path).replace("\\n", "\n");
            for clause in matter_not_intent_clauses {
                assert!(
                    packet.contains(clause),
                    "{path} preserves the matter-not-intent classification clause: {clause}"
                );
            }
        }
    }
}

#[test]
fn host_reboot_requires_specific_psyche_approval() {
    let safeguards = include_str!("../modules/manager-safeguards/full.md");
    let operations = include_str!("../modules/operating-system-operations/full.md");

    for (source_name, source) in [
        ("manager-safeguards", safeguards),
        ("operating-system-operations", operations),
    ] {
        let normalized_source = source.replace('\n', " ");
        for required in [
            "A host reboot is forbidden by default.",
            "explicit, contemporaneous psyche approval specifically for reboot",
            "reboot terminates local processes and agent sessions",
            "narrower recovery options already attempted or remaining",
            "generic repair request, including an instruction to fix it, does not authorize reboot.",
        ] {
            assert!(
                normalized_source.contains(required),
                "missing {source_name} reboot safeguard: {required}"
            );
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
    assert!(!claude.contains("Skill-read de-duplication"));

    let pi = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi.starts_with("---\nname: worker\ndescription: 'Worker role.'\nmodel: 'openai-codex/gpt-test'\nthinking: high\nprojectRoleIdentity: worker\nprojectRoleDispatchKind: leaf\n---\n\n"));
    assert!(!pi.contains("Skill-read de-duplication"));

    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    assert!(inventory.contains(".claude/agents/worker.md"));
    assert!(inventory.contains(".codex/agents/worker.toml"));
    assert!(inventory.contains(".pi/agents/worker.md"));
}

#[test]
fn generation_rejects_configured_execution_limit_fields_in_agent_packets() {
    let fixture = Fixture::new();
    fixture.write_role_generation_sources();
    fixture.write_source_file(
        "roles/worker/full.md",
        "# Role - worker\n\n## Contract\n\ntimeoutMs: 1\n",
    );
    fixture.write_source_file(
        "modules/shared/full.md",
        "# Module - shared\n\n## Shared Rule\n\nShared rule.\n",
    );
    fixture.write_source_file(
        "modules/feature/full.md",
        "# Module - feature\n\n## Feature Rule\n\nFeature rule.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("execution-limit field rejects agent packet generation");

    assert!(matches!(
        error,
        Error::GeneratedAgentExecutionLimit { field_name, .. } if field_name == "timeoutMs"
    ));
}

#[test]
fn manager_rosters_are_target_relative_and_never_instruct_role_listing() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (manager manager [] [Manager root.] [ClaudeAgent CodexAgent PiAgent])) (Role (shared shared [] [Shared role.] [ClaudeAgent CodexAgent PiAgent])) (Role (pi-only pi-only [] [Pi role.] [PiAgent])) (Role (claude-only claude-only [] [Claude role.] [ClaudeAgent])) (Role (codex-only codex-only [] [Codex role.] [CodexAgent]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(manager roles/manager/full.md [] RoleSource) (shared roles/shared/full.md [] RoleSource) (pi-only roles/pi-only/full.md [] RoleSource) (claude-only roles/claude-only/full.md [] RoleSource) (codex-only roles/codex-only/full.md [] RoleSource)]\n",
    );
    for role in ["manager", "shared", "pi-only", "claude-only", "codex-only"] {
        fixture.write_source_file(
            &format!("roles/{role}/full.md"),
            &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
        );
    }
    fixture.write_role_metadata(&["manager", "shared", "pi-only", "claude-only", "codex-only"]);

    fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");

    let roster = |packet: &str| {
        packet
            .replace("\\n", "\n")
            .lines()
            .filter_map(|line| line.strip_prefix("- `")?.split('`').next())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let pi = fixture.read_workspace_file(".pi/agents/manager.md");
    let claude = fixture.read_workspace_file(".claude/agents/manager.md");
    let codex = fixture.read_workspace_file(".codex/agents/manager.toml");

    assert_eq!(roster(&pi), ["shared", "pi-only"]);
    assert_eq!(roster(&claude), ["shared", "claude-only"]);
    assert_eq!(roster(&codex), ["shared", "codex-only"]);
    assert!(pi.contains("projectRoleIdentity: manager"));
    assert!(pi.contains("projectRoleDispatchKind: manager"));
    assert!(!pi.contains("allowedChildRoleNames:"));
    for packet in [&pi, &claude, &codex] {
        assert!(packet.contains("Manager dispatch roster"));
        assert!(!packet.contains("`list`"));
        assert!(!packet.contains("Orchestrator"));
        assert!(!roster(packet).contains(&"manager".to_owned()));
    }
}

#[test]
fn pi_project_role_frontmatter_matches_extension_parser_contract_fixture() {
    let fixture = Fixture::new();
    fixture.write_project_role_contract_sources();
    fixture
        .generate(GenerationMode::Write)
        .expect("project-role contract fixture generates");

    let generated = fixture.read_workspace_file(".pi/agents/planner.md");
    let contract_fixture = include_str!("fixtures/pi-project-role-frontmatter-contract.md");
    assert_eq!(
        frontmatter_block(&generated),
        frontmatter_block(contract_fixture)
    );
    assert_eq!(
        project_role_contract(&generated, "planner"),
        ParsedProjectRoleContract {
            project_role_identity: "planner".to_owned(),
            project_role_dispatch_kind: "nested".to_owned(),
            allowed_child_role_names: vec!["reader".to_owned(), "writer".to_owned()],
        }
    );
    for leaf in ["reader", "writer"] {
        let packet = fixture.read_workspace_file(&format!(".pi/agents/{leaf}.md"));
        assert_eq!(
            project_role_contract(&packet, leaf),
            ParsedProjectRoleContract {
                project_role_identity: leaf.to_owned(),
                project_role_dispatch_kind: "leaf".to_owned(),
                allowed_child_role_names: Vec::new(),
            }
        );
    }
}

#[test]
fn nested_role_schema_preserves_child_rosters_without_model_upgrades() {
    let relations = NotaSource::new(include_str!("../manifests/nested-role-relations.nota"))
        .parse::<NestedRoleRelations>()
        .expect("nested role relations parse");
    let observed: BTreeMap<_, _> = relations
        .payload()
        .iter()
        .map(|relation| {
            (
                relation.output_identifier.as_ref(),
                relation
                    .allowed_leaf_roles
                    .payload()
                    .iter()
                    .map(|role| role.as_ref())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        observed,
        BTreeMap::from([
            (
                "generalist",
                vec![
                    "scout",
                    "repo-scaffolder",
                    "general-code-implementer",
                    "rust-auditor",
                    "nix-auditor",
                    "repository-closeout",
                    "tracker-weaver",
                ],
            ),
            (
                "operating-system-implementer",
                vec![
                    "scout",
                    "general-code-implementer",
                    "rust-auditor",
                    "nix-auditor",
                    "repository-closeout",
                ],
            ),
            (
                "skill-editor",
                vec![
                    "scout",
                    "general-code-implementer",
                    "rust-auditor",
                    "repository-closeout",
                ],
            ),
        ])
    );
    for relation in relations.payload() {
        for minimum in relation.nested_role_minimum_models.payload() {
            assert_eq!(minimum.effort_level, EffortLevel::Medium);
            match minimum.role_target_surface {
                RoleTargetSurface::ClaudeAgent => {
                    assert_eq!(minimum.model_identifier.as_ref(), "claude-sonnet-5")
                }
                RoleTargetSurface::CodexAgent | RoleTargetSurface::PiAgent => {
                    assert_eq!(minimum.model_identifier.as_ref(), "gpt-5.6-luna")
                }
            }
        }
    }
    let active_outputs = NotaSource::new(include_str!("../manifests/active-outputs.nota"))
        .parse::<ActiveOutputs>()
        .expect("active outputs parse");
    assert!(active_outputs.payload().iter().all(|output| {
        match output {
            skills::schema::assembly::ActiveOutput::Role(role) => !role
                .output_identifier
                .as_ref()
                .starts_with("crucial-greenfield-"),
            skills::schema::assembly::ActiveOutput::Skill(_) => true,
        }
    }));
}
#[test]
fn generated_packets_keep_rosters_and_exclude_disallowed_worker_models() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current manifests generate");

    let roster = |path: &str| {
        let packet = fixture.read_workspace_file(path).replace("\\n", "\n");
        let roster_body = packet
            .split("## Allowed child-role roster")
            .nth(1)
            .or_else(|| packet.split("## Manager dispatch roster").nth(1))
            .expect("generated roster heading exists");
        roster_body
            .split("## optional skills")
            .next()
            .expect("role roster has content")
            .lines()
            .filter_map(|line| line.strip_prefix("- `")?.split('`').next())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        roster(".pi/agents/generalist.md"),
        [
            "scout",
            "repo-scaffolder",
            "general-code-implementer",
            "rust-auditor",
            "nix-auditor",
            "repository-closeout",
            "tracker-weaver",
        ]
    );
    assert_eq!(
        roster(".pi/agents/operating-system-implementer.md"),
        [
            "scout",
            "general-code-implementer",
            "rust-auditor",
            "nix-auditor",
            "repository-closeout",
        ]
    );
    assert_eq!(
        roster(".pi/agents/skill-editor.md"),
        [
            "scout",
            "general-code-implementer",
            "rust-auditor",
            "repository-closeout",
        ]
    );
    for path in [
        ".pi/agents/manager.md",
        ".codex/agents/manager.toml",
        ".claude/agents/manager.md",
    ] {
        assert!(
            !roster(path)
                .iter()
                .any(|role| role.starts_with("crucial-greenfield-")),
            "deactivated greenfield roles are absent from {path}"
        );
    }

    for role in [
        "generalist",
        "intent-translator",
        "operating-system-implementer",
        "skill-editor",
        "intent-curator",
    ] {
        assert!(
            fixture
                .read_workspace_file(&format!(".pi/agents/{role}.md"))
                .contains("model: 'openai-codex/gpt-5.6-terra'\nthinking: xhigh"),
            "{role} has the Terra xhigh Pi assignment"
        );
    }
    let active_roles = [
        "manager",
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
    for role in active_roles {
        let pi = fixture.read_workspace_file(&format!(".pi/agents/{role}.md"));
        let codex = fixture.read_workspace_file(&format!(".codex/agents/{role}.toml"));
        let claude = fixture.read_workspace_file(&format!(".claude/agents/{role}.md"));
        if role == "manager" {
            assert!(pi.contains("model: 'openai-codex/gpt-5.6-sol'\nthinking: high"));
            assert!(codex.contains("model = \"gpt-5.6-sol\""));
        } else {
            assert!(!pi.contains("gpt-5.6-sol"), "{role} has no Pi Sol model");
            assert!(
                !codex.contains("model = \"gpt-5.6-sol\""),
                "{role} has no Codex Sol model"
            );
        }
        assert!(
            !claude.contains("model: fable-5"),
            "{role} has no Claude Fable model"
        );
    }
}

#[test]
fn general_instructions_compose_once_and_specialized_guidance_stays_owned() {
    let general_instructions = include_str!("../modules/general-instructions/full.md");
    let universal_manifest = include_str!("../manifests/universal-role-modules.nota");
    let dependency_index = include_str!("../manifests/module-dependencies.nota");
    let language = "Avoid inventing your own expressions, labels, shorthand, metaphors, or jargon.\nFavor correct, established, plain-language descriptions even when they are\nlonger. Explain necessary established domain terminology plainly rather than\nreplacing it with agent-coined vocabulary.";
    let feedback = "Report only instruction, tooling, or documentation friction that affected or\nplausibly affects efficiency or correctness.";
    let ambiguity_return = "When unresolved ambiguity concerns intent, authority, safety, or privacy, stop\nonly the affected branch and return it to the Manager.";
    let design_authority = "Agents may investigate and propose major design changes and decide narrow\nimplementation details inside an explicitly accepted design.";
    let execution_limits = "No agent may ever introduce, recommend, configure, generate, or deploy any\nlimit on agent execution anywhere: turn, tool, token, time, cost, deadline,\nbudget, or equivalent. Treat every existing configured execution limit as a\ndefect to surface and remove, never as precedent.";

    assert!(universal_manifest.contains("[general-instructions]"));
    assert!(dependency_index.contains(
        "(general-instructions modules/general-instructions/full.md [] RoleComposition)"
    ));
    for removed_module in [
        "agent-feedback-loop",
        "return-to-manager",
        "design-authority",
    ] {
        assert!(
            !dependency_index.contains(&format!("({removed_module} ")),
            "{removed_module} has no duplicate authoritative module entry"
        );
        assert!(
            !Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("modules")
                .join(removed_module)
                .exists(),
            "{removed_module} source is consolidated into general-instructions"
        );
    }
    for universal_clause in [language, feedback, ambiguity_return, design_authority] {
        assert!(
            general_instructions.contains(universal_clause),
            "general-instructions preserves universal guidance: {universal_clause}"
        );
    }
    for specialized_clause in [
        "Use only at fresh-context startup when the psyche wants management.",
        "Classify each instruction before placing it.",
        "Before editing shared files or running a command that writes them,",
        "Use only when the brief explicitly requests a worker output artifact",
        "Treat secret bytes as transient hazardous material.",
    ] {
        assert!(
            !general_instructions.contains(specialized_clause),
            "specialized guidance does not leak into general-instructions: {specialized_clause}"
        );
    }

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current role packets generate");

    let skill_bound_commitment = "Agents are ephemeral. In psyche-facing conversation, future behavior exists\nonly in durable role or skill instruction, never in this session's continuity,\nmemory, resolve, or persona.\n\nTreat a concrete failure as evidence that its governing guard is inadequate. Do\nnot answer it with “I will follow it more strictly,” “I will avoid this next\ntime,” or a claim that the guard is sufficient. Strengthen the owning role or\nskill guard before claiming changed future behavior, unless specific contrary\nevidence shows the guard did prevent the behavior. Until then, describe the\nchange as a proposal or pending work, not an accomplished behavioral change.\nCite the durable guard and its verification when claiming future behavior has\nchanged.";
    let manager_spirit_clause = "show\nthe psyche the exact proposed Spirit intent wording, scope, and proposed privacy,\nand receive explicit approval.";
    let codex_skill_read_clause = "A pasted `<skill ...>...</skill>` block is complete when it has matching opening\nand closing `<skill>` tags, a skill name, a location, and non-empty body text.";
    let recorder_spirit_clause = "Reject a submission brief unless it evidences that the exact proposed Spirit\nintent wording, scope, and proposed privacy were shown to and explicitly approved\nby the psyche. Never invent missing entry metadata.";
    let active_roles = [
        "manager",
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

    for role in active_roles {
        for path in [
            format!(".pi/agents/{role}.md"),
            format!(".claude/agents/{role}.md"),
            format!(".codex/agents/{role}.toml"),
        ] {
            let packet = fixture.read_workspace_file(&path).replace("\\n", "\n");
            for (name, universal_clause) in [
                ("language", language),
                ("feedback", feedback),
                ("ambiguity return", ambiguity_return),
                ("authority boundary", design_authority),
                ("execution limits", execution_limits),
            ] {
                assert_eq!(
                    packet.matches(universal_clause).count(),
                    1,
                    "{path} receives general-instructions {name} guidance exactly once through the canonical universal composition path"
                );
            }
            if role == "manager" {
                assert!(
                    packet.contains(skill_bound_commitment),
                    "{path} is the sole psyche-facing packet with skill-bound commitment guidance"
                );
            } else {
                assert!(
                    !packet.contains(skill_bound_commitment),
                    "{path} is not a psyche-facing packet"
                );
            }
            if path.ends_with(".toml") {
                assert!(
                    packet.contains(codex_skill_read_clause),
                    "{path} receives source-owned Codex skill-read guidance"
                );
            } else {
                assert!(
                    !packet.contains(codex_skill_read_clause),
                    "{path} excludes Codex-only skill-read guidance"
                );
            }
        }
    }

    for (role, clauses) in [
        (
            "scout",
            &[
                "Be skeptical and conservative: an unwitnessed cause is Unknown.",
                "Never treat a proxy metric, correlation, salient fact,\nor suspected diagnosis in the brief as causal fact; a brief's diagnosis is not\nindependent evidence.",
                "State the exact missing witnesses and confidence.",
            ][..],
        ),
        (
            "manager",
            &[
                "Require hard direct evidence for every judgment call, especially a disruptive,\nglobal, or default behavior change.",
                "Do not authorize behavioral control as a fix until its causal mechanism is\nreproduced or directly witnessed.",
                "General repair\nauthorization does not approve that concrete delta.",
            ][..],
        ),
        (
            "skill-editor",
            &[
                "Write only instructions that change a decision or action. On every skill edit,\ndelete no-op statements, restatements, aspirations, and untestable advice; delete\nor simplify non-obvious wording.",
                "Keep every skill very small and single-purpose.\nImmediately flag an emerging large or mixed-responsibility skill and propose\ndeletion or the smallest split. Do not use a numeric size threshold.",
            ][..],
        ),
    ] {
        for path in [
            format!(".pi/agents/{role}.md"),
            format!(".claude/agents/{role}.md"),
            format!(".codex/agents/{role}.toml"),
        ] {
            let packet = fixture.read_workspace_file(&path).replace("\\n", "\n");
            for clause in clauses {
                assert!(
                    packet.contains(clause),
                    "{path} preserves {role} doctrine: {clause}"
                );
            }
        }
    }

    for path in [
        ".pi/agents/manager.md",
        ".claude/agents/manager.md",
        ".codex/agents/manager.toml",
    ] {
        assert!(
            fixture
                .read_workspace_file(path)
                .replace("\\n", "\n")
                .contains(manager_spirit_clause),
            "{path} requires exact Spirit proposal approval before dispatch"
        );
    }
    assert!(
        fixture
            .read_workspace_file(".codex/agents/manager.toml")
            .replace("\\n", "\n")
            .contains(codex_skill_read_clause),
        "Codex manager packet receives its source-owned skill-read overlay"
    );
    for path in [".pi/agents/manager.md", ".claude/agents/manager.md"] {
        assert!(
            !fixture
                .read_workspace_file(path)
                .contains(codex_skill_read_clause),
            "{path} excludes the Codex-only skill-read overlay"
        );
    }
    for path in [
        ".pi/agents/intent-recorder.md",
        ".claude/agents/intent-recorder.md",
        ".codex/agents/intent-recorder.toml",
    ] {
        assert!(
            fixture
                .read_workspace_file(path)
                .replace("\\n", "\n")
                .contains(recorder_spirit_clause),
            "{path} rejects a Spirit submission without approved proposal evidence"
        );
    }
}

#[test]
fn nested_model_resolution_uses_strongest_assignment_and_ordinary_wins_ties() {
    let tie = Fixture::new();
    tie.write_model_resolution_sources("Medium");
    tie.generate(GenerationMode::Write)
        .expect("equal-strength ordinary assignments generate");
    assert!(
        tie.read_workspace_file(".pi/agents/parent.md")
            .contains("model: 'ordinary-provider/gpt-ordinary'\nthinking: medium")
    );
    assert!(
        tie.read_workspace_file(".claude/agents/parent.md")
            .contains("model: claude-ordinary\neffort: medium")
    );

    let stronger_floor = Fixture::new();
    stronger_floor.write_model_resolution_sources("High");
    stronger_floor
        .generate(GenerationMode::Write)
        .expect("stronger minimum assignments generate");
    assert!(
        stronger_floor
            .read_workspace_file(".pi/agents/parent.md")
            .contains("model: 'openai-codex/gpt-5.6-sol'\nthinking: high")
    );
    assert!(
        stronger_floor
            .read_workspace_file(".codex/agents/parent.toml")
            .contains("model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"high\"")
    );
    assert!(
        stronger_floor
            .read_workspace_file(".claude/agents/parent.md")
            .contains("model: fable-5\neffort: high")
    );
}

#[test]
fn nested_model_resolution_uses_typed_cross_model_strength_not_effort_rank() {
    let fixture = Fixture::new();
    fixture.write_cross_model_floor_sources();
    fixture
        .generate(GenerationMode::Write)
        .expect("cross-model floors generate");

    assert!(
        fixture
            .read_workspace_file(".pi/agents/parent.md")
            .contains("model: 'openai-codex/gpt-5.6-sol'\nthinking: medium")
    );
    assert!(
        fixture
            .read_workspace_file(".codex/agents/parent.toml")
            .contains("model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"medium\"")
    );
    assert!(
        fixture
            .read_workspace_file(".claude/agents/parent.md")
            .contains("model: fable-5\neffort: medium")
    );
}

#[test]
fn nested_role_validation_rejects_child_and_recursion_inconsistencies() {
    let missing = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [])]",
    );
    assert!(matches!(missing, Error::MissingNestedRoleChild { .. }));
    let duplicate_relation = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child]) (parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        duplicate_relation,
        Error::DuplicateNestedRoleRelation { .. }
    ));
    let inactive_parent =
        nested_relation_error("[(inactive [(ClaudeAgent claude-test Medium)] [child])]");
    assert!(matches!(inactive_parent, Error::InactiveNestedRole { .. }));
    let duplicate_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child child])]",
    );
    assert!(matches!(
        duplicate_child,
        Error::DuplicateNestedRoleChild { .. }
    ));
    let inactive_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [inactive])]",
    );
    assert!(matches!(
        inactive_child,
        Error::InactiveNestedRoleChild { .. }
    ));
    let incompatible_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [claude-child])]",
    );
    assert!(matches!(
        incompatible_child,
        Error::TargetIncompatibleNestedRoleChild { .. }
    ));
    let self_edge = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [parent])]",
    );
    assert!(matches!(self_edge, Error::NestedRoleSelfEdge { .. }));
    let nested_edge = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [nested-two]) (nested-two [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        nested_edge,
        Error::NestedRoleChildCannotBeNested { .. }
    ));
    let manager_nested = nested_relation_error(
        "[(manager [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(manager_nested, Error::ManagerCannotBeNestedRole));
    let manager_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [manager])]",
    );
    assert!(matches!(
        manager_child,
        Error::ManagerCannotBeNestedChild { .. }
    ));
}

#[test]
fn nested_role_validation_rejects_minimum_model_target_inconsistencies() {
    let missing = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        missing,
        Error::MissingNestedRoleMinimumModel { .. }
    ));
    let duplicate = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        duplicate,
        Error::DuplicateNestedRoleMinimumModel { .. }
    ));
    let inactive_target = nested_relation_error(
        "[(claude-child [(ClaudeAgent claude-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        inactive_target,
        Error::NestedRoleMinimumForInactiveTarget { .. }
    ));
    let wrong_family = nested_relation_error(
        "[(parent [(ClaudeAgent gpt-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        wrong_family,
        Error::NestedRoleMinimumModelFamilyMismatch { .. }
    ));
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
    assert!(pi.contains("model: 'openai-codex/gpt-test'\nthinking: high\nprojectRoleIdentity: worker\nprojectRoleDispatchKind: leaf\nskills: example"));
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
        "[(ChatGpt (gpt-test openai-codex [(High 30)])) (ChatGpt (gpt-test openai-codex [(High 30)])) (Claude (claude-test [(High 30)]))]\n",
    );
    let error = duplicate_catalog
        .generate(GenerationMode::Write)
        .expect_err("duplicate catalog entry fails");
    assert!(
        matches!(error, Error::DuplicateModelCatalogEntry { .. }),
        "{error:?}"
    );

    let duplicate_effort = Fixture::new();
    duplicate_effort.write_role_generation_sources();
    duplicate_effort.write_source_file(
        "manifests/model-catalog.nota",
        "[(ChatGpt (gpt-test openai-codex [(High 30) (High 40)])) (Claude (claude-test [(High 30)]))]\n",
    );
    let error = duplicate_effort
        .generate(GenerationMode::Write)
        .expect_err("duplicate model effort fails");
    assert!(
        matches!(error, Error::DuplicateModelCatalogEffort { .. }),
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

fn nested_relation_error(relations: &str) -> Error {
    let fixture = Fixture::new();
    fixture.write_nested_validation_sources(relations);
    fixture
        .generate(GenerationMode::Write)
        .expect_err("invalid nested-role relation fails generation")
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

    fn write_project_role_contract_sources(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (planner planner [] [Planner role.] [PiAgent])) (Role (reader reader [] [Reader role.] [PiAgent])) (Role (writer writer [] [Writer role.] [PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(planner roles/planner/full.md [] RoleSource) (reader roles/reader/full.md [] RoleSource) (writer roles/writer/full.md [] RoleSource)]\n",
        );
        self.write_role_metadata(&["planner", "reader", "writer"]);
        self.write_source_file(
            "manifests/nested-role-relations.nota",
            "[(planner [(PiAgent gpt-test Medium)] [reader writer])]\n",
        );
        self.write_source_file(
            "roles/planner/full.md",
            "# Role - planner\n\n## Contract\n\nPlan work.\n",
        );
        for role in ["reader", "writer"] {
            self.write_source_file(
                &format!("roles/{role}/full.md"),
                &format!("# Role - {role}\n\n## Contract\n\n{role} work.\n"),
            );
        }
    }

    fn write_nested_validation_sources(&self, relations: &str) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (manager manager [] [Manager role.] [ClaudeAgent CodexAgent PiAgent])) (Role (parent parent [] [Parent role.] [ClaudeAgent CodexAgent PiAgent])) (Role (nested-two nested-two [] [Nested two.] [ClaudeAgent CodexAgent PiAgent])) (Role (child child [] [Child role.] [ClaudeAgent CodexAgent PiAgent])) (Role (claude-child claude-child [] [Claude child.] [ClaudeAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(manager roles/manager/full.md [] RoleSource) (parent roles/parent/full.md [] RoleSource) (nested-two roles/nested-two/full.md [] RoleSource) (child roles/child/full.md [] RoleSource) (claude-child roles/claude-child/full.md [] RoleSource)]\n",
        );
        self.write_role_metadata(&["manager", "parent", "nested-two", "child", "claude-child"]);
        self.write_source_file("manifests/nested-role-relations.nota", relations);
    }

    fn write_cross_model_floor_sources(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (parent parent [] [Parent role.] [ClaudeAgent CodexAgent PiAgent])) (Role (child child [] [Child role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(parent roles/parent/full.md [] RoleSource) (child roles/child/full.md [] RoleSource)]\n",
        );
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-5.6-sol openai-codex [(Medium 50)])) (ChatGpt (gpt-5.6-terra openai-codex [(High 30)])) (Claude (fable-5 [(Medium 50)])) (Claude (claude-opus-4-8 [(Xhigh 40)]))]\n",
        );
        self.write_source_file(
            "manifests/role-model-assignments.nota",
            "[(parent (gpt-5.6-terra High) (claude-opus-4-8 Xhigh)) (child (gpt-5.6-terra High) (claude-opus-4-8 Xhigh))]\n",
        );
        self.write_source_file(
            "manifests/role-optional-skills.nota",
            "[(parent []) (child [])]\n",
        );
        self.write_source_file(
            "manifests/nested-role-relations.nota",
            "[(parent [(ClaudeAgent fable-5 Medium) (CodexAgent gpt-5.6-sol Medium) (PiAgent gpt-5.6-sol Medium)] [child])]\n",
        );
        for role in ["parent", "child"] {
            self.write_source_file(
                &format!("roles/{role}/full.md"),
                &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
            );
        }
    }

    fn write_model_resolution_sources(&self, minimum_effort: &str) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (parent parent [] [Parent role.] [ClaudeAgent CodexAgent PiAgent])) (Role (child child [] [Child role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(parent roles/parent/full.md [] RoleSource) (child roles/child/full.md [] RoleSource)]\n",
        );
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-ordinary ordinary-provider [(Medium 50)])) (ChatGpt (gpt-5.6-sol openai-codex [(Medium 50) (High 60)])) (Claude (claude-ordinary [(Medium 50)])) (Claude (fable-5 [(Medium 50) (High 60)]))]\n",
        );
        self.write_source_file(
            "manifests/role-model-assignments.nota",
            "[(parent (gpt-ordinary Medium) (claude-ordinary Medium)) (child (gpt-ordinary Medium) (claude-ordinary Medium))]\n",
        );
        self.write_source_file(
            "manifests/role-optional-skills.nota",
            "[(parent []) (child [])]\n",
        );
        self.write_source_file(
            "manifests/nested-role-relations.nota",
            &format!(
                "[(parent [(ClaudeAgent fable-5 {minimum_effort}) (CodexAgent gpt-5.6-sol {minimum_effort}) (PiAgent gpt-5.6-sol {minimum_effort})] [child])]\n"
            ),
        );
        for role in ["parent", "child"] {
            self.write_source_file(
                &format!("roles/{role}/full.md"),
                &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
            );
        }
    }

    fn write_role_metadata(&self, role_identifiers: &[&str]) {
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-test openai-codex [(Medium 20) (High 30)])) (Claude (claude-test [(Medium 20) (High 30)]))]\n",
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
