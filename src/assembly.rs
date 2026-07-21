use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::PathBuf,
};

use nota::{NotaDecode, NotaEncode, NotaSource};

use crate::{
    error::{Error, Result},
    markdown::{MarkdownAssembly, MarkdownFragment},
    schema::assembly::{
        ActiveOutput, ActiveOutputs, ActiveRole, ActiveSkill, AssembledModule, ByteCount,
        ChatGptModelAssignment, ClaudeModelAssignment, EffortLevel, EntryPoint, EntryPointExtra,
        ExtraSurface, Frontmatter, FrontmatterEntry, FrontmatterKey, FrontmatterValue,
        GeneratedFile, GeneratedFiles, GeneratedRoleOutputs, GenerationMode, GenerationOutcome,
        GenerationReport, GenerationRequest, Manifest, ManifestPath, ModelCatalog,
        ModelCatalogEntry, ModelEffortStrength, ModelIdentifier, ModelStrength, ModuleDependencies,
        ModuleDependency, ModuleIdentifier, ModuleKind, ModuleLifecycle, ModulePath, Modules,
        NestedRoleMinimumModel, NestedRoleRelations, Operation, OptionalSkills, OutputIdentifier,
        OutputKind, OutputPath, OutputSurface, RoleModelAssignment, RoleModelAssignments,
        RoleOptionalSkills, RoleTargetSurface, SkillMetadata, SkillModule, SkillRoster,
        TargetModuleInsertion, TargetModuleInsertions, TargetSurface, UniversalFullAgentModules,
    },
    trunk_guard::TrunkDescendantGuard,
    workspace_path::WorkspacePath,
};

const GENERATED_SKILL_BLOCK_BYTE_LIMIT: usize = 32 * 1024;
const RETIRED_CURRENT_DESTINATION_PHRASES: &[&str] = &[
    "Repo Operator",
    "Weave Operator",
    "Intent Maintainer",
    "workspace essence",
    "workspace intent",
];
const EXECUTION_LIMIT_FIELD_PARTS: &[&str] = &["budget", "deadline", "limit", "timeout"];
const EXECUTION_LIMIT_MAXIMUMS: &[&str] = &[
    "cost",
    "iteration",
    "output",
    "retry",
    "runtime",
    "time",
    "token",
    "tool",
    "turn",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandLine {
    arguments: Vec<String>,
}

impl CommandLine {
    pub fn from_environment() -> Self {
        Self {
            arguments: env::args().skip(1).collect(),
        }
    }

    pub fn run(&self) -> Result<GenerationOutcome> {
        let operation = RequestArgument::new(self.arguments.clone())
            .read()?
            .parse()?;
        operation.guard_source()?;
        operation.execute()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequestArgument {
    arguments: Vec<String>,
}

impl RequestArgument {
    fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }

    fn read(self) -> Result<RequestText> {
        if self.arguments.len() != 1 {
            return Err(Error::ArgumentCount {
                count: self.arguments.len(),
            });
        }
        let argument = self.arguments.into_iter().next().expect("length checked");
        if argument.trim_start().starts_with('(') {
            Ok(RequestText::new(argument))
        } else {
            RequestFile::new(PathBuf::from(argument)).read()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequestFile {
    path: PathBuf,
}

impl RequestFile {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn read(self) -> Result<RequestText> {
        fs::read_to_string(&self.path)
            .map(RequestText::new)
            .map_err(|source| Error::ReadFile {
                path: self.path,
                source,
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequestText {
    text: String,
}

impl RequestText {
    fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    fn parse(&self) -> Result<Operation> {
        NotaSource::new(&self.text)
            .parse::<Operation>()
            .map_err(Error::DecodeNotaArgument)
    }
}

impl Operation {
    fn guard_source(&self) -> Result<()> {
        match self {
            Self::Generate(request) => request.guard_source(),
        }
    }

    pub fn execute(self) -> Result<GenerationOutcome> {
        match self {
            Self::Generate(request) => request.generate().map(GenerationOutcome::Generated),
        }
    }
}

impl GenerationRequest {
    fn guard_source(&self) -> Result<()> {
        let source_root = RootPath::new(self.source_root.as_ref()).to_path_buf()?;
        TrunkDescendantGuard::new(source_root).verify()
    }

    pub fn generate(&self) -> Result<GenerationReport> {
        let source_root = RootPath::new(self.source_root.as_ref()).to_path_buf()?;
        let workspace_root = RootPath::new(self.workspace_root.as_ref()).to_path_buf()?;
        let configuration =
            GenerationSource::new(source_root.clone(), self.manifest_path.clone()).read()?;
        let jobs = GenerationJobs::new(source_root, workspace_root.clone(), configuration.clone())
            .jobs()?;
        if self.generation_mode == GenerationMode::Write {
            WorkspacePruner::new(workspace_root.clone(), configuration.clone()).prune()?;
        }
        let mut files = Vec::new();
        for job in jobs {
            files.push(job.generate(self.generation_mode)?);
        }
        StaleOutputScan::new(workspace_root, configuration).validate()?;
        Ok(GenerationReport::new(GeneratedFiles::new(files)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RootPath<'a> {
    text: &'a str,
}

impl<'a> RootPath<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn to_path_buf(&self) -> Result<PathBuf> {
        if let Some(name) = self.text.strip_prefix('$') {
            return env::var_os(name).map(PathBuf::from).ok_or_else(|| {
                Error::MissingEnvironmentRoot {
                    variable: name.to_owned(),
                }
            });
        }
        Ok(PathBuf::from(self.text))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationSource {
    source_root: PathBuf,
    manifest_path: ManifestPath,
}

impl GenerationSource {
    fn new(source_root: PathBuf, manifest_path: ManifestPath) -> Self {
        Self {
            source_root,
            manifest_path,
        }
    }

    fn read(&self) -> Result<GenerationConfiguration> {
        if self.manifest_path.as_ref().ends_with("skills-roster.nota") {
            return self.read_legacy_roster();
        }
        let active_outputs: ActiveOutputs = SourceFile::new(
            self.source_root.clone(),
            PathBuf::from(self.manifest_path.as_ref()),
        )
        .read()?;
        let dependency_path = PathBuf::from(self.manifest_path.as_ref())
            .parent()
            .map(|parent| parent.join("module-dependencies.nota"))
            .unwrap_or_else(|| PathBuf::from("module-dependencies.nota"));
        let module_dependencies: ModuleDependencies =
            SourceFile::new(self.source_root.clone(), dependency_path).read()?;
        let manifest_directory = PathBuf::from(self.manifest_path.as_ref())
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default();
        let insertion_path = manifest_directory.join("target-module-insertions.nota");
        let target_module_insertions: TargetModuleInsertions =
            SourceFile::new(self.source_root.clone(), insertion_path)
                .read_optional()?
                .unwrap_or_else(|| TargetModuleInsertions::new(Vec::new()));
        let universal_full_agent_modules_path =
            manifest_directory.join("universal-full-agent-modules.nota");
        let universal_full_agent_modules: UniversalFullAgentModules =
            SourceFile::new(self.source_root.clone(), universal_full_agent_modules_path)
                .read_optional()?
                .unwrap_or_else(|| UniversalFullAgentModules::new(Vec::new()));
        let model_catalog_path = manifest_directory.join("model-catalog.nota");
        let model_catalog: ModelCatalog =
            SourceFile::new(self.source_root.clone(), model_catalog_path)
                .read_optional()?
                .unwrap_or_else(|| ModelCatalog::new(Vec::new()));
        let role_model_assignments_path = manifest_directory.join("role-model-assignments.nota");
        let role_model_assignments: RoleModelAssignments =
            SourceFile::new(self.source_root.clone(), role_model_assignments_path)
                .read_optional()?
                .unwrap_or_else(|| RoleModelAssignments::new(Vec::new()));
        let role_optional_skills_path = manifest_directory.join("role-optional-skills.nota");
        let role_optional_skills: RoleOptionalSkills =
            SourceFile::new(self.source_root.clone(), role_optional_skills_path)
                .read_optional()?
                .unwrap_or_else(|| RoleOptionalSkills::new(Vec::new()));
        let nested_role_relations_path = manifest_directory.join("nested-role-relations.nota");
        let nested_role_relations: NestedRoleRelations =
            SourceFile::new(self.source_root.clone(), nested_role_relations_path)
                .read_optional()?
                .unwrap_or_else(|| NestedRoleRelations::new(Vec::new()));
        GenerationConfiguration::active(
            active_outputs,
            module_dependencies,
            target_module_insertions,
            universal_full_agent_modules,
            RoleMetadataSources {
                model_catalog,
                role_model_assignments,
                role_optional_skills,
                nested_role_relations,
            },
        )
    }

    fn read_legacy_roster(&self) -> Result<GenerationConfiguration> {
        let roster: SkillRoster = SourceFile::new(
            self.source_root.clone(),
            PathBuf::from(self.manifest_path.as_ref()),
        )
        .read()?;
        let manifest_directory = PathBuf::from(self.manifest_path.as_ref())
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default();
        let module_dependencies = SourceFile::new(
            self.source_root.clone(),
            manifest_directory.join("module-dependencies.nota"),
        )
        .read_optional()?
        .unwrap_or_else(|| roster.module_dependencies());
        let target_module_insertions = SourceFile::new(
            self.source_root.clone(),
            manifest_directory.join("target-module-insertions.nota"),
        )
        .read_optional()?
        .unwrap_or_else(|| TargetModuleInsertions::new(Vec::new()));
        let universal_full_agent_modules = SourceFile::new(
            self.source_root.clone(),
            manifest_directory.join("universal-full-agent-modules.nota"),
        )
        .read_optional()?
        .unwrap_or_else(|| UniversalFullAgentModules::new(Vec::new()));
        GenerationConfiguration::legacy(
            roster,
            module_dependencies,
            target_module_insertions,
            universal_full_agent_modules,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceFile {
    source_root: PathBuf,
    relative_path: PathBuf,
}

impl SourceFile {
    fn new(source_root: PathBuf, relative_path: PathBuf) -> Self {
        Self {
            source_root,
            relative_path,
        }
    }

    fn read<T>(&self) -> Result<T>
    where
        T: NotaDecode,
    {
        let workspace_path =
            WorkspacePath::new(self.source_root.clone(), self.relative_path.clone())?;
        let path = workspace_path.full_path();
        let text = fs::read_to_string(&path).map_err(|source| Error::ReadFile {
            path: path.clone(),
            source,
        })?;
        NotaSource::new(&text)
            .parse::<T>()
            .map_err(|source| Error::DecodeNota { path, source })
    }

    fn read_optional<T>(&self) -> Result<Option<T>>
    where
        T: NotaDecode,
    {
        let workspace_path =
            WorkspacePath::new(self.source_root.clone(), self.relative_path.clone())?;
        let path = workspace_path.full_path();
        match fs::read_to_string(&path) {
            Ok(text) => NotaSource::new(&text)
                .parse::<T>()
                .map(Some)
                .map_err(|source| Error::DecodeNota { path, source }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(Error::ReadFile { path, source }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationJobs {
    source_root: PathBuf,
    workspace_root: PathBuf,
    configuration: GenerationConfiguration,
}

impl GenerationJobs {
    fn new(
        source_root: PathBuf,
        workspace_root: PathBuf,
        configuration: GenerationConfiguration,
    ) -> Self {
        Self {
            source_root,
            workspace_root,
            configuration,
        }
    }

    fn jobs(&self) -> Result<Vec<GenerationJob>> {
        let mut jobs = Vec::new();
        for manifest in self.configuration.skill_manifests()? {
            jobs.push(GenerationJob::Manifest(ManifestAssembler::new(
                self.source_root.clone(),
                self.workspace_root.clone(),
                manifest,
            )));
        }
        for manifest in self.configuration.role_manifests()? {
            let generated_roster = self.configuration.generated_role_roster(&manifest);
            let mut assembler = ManifestAssembler::new(
                self.source_root.clone(),
                self.workspace_root.clone(),
                manifest,
            );
            if let Some(roster) = generated_roster {
                assembler = assembler.with_generated_fragment(roster);
            }
            jobs.push(GenerationJob::Manifest(assembler));
        }
        if self.configuration.uses_active_manifest() {
            jobs.push(GenerationJob::Rendered(RenderedOutput::new(
                self.workspace_root.clone(),
                RoleOutputInventory::relative_path(),
                self.configuration.role_output_inventory().render(),
            )?));
        }
        if let Some(roster) = self.configuration.compatibility_roster() {
            for entry_point in roster.entry_points.payload() {
                for manifest in entry_point.extra_manifests()? {
                    jobs.push(GenerationJob::Manifest(ManifestAssembler::new(
                        self.source_root.clone(),
                        self.workspace_root.clone(),
                        manifest,
                    )));
                }
            }
        }
        OutputPathIndex::new().validate(&jobs)?;
        Ok(jobs)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleMetadataIndex {
    entries: BTreeMap<OutputIdentifier, RoleMetadata>,
    nested_roles: NestedRoleIndex,
}

impl RoleMetadataIndex {
    fn new(
        active_outputs: &ActiveOutputs,
        model_catalog: ModelCatalog,
        role_model_assignments: RoleModelAssignments,
        role_optional_skills: RoleOptionalSkills,
        nested_role_relations: NestedRoleRelations,
    ) -> Result<Self> {
        let active_roles: BTreeMap<OutputIdentifier, ActiveRole> = active_outputs
            .payload()
            .iter()
            .filter_map(|output| match output {
                ActiveOutput::FullAgentRole(role) => {
                    Some((role.output_identifier.clone(), role.clone()))
                }
                ActiveOutput::Skill(_) | ActiveOutput::FullAgentSkill(_) => None,
            })
            .collect();
        let active_skills: BTreeMap<OutputIdentifier, ActiveSkill> = active_outputs
            .payload()
            .iter()
            .filter_map(|output| match output {
                ActiveOutput::Skill(skill) | ActiveOutput::FullAgentSkill(skill) => {
                    Some((skill.output_identifier.clone(), skill.clone()))
                }
                ActiveOutput::FullAgentRole(_) => None,
            })
            .collect();
        let catalog = ModelCatalogIndex::new(model_catalog)?;
        let nested_roles = NestedRoleIndex::new(nested_role_relations, &active_roles, &catalog)?;
        let assignments = RoleModelAssignmentIndex::new(role_model_assignments, &active_roles)?;
        let optional_skills =
            RoleOptionalSkillIndex::new(role_optional_skills, &active_roles, &active_skills)?;
        let mut entries = BTreeMap::new();
        for role_identifier in active_roles.into_keys() {
            let assignment = assignments.entries.get(&role_identifier).ok_or_else(|| {
                Error::MissingRoleModelAssignment {
                    role_identifier: role_identifier.as_ref().to_owned(),
                }
            })?;
            let optional = optional_skills
                .entries
                .get(&role_identifier)
                .ok_or_else(|| Error::MissingRoleOptionalSkills {
                    role_identifier: role_identifier.as_ref().to_owned(),
                })?;
            let profile = catalog.profile(role_identifier.as_ref(), assignment)?;
            entries.insert(
                role_identifier,
                RoleMetadata {
                    profile,
                    optional_skills: optional.clone(),
                },
            );
        }
        Ok(Self {
            entries,
            nested_roles,
        })
    }

    fn metadata(&self, role_identifier: &OutputIdentifier) -> Result<&RoleMetadata> {
        self.entries
            .get(role_identifier)
            .ok_or_else(|| Error::MissingRoleModelAssignment {
                role_identifier: role_identifier.as_ref().to_owned(),
            })
    }

    fn nested_role(&self, role_identifier: &OutputIdentifier) -> Option<&NestedRoleMetadata> {
        self.nested_roles.entries.get(role_identifier)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleMetadata {
    profile: RoleModelProfile,
    optional_skills: OptionalSkills,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleModelProfile {
    chat_gpt_model: String,
    pi_provider: String,
    chat_gpt_effort: EffortLevel,
    chat_gpt_strength: ModelStrength,
    claude_model: String,
    claude_effort: EffortLevel,
    claude_strength: ModelStrength,
}

impl RoleModelProfile {
    fn target(&self, output_surface: OutputSurface) -> TargetModelProfile {
        match output_surface {
            OutputSurface::ClaudeAgent => TargetModelProfile {
                model_identifier: self.claude_model.clone(),
                pi_provider: None,
                effort_level: self.claude_effort,
                model_strength: self.claude_strength.clone(),
            },
            OutputSurface::CodexAgent => TargetModelProfile {
                model_identifier: self.chat_gpt_model.clone(),
                pi_provider: None,
                effort_level: self.chat_gpt_effort,
                model_strength: self.chat_gpt_strength.clone(),
            },
            OutputSurface::PiAgent => TargetModelProfile {
                model_identifier: self.chat_gpt_model.clone(),
                pi_provider: Some(self.pi_provider.clone()),
                effort_level: self.chat_gpt_effort,
                model_strength: self.chat_gpt_strength.clone(),
            },
            _ => unreachable!("target model profiles exist only for role surfaces"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TargetModelProfile {
    model_identifier: String,
    pi_provider: Option<String>,
    effort_level: EffortLevel,
    model_strength: ModelStrength,
}

impl TargetModelProfile {
    fn strongest(ordinary: Self, minimum: Option<&Self>) -> Self {
        match minimum {
            Some(minimum)
                if minimum.model_strength.payload() > ordinary.model_strength.payload() =>
            {
                minimum.clone()
            }
            Some(_) | None => ordinary,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NestedRoleIndex {
    entries: BTreeMap<OutputIdentifier, NestedRoleMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NestedRoleMetadata {
    minimum_models: Vec<(RoleTargetSurface, TargetModelProfile)>,
    allowed_leaf_roles: Vec<OutputIdentifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModelCatalogIndex {
    entries: BTreeMap<ModelIdentifier, CatalogModel>,
}

impl ModelCatalogIndex {
    fn new(model_catalog: ModelCatalog) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for entry in model_catalog.into_payload() {
            let (identifier, model) = match entry {
                ModelCatalogEntry::ChatGpt(model) => {
                    let identifier = model.model_identifier.clone();
                    let effort_strengths = model.model_effort_strengths.into_payload();
                    Self::validate_catalog_efforts(&identifier, &effort_strengths)?;
                    (
                        identifier,
                        CatalogModel::ChatGpt {
                            pi_provider: model.pi_provider.into_payload(),
                            effort_strengths,
                        },
                    )
                }
                ModelCatalogEntry::Claude(model) => {
                    let identifier = model.model_identifier.clone();
                    let effort_strengths = model.model_effort_strengths.into_payload();
                    Self::validate_catalog_efforts(&identifier, &effort_strengths)?;
                    (identifier, CatalogModel::Claude { effort_strengths })
                }
            };
            if entries.insert(identifier.clone(), model).is_some() {
                return Err(Error::DuplicateModelCatalogEntry {
                    model_identifier: identifier.into_payload(),
                });
            }
        }
        Ok(Self { entries })
    }

    fn validate_catalog_efforts(
        model_identifier: &ModelIdentifier,
        effort_strengths: &[ModelEffortStrength],
    ) -> Result<()> {
        let mut seen = BTreeSet::new();
        for profile in effort_strengths {
            if !seen.insert(profile.effort_level.as_str()) {
                return Err(Error::DuplicateModelCatalogEffort {
                    model_identifier: model_identifier.as_ref().to_owned(),
                    effort: profile.effort_level.as_str().to_owned(),
                });
            }
        }
        Ok(())
    }

    fn profile(
        &self,
        role_identifier: &str,
        assignment: &RoleModelAssignment,
    ) -> Result<RoleModelProfile> {
        let (chat_gpt_model, pi_provider, chat_gpt_effort, chat_gpt_strength) =
            self.chat_gpt_assignment(role_identifier, &assignment.chat_gpt_model_assignment)?;
        let (claude_model, claude_effort, claude_strength) =
            self.claude_assignment(role_identifier, &assignment.claude_model_assignment)?;
        Ok(RoleModelProfile {
            chat_gpt_model,
            pi_provider,
            chat_gpt_effort,
            chat_gpt_strength,
            claude_model,
            claude_effort,
            claude_strength,
        })
    }

    fn chat_gpt_assignment(
        &self,
        role_identifier: &str,
        assignment: &ChatGptModelAssignment,
    ) -> Result<(String, String, EffortLevel, ModelStrength)> {
        let identifier = assignment.model_identifier.as_ref();
        let model = self.model(role_identifier, identifier)?;
        match model {
            CatalogModel::ChatGpt {
                pi_provider,
                effort_strengths,
            } => {
                let strength = self.model_strength(
                    role_identifier,
                    identifier,
                    assignment.effort_level,
                    effort_strengths,
                )?;
                Ok((
                    identifier.to_owned(),
                    pi_provider.clone(),
                    assignment.effort_level,
                    strength,
                ))
            }
            CatalogModel::Claude { .. } => Err(Error::RoleModelFamilyMismatch {
                role_identifier: role_identifier.to_owned(),
                model_identifier: identifier.to_owned(),
                expected_family: "ChatGPT".to_owned(),
                actual_family: "Claude".to_owned(),
            }),
        }
    }

    fn claude_assignment(
        &self,
        role_identifier: &str,
        assignment: &ClaudeModelAssignment,
    ) -> Result<(String, EffortLevel, ModelStrength)> {
        let identifier = assignment.model_identifier.as_ref();
        let model = self.model(role_identifier, identifier)?;
        match model {
            CatalogModel::Claude { effort_strengths } => {
                let strength = self.model_strength(
                    role_identifier,
                    identifier,
                    assignment.effort_level,
                    effort_strengths,
                )?;
                Ok((identifier.to_owned(), assignment.effort_level, strength))
            }
            CatalogModel::ChatGpt { .. } => Err(Error::RoleModelFamilyMismatch {
                role_identifier: role_identifier.to_owned(),
                model_identifier: identifier.to_owned(),
                expected_family: "Claude".to_owned(),
                actual_family: "ChatGPT".to_owned(),
            }),
        }
    }

    fn model(&self, role_identifier: &str, model_identifier: &str) -> Result<&CatalogModel> {
        self.entries
            .get(&ModelIdentifier::new(model_identifier))
            .ok_or_else(|| Error::UnsupportedRoleModel {
                role_identifier: role_identifier.to_owned(),
                model_identifier: model_identifier.to_owned(),
            })
    }

    fn model_strength(
        &self,
        role_identifier: &str,
        model_identifier: &str,
        effort: EffortLevel,
        effort_strengths: &[ModelEffortStrength],
    ) -> Result<ModelStrength> {
        effort_strengths
            .iter()
            .find(|profile| profile.effort_level == effort)
            .map(|profile| profile.model_strength.clone())
            .ok_or_else(|| Error::UnsupportedRoleModelEffort {
                role_identifier: role_identifier.to_owned(),
                model_identifier: model_identifier.to_owned(),
                effort: effort.as_str().to_owned(),
            })
    }

    fn nested_minimum(
        &self,
        role_identifier: &str,
        minimum: &NestedRoleMinimumModel,
    ) -> Result<TargetModelProfile> {
        let identifier = minimum.model_identifier.as_ref();
        let model = self.model(role_identifier, identifier)?;
        match (&minimum.role_target_surface, model) {
            (
                RoleTargetSurface::CodexAgent | RoleTargetSurface::PiAgent,
                CatalogModel::ChatGpt {
                    pi_provider,
                    effort_strengths,
                },
            ) => {
                let model_strength = self.model_strength(
                    role_identifier,
                    identifier,
                    minimum.effort_level,
                    effort_strengths,
                )?;
                Ok(TargetModelProfile {
                    model_identifier: identifier.to_owned(),
                    pi_provider: match minimum.role_target_surface {
                        RoleTargetSurface::PiAgent => Some(pi_provider.clone()),
                        RoleTargetSurface::CodexAgent => None,
                        RoleTargetSurface::ClaudeAgent => unreachable!(),
                    },
                    effort_level: minimum.effort_level,
                    model_strength,
                })
            }
            (RoleTargetSurface::ClaudeAgent, CatalogModel::Claude { effort_strengths }) => {
                let model_strength = self.model_strength(
                    role_identifier,
                    identifier,
                    minimum.effort_level,
                    effort_strengths,
                )?;
                Ok(TargetModelProfile {
                    model_identifier: identifier.to_owned(),
                    pi_provider: None,
                    effort_level: minimum.effort_level,
                    model_strength,
                })
            }
            (surface, CatalogModel::ChatGpt { .. }) => {
                Err(Error::NestedRoleMinimumModelFamilyMismatch {
                    role_identifier: role_identifier.to_owned(),
                    model_identifier: identifier.to_owned(),
                    role_surface: format!("{surface:?}"),
                    expected_family: "Claude".to_owned(),
                    actual_family: "ChatGPT".to_owned(),
                })
            }
            (surface, CatalogModel::Claude { .. }) => {
                Err(Error::NestedRoleMinimumModelFamilyMismatch {
                    role_identifier: role_identifier.to_owned(),
                    model_identifier: identifier.to_owned(),
                    role_surface: format!("{surface:?}"),
                    expected_family: "ChatGPT".to_owned(),
                    actual_family: "Claude".to_owned(),
                })
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CatalogModel {
    ChatGpt {
        pi_provider: String,
        effort_strengths: Vec<ModelEffortStrength>,
    },
    Claude {
        effort_strengths: Vec<ModelEffortStrength>,
    },
}

impl NestedRoleIndex {
    fn new(
        relations: NestedRoleRelations,
        active_roles: &BTreeMap<OutputIdentifier, ActiveRole>,
        catalog: &ModelCatalogIndex,
    ) -> Result<Self> {
        let mut relations_by_role = BTreeMap::new();
        for relation in relations.into_payload() {
            let role_identifier = relation.output_identifier.clone();
            if relations_by_role
                .insert(role_identifier.clone(), relation)
                .is_some()
            {
                return Err(Error::DuplicateNestedRoleRelation {
                    role_identifier: role_identifier.into_payload(),
                });
            }
        }
        let nested_role_identifiers: BTreeSet<OutputIdentifier> =
            relations_by_role.keys().cloned().collect();
        let mut entries = BTreeMap::new();
        for (role_identifier, relation) in relations_by_role {
            let role =
                active_roles
                    .get(&role_identifier)
                    .ok_or_else(|| Error::InactiveNestedRole {
                        role_identifier: role_identifier.as_ref().to_owned(),
                    })?;
            if role_identifier.as_ref() == "manager" {
                return Err(Error::ManagerCannotBeNestedRole);
            }

            let mut minimum_models = Vec::new();
            for minimum in relation.nested_role_minimum_models.into_payload() {
                let surface = minimum.role_target_surface;
                if minimum_models
                    .iter()
                    .any(|(recorded_surface, _)| *recorded_surface == surface)
                {
                    return Err(Error::DuplicateNestedRoleMinimumModel {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        role_surface: format!("{surface:?}"),
                    });
                }
                if !role.role_target_surfaces.payload().contains(&surface) {
                    return Err(Error::NestedRoleMinimumForInactiveTarget {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        role_surface: format!("{surface:?}"),
                    });
                }
                let profile = catalog.nested_minimum(role_identifier.as_ref(), &minimum)?;
                minimum_models.push((surface, profile));
            }
            for surface in role.role_target_surfaces.payload() {
                if !minimum_models
                    .iter()
                    .any(|(recorded_surface, _)| recorded_surface == surface)
                {
                    return Err(Error::MissingNestedRoleMinimumModel {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        role_surface: format!("{surface:?}"),
                    });
                }
            }

            let allowed_leaf_roles = relation.allowed_leaf_roles.into_payload();
            if allowed_leaf_roles.is_empty() {
                return Err(Error::MissingNestedRoleChild {
                    role_identifier: role_identifier.as_ref().to_owned(),
                });
            }
            let mut seen_children = BTreeSet::new();
            for child_identifier in &allowed_leaf_roles {
                if !seen_children.insert(child_identifier.as_ref()) {
                    return Err(Error::DuplicateNestedRoleChild {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        child_identifier: child_identifier.as_ref().to_owned(),
                    });
                }
                if child_identifier == &role_identifier {
                    return Err(Error::NestedRoleSelfEdge {
                        role_identifier: role_identifier.as_ref().to_owned(),
                    });
                }
                if child_identifier.as_ref() == "manager" {
                    return Err(Error::ManagerCannotBeNestedChild {
                        role_identifier: role_identifier.as_ref().to_owned(),
                    });
                }
                if nested_role_identifiers.contains(child_identifier) {
                    return Err(Error::NestedRoleChildCannotBeNested {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        child_identifier: child_identifier.as_ref().to_owned(),
                    });
                }
                let child = active_roles.get(child_identifier).ok_or_else(|| {
                    Error::InactiveNestedRoleChild {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        child_identifier: child_identifier.as_ref().to_owned(),
                    }
                })?;
                for surface in role.role_target_surfaces.payload() {
                    if !child.role_target_surfaces.payload().contains(surface) {
                        return Err(Error::TargetIncompatibleNestedRoleChild {
                            role_identifier: role_identifier.as_ref().to_owned(),
                            child_identifier: child_identifier.as_ref().to_owned(),
                            role_surface: format!("{surface:?}"),
                        });
                    }
                }
            }
            entries.insert(
                role_identifier,
                NestedRoleMetadata {
                    minimum_models,
                    allowed_leaf_roles,
                },
            );
        }
        Ok(Self { entries })
    }
}

impl NestedRoleMetadata {
    fn minimum_model(&self, output_surface: OutputSurface) -> Option<&TargetModelProfile> {
        let role_surface = output_surface.role_target_surface();
        self.minimum_models
            .iter()
            .find(|(surface, _)| *surface == role_surface)
            .map(|(_, profile)| profile)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleModelAssignmentIndex {
    entries: BTreeMap<OutputIdentifier, RoleModelAssignment>,
}

impl RoleModelAssignmentIndex {
    fn new(
        assignments: RoleModelAssignments,
        active_roles: &BTreeMap<OutputIdentifier, ActiveRole>,
    ) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for assignment in assignments.into_payload() {
            let role_identifier = assignment.output_identifier.clone();
            if !active_roles.contains_key(&role_identifier) {
                return Err(Error::StaleRoleModelAssignment {
                    role_identifier: role_identifier.into_payload(),
                });
            }
            if entries
                .insert(role_identifier.clone(), assignment)
                .is_some()
            {
                return Err(Error::DuplicateRoleModelAssignment {
                    role_identifier: role_identifier.into_payload(),
                });
            }
        }
        Ok(Self { entries })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleOptionalSkillIndex {
    entries: BTreeMap<OutputIdentifier, OptionalSkills>,
}

impl RoleOptionalSkillIndex {
    fn new(
        role_optional_skills: RoleOptionalSkills,
        active_roles: &BTreeMap<OutputIdentifier, ActiveRole>,
        active_skills: &BTreeMap<OutputIdentifier, ActiveSkill>,
    ) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for role_optional_skill in role_optional_skills.into_payload() {
            let role_identifier = role_optional_skill.output_identifier;
            let role = active_roles.get(&role_identifier).ok_or_else(|| {
                Error::StaleRoleOptionalSkills {
                    role_identifier: role_identifier.as_ref().to_owned(),
                }
            })?;
            let mut seen = BTreeSet::new();
            for skill_identifier in role_optional_skill.optional_skills.payload() {
                if !seen.insert(skill_identifier.as_ref()) {
                    return Err(Error::DuplicateOptionalSkill {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        skill_identifier: skill_identifier.as_ref().to_owned(),
                    });
                }
                let skill = active_skills.get(skill_identifier).ok_or_else(|| {
                    Error::MissingOptionalSkill {
                        role_identifier: role_identifier.as_ref().to_owned(),
                        skill_identifier: skill_identifier.as_ref().to_owned(),
                    }
                })?;
                Self::validate_targets(role, skill)?;
            }
            if entries
                .insert(role_identifier.clone(), role_optional_skill.optional_skills)
                .is_some()
            {
                return Err(Error::DuplicateRoleOptionalSkills {
                    role_identifier: role_identifier.into_payload(),
                });
            }
        }
        Ok(Self { entries })
    }

    fn validate_targets(role: &ActiveRole, skill: &ActiveSkill) -> Result<()> {
        for role_surface in role.role_target_surfaces.payload() {
            let required_skill_surface = match role_surface {
                RoleTargetSurface::ClaudeAgent => TargetSurface::ClaudeSkill,
                RoleTargetSurface::CodexAgent | RoleTargetSurface::PiAgent => {
                    TargetSurface::AgentsSkill
                }
            };
            if !skill
                .target_surfaces
                .payload()
                .contains(&required_skill_surface)
            {
                return Err(Error::TargetIncompatibleOptionalSkill {
                    role_identifier: role.output_identifier.as_ref().to_owned(),
                    skill_identifier: skill.output_identifier.as_ref().to_owned(),
                    role_surface: format!("{role_surface:?}"),
                });
            }
        }
        Ok(())
    }
}

impl EffortLevel {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleMetadataSources {
    model_catalog: ModelCatalog,
    role_model_assignments: RoleModelAssignments,
    role_optional_skills: RoleOptionalSkills,
    nested_role_relations: NestedRoleRelations,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationConfiguration {
    active_outputs: ActiveOutputs,
    module_dependencies: ModuleDependencies,
    target_module_insertions: TargetModuleInsertions,
    universal_full_agent_modules: UniversalFullAgentModules,
    role_metadata: Option<RoleMetadataIndex>,
    compatibility_roster: Option<SkillRoster>,
}

impl GenerationConfiguration {
    fn active(
        active_outputs: ActiveOutputs,
        module_dependencies: ModuleDependencies,
        target_module_insertions: TargetModuleInsertions,
        universal_full_agent_modules: UniversalFullAgentModules,
        role_metadata_sources: RoleMetadataSources,
    ) -> Result<Self> {
        let module_index = ModuleIndex::new(
            module_dependencies.clone(),
            target_module_insertions.clone(),
        )?;
        module_index.validate_universal_full_agent_modules(&universal_full_agent_modules)?;
        let role_metadata = RoleMetadataIndex::new(
            &active_outputs,
            role_metadata_sources.model_catalog,
            role_metadata_sources.role_model_assignments,
            role_metadata_sources.role_optional_skills,
            role_metadata_sources.nested_role_relations,
        )?;
        Ok(Self {
            active_outputs,
            module_dependencies,
            target_module_insertions,
            universal_full_agent_modules,
            role_metadata: Some(role_metadata),
            compatibility_roster: None,
        })
    }

    fn legacy(
        roster: SkillRoster,
        module_dependencies: ModuleDependencies,
        target_module_insertions: TargetModuleInsertions,
        universal_full_agent_modules: UniversalFullAgentModules,
    ) -> Result<Self> {
        let module_index = ModuleIndex::new(
            module_dependencies.clone(),
            target_module_insertions.clone(),
        )?;
        module_index.validate_universal_full_agent_modules(&universal_full_agent_modules)?;
        Ok(Self {
            active_outputs: roster.active_outputs(),
            module_dependencies,
            target_module_insertions,
            universal_full_agent_modules,
            role_metadata: None,
            compatibility_roster: Some(roster),
        })
    }

    fn uses_active_manifest(&self) -> bool {
        self.compatibility_roster.is_none()
    }

    fn compatibility_roster(&self) -> Option<&SkillRoster> {
        self.compatibility_roster.as_ref()
    }

    fn active_skills(&self) -> Vec<(ActiveSkill, bool)> {
        self.active_outputs
            .payload()
            .iter()
            .filter_map(|output| match output {
                ActiveOutput::Skill(skill) => Some((skill.clone(), false)),
                ActiveOutput::FullAgentSkill(skill) => Some((skill.clone(), true)),
                ActiveOutput::FullAgentRole(_) => None,
            })
            .collect()
    }

    fn active_roles(&self) -> Vec<ActiveRole> {
        self.active_outputs
            .payload()
            .iter()
            .filter_map(|output| match output {
                ActiveOutput::FullAgentRole(role) => Some(role.clone()),
                ActiveOutput::Skill(_) | ActiveOutput::FullAgentSkill(_) => None,
            })
            .collect()
    }

    fn skill_manifests(&self) -> Result<Vec<Manifest>> {
        let module_index = ModuleIndex::new(
            self.module_dependencies.clone(),
            self.target_module_insertions.clone(),
        )?;
        let mut manifests = Vec::new();
        for (skill, is_full_agent) in self.active_skills() {
            for manifest in skill.first_class_manifests(
                &module_index,
                &self.universal_full_agent_modules,
                is_full_agent,
            )? {
                manifests.push(manifest);
            }
        }
        Ok(manifests)
    }

    fn role_manifests(&self) -> Result<Vec<Manifest>> {
        let module_index = ModuleIndex::new(
            self.module_dependencies.clone(),
            self.target_module_insertions.clone(),
        )?;
        let active_roles = self.active_roles();
        let mut manifests = Vec::new();
        for role in &active_roles {
            let role_metadata = self
                .role_metadata
                .as_ref()
                .expect("active role generation has validated metadata");
            let metadata = role_metadata.metadata(&role.output_identifier)?;
            for manifest in role.manifests(
                &module_index,
                &self.universal_full_agent_modules,
                metadata,
                role_metadata.nested_role(&role.output_identifier),
                &active_roles,
            )? {
                manifests.push(manifest);
            }
        }
        Ok(manifests)
    }

    fn allowed_child_roles(
        &self,
        role_identifier: &OutputIdentifier,
        output_surface: OutputSurface,
    ) -> Vec<ActiveRole> {
        let active_roles = self.active_roles();
        if role_identifier.as_ref() == "manager" {
            return active_roles
                .into_iter()
                .filter(|role| {
                    role.output_identifier.as_ref() != "manager"
                        && role
                            .role_target_surfaces
                            .payload()
                            .contains(&output_surface.role_target_surface())
                })
                .collect();
        }
        let Some(nested_role) = self
            .role_metadata
            .as_ref()
            .and_then(|metadata| metadata.nested_role(role_identifier))
        else {
            return Vec::new();
        };
        nested_role
            .allowed_leaf_roles
            .iter()
            .filter_map(|allowed_identifier| {
                active_roles
                    .iter()
                    .find(|role| &role.output_identifier == allowed_identifier)
                    .cloned()
            })
            .collect()
    }

    fn generated_role_roster(&self, manifest: &Manifest) -> Option<String> {
        let role_identifier = manifest.role_identifier();
        let nested = self
            .role_metadata
            .as_ref()
            .and_then(|metadata| metadata.nested_role(&role_identifier))
            .is_some();
        if role_identifier.as_ref() != "manager" && !nested {
            return None;
        }
        let entries = self
            .allowed_child_roles(&role_identifier, manifest.output_surface)
            .into_iter()
            .map(|role| {
                format!(
                    "- `{}` — {}",
                    role.output_identifier.as_ref(),
                    role.role_description.as_ref()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        if role_identifier.as_ref() == "manager" {
            Some(format!(
                "# Module - generated Manager roster\n\n## Manager dispatch roster\n\nThe root Manager may dispatch these target-available roles directly. Use `generalist` when no specialist fits.\n\n{entries}\n"
            ))
        } else {
            Some(format!(
                "# Module - generated nested role roster\n\n## Allowed child-role roster\n\nThis NestedRole may dispatch only these leaf roles on this target.\n\n{entries}\n"
            ))
        }
    }

    fn role_output_inventory(&self) -> RoleOutputInventory {
        RoleOutputInventory::new(
            self.active_roles()
                .iter()
                .flat_map(ActiveRole::output_paths)
                .collect(),
        )
    }

    fn expected_outputs(&self) -> Result<BTreeSet<String>> {
        let mut expected = BTreeSet::new();
        for manifest in self.skill_manifests()? {
            expected.insert(manifest.output_path.as_ref().to_owned());
        }
        for manifest in self.role_manifests()? {
            expected.insert(manifest.output_path.as_ref().to_owned());
        }
        if self.uses_active_manifest() {
            expected.insert(RoleOutputInventory::relative_path().into_payload());
        }
        if let Some(roster) = &self.compatibility_roster {
            for entry_point in roster.entry_points.payload() {
                for output_path in entry_point.extra_paths() {
                    expected.insert(output_path.into_payload());
                }
            }
        }
        Ok(expected)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum GenerationJob {
    Manifest(ManifestAssembler),
    Rendered(RenderedOutput),
}

impl GenerationJob {
    fn generate(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        match self {
            Self::Manifest(assembler) => assembler.generate(mode),
            Self::Rendered(output) => output.write(mode),
        }
    }

    fn output_path(&self) -> Result<WorkspacePath> {
        match self {
            Self::Manifest(assembler) => WorkspacePath::new(
                assembler.workspace_root.clone(),
                PathBuf::from(assembler.manifest.output_path.as_ref()),
            ),
            Self::Rendered(output) => Ok(output.output_path.clone()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OutputPathIndex {
    physical_paths: BTreeMap<PathBuf, String>,
}

impl OutputPathIndex {
    fn new() -> Self {
        Self {
            physical_paths: BTreeMap::new(),
        }
    }

    fn validate(mut self, jobs: &[GenerationJob]) -> Result<()> {
        for job in jobs {
            self.record(job)?;
        }
        Ok(())
    }

    fn record(&mut self, job: &GenerationJob) -> Result<()> {
        let output_path = job.output_path()?;
        let physical_path = output_path.full_path();
        let relative_path = output_path.relative_path().to_string_lossy().into_owned();
        if self
            .physical_paths
            .insert(physical_path.clone(), relative_path.clone())
            .is_some()
        {
            return Err(Error::DuplicateOutputPath {
                relative_path,
                physical_path,
            });
        }
        Ok(())
    }
}

impl ActiveSkill {
    fn first_class_manifests(
        &self,
        module_index: &ModuleIndex,
        universal_full_agent_modules: &UniversalFullAgentModules,
        is_full_agent: bool,
    ) -> Result<Vec<Manifest>> {
        let mut manifests = Vec::new();
        for surface in self.target_surfaces.payload() {
            let output_surface = OutputSurface::from(surface);
            let mut expansion = ModuleExpansion::new(
                module_index,
                if is_full_agent {
                    ModuleUse::FullAgentSkillContent
                } else {
                    ModuleUse::SkillContent
                },
                output_surface,
            );
            expansion.append(&self.module_identifier)?;
            if is_full_agent {
                for module_identifier in universal_full_agent_modules.payload() {
                    expansion.append_without_target_modules(module_identifier)?;
                }
                for module_identifier in universal_full_agent_modules.payload() {
                    for target_module_identifier in
                        module_index.target_modules(module_identifier, output_surface)
                    {
                        expansion.append(&target_module_identifier)?;
                    }
                }
            }
            let modules = Modules::new(expansion.into_modules());
            manifests.push(Manifest {
                output_path: OutputPath::new(
                    output_surface.skill_path(self.output_identifier.as_ref()),
                ),
                output_kind: OutputKind::Markdown,
                output_surface,
                frontmatter: self.frontmatter(),
                optional_skills: OptionalSkills::new(Vec::new()),
                modules: modules.clone(),
            });
        }
        Ok(manifests)
    }

    fn frontmatter(&self) -> Frontmatter {
        SkillMetadata {
            skill_category: self.skill_category,
            skill_tier: self.skill_tier,
            skill_description: self.skill_description.clone(),
        }
        .frontmatter(self.output_identifier.as_ref())
    }
}

impl ActiveRole {
    fn manifests(
        &self,
        module_index: &ModuleIndex,
        universal_full_agent_modules: &UniversalFullAgentModules,
        metadata: &RoleMetadata,
        nested_role: Option<&NestedRoleMetadata>,
        active_roles: &[ActiveRole],
    ) -> Result<Vec<Manifest>> {
        let mut manifests = Vec::new();
        for surface in self.role_target_surfaces.payload() {
            let output_surface = OutputSurface::from(surface);
            let target_profile = TargetModelProfile::strongest(
                metadata.profile.target(output_surface),
                nested_role.and_then(|nested| nested.minimum_model(output_surface)),
            );
            let allowed_child_role_identifiers =
                self.allowed_child_role_identifiers(output_surface, nested_role, active_roles);
            manifests.push(Manifest {
                output_path: OutputPath::new(
                    output_surface.role_path(self.output_identifier.as_ref()),
                ),
                output_kind: output_surface.role_output_kind(),
                output_surface,
                frontmatter: self.frontmatter(
                    output_surface,
                    &target_profile,
                    &metadata.optional_skills,
                    nested_role.is_some(),
                    &allowed_child_role_identifiers,
                ),
                optional_skills: metadata.optional_skills.clone(),
                modules: Modules::new(self.assembled_modules(
                    module_index,
                    universal_full_agent_modules,
                    output_surface,
                )?),
            });
        }
        Ok(manifests)
    }

    fn allowed_child_role_identifiers(
        &self,
        output_surface: OutputSurface,
        nested_role: Option<&NestedRoleMetadata>,
        active_roles: &[ActiveRole],
    ) -> Vec<OutputIdentifier> {
        if self.output_identifier.as_ref() == "manager" {
            return active_roles
                .iter()
                .filter(|role| {
                    role.output_identifier.as_ref() != "manager"
                        && role
                            .role_target_surfaces
                            .payload()
                            .contains(&output_surface.role_target_surface())
                })
                .map(|role| role.output_identifier.clone())
                .collect();
        }
        nested_role
            .map(|nested| nested.allowed_leaf_roles.clone())
            .unwrap_or_default()
    }

    fn assembled_modules(
        &self,
        module_index: &ModuleIndex,
        universal_full_agent_modules: &UniversalFullAgentModules,
        output_surface: OutputSurface,
    ) -> Result<Vec<AssembledModule>> {
        let mut expansion = ModuleExpansion::new(
            module_index,
            ModuleUse::FullAgentRoleContent,
            output_surface,
        );
        expansion.append_role_source(&self.module_identifier)?;
        for module_identifier in universal_full_agent_modules.payload() {
            expansion.append_without_target_modules(module_identifier)?;
        }
        for module_identifier in universal_full_agent_modules.payload() {
            for target_module_identifier in
                module_index.target_modules(module_identifier, output_surface)
            {
                expansion.append(&target_module_identifier)?;
            }
        }
        for module_identifier in self.included_modules.payload() {
            expansion.append(module_identifier)?;
        }
        Ok(expansion.into_modules())
    }

    fn frontmatter(
        &self,
        output_surface: OutputSurface,
        profile: &TargetModelProfile,
        optional_skills: &OptionalSkills,
        is_nested_role: bool,
        allowed_child_role_identifiers: &[OutputIdentifier],
    ) -> Frontmatter {
        let mut entries = vec![
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("name"),
                frontmatter_value: FrontmatterValue::new(self.output_identifier.as_ref()),
            },
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("description"),
                frontmatter_value: FrontmatterValue::new(self.role_description.as_ref()),
            },
        ];
        match output_surface {
            OutputSurface::ClaudeAgent => {
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model"),
                    frontmatter_value: FrontmatterValue::new(&profile.model_identifier),
                });
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("effort"),
                    frontmatter_value: FrontmatterValue::new(profile.effort_level.as_str()),
                });
            }
            OutputSurface::PiAgent => {
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model"),
                    frontmatter_value: FrontmatterValue::new(format!(
                        "{}/{}",
                        profile
                            .pi_provider
                            .as_deref()
                            .expect("Pi target model has a provider"),
                        profile.model_identifier
                    )),
                });
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("thinking"),
                    frontmatter_value: FrontmatterValue::new(profile.effort_level.as_str()),
                });
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("projectRoleIdentity"),
                    frontmatter_value: FrontmatterValue::new(self.output_identifier.as_ref()),
                });
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("projectRoleDispatchKind"),
                    frontmatter_value: FrontmatterValue::new(
                        if self.output_identifier.as_ref() == "manager" {
                            "manager"
                        } else if is_nested_role {
                            "nested"
                        } else {
                            "leaf"
                        },
                    ),
                });
                if is_nested_role {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("allowedChildRoleNames"),
                        frontmatter_value: FrontmatterValue::new(
                            allowed_child_role_identifiers
                                .iter()
                                .map(|identifier| identifier.as_ref())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    });
                }
                if !optional_skills.payload().is_empty() {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("skills"),
                        frontmatter_value: FrontmatterValue::new(
                            optional_skills
                                .payload()
                                .iter()
                                .map(|skill| skill.as_ref())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    });
                }
            }
            OutputSurface::CodexAgent => {
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model"),
                    frontmatter_value: FrontmatterValue::new(&profile.model_identifier),
                });
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model_reasoning_effort"),
                    frontmatter_value: FrontmatterValue::new(profile.effort_level.as_str()),
                });
            }
            _ => unreachable!("role frontmatter renders only for role surfaces"),
        }
        Frontmatter::new(entries)
    }

    fn output_paths(&self) -> Vec<OutputPath> {
        self.role_target_surfaces
            .payload()
            .iter()
            .map(|surface| {
                let output_surface = OutputSurface::from(surface);
                OutputPath::new(output_surface.role_path(self.output_identifier.as_ref()))
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModuleIndex {
    entries: BTreeMap<ModuleIdentifier, ModuleDependency>,
    target_module_insertions: TargetModuleInsertions,
}

impl ModuleIndex {
    fn new(
        module_dependencies: ModuleDependencies,
        target_module_insertions: TargetModuleInsertions,
    ) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for dependency in module_dependencies.into_payload() {
            let module_identifier = dependency.module_identifier.clone();
            if entries
                .insert(module_identifier.clone(), dependency)
                .is_some()
            {
                return Err(Error::DuplicateModule {
                    module_identifier: module_identifier.into_payload(),
                });
            }
        }
        Ok(Self {
            entries,
            target_module_insertions,
        })
    }

    fn validate_universal_full_agent_modules(
        &self,
        universal_full_agent_modules: &UniversalFullAgentModules,
    ) -> Result<()> {
        for module_identifier in universal_full_agent_modules.payload() {
            self.dependency(module_identifier)?
                .require_kind(ModuleKind::SharedComposition, "SharedComposition")?;
        }
        Ok(())
    }

    fn dependency(&self, module_identifier: &ModuleIdentifier) -> Result<&ModuleDependency> {
        self.entries
            .get(module_identifier)
            .ok_or_else(|| Error::MissingModule {
                module_identifier: module_identifier.as_ref().to_owned(),
            })
    }

    fn target_modules(
        &self,
        module_identifier: &ModuleIdentifier,
        output_surface: OutputSurface,
    ) -> Vec<ModuleIdentifier> {
        self.target_module_insertions
            .payload()
            .iter()
            .filter(|insertion| insertion.applies_to(module_identifier, output_surface))
            .flat_map(|insertion| insertion.included_modules.payload().iter().cloned())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleUse {
    SkillContent,
    FullAgentSkillContent,
    FullAgentRoleContent,
}

impl ModuleUse {
    fn expected_description(&self) -> &'static str {
        match self {
            Self::SkillContent => "RuntimeSkill",
            Self::FullAgentSkillContent => "RuntimeSkill or SharedComposition",
            Self::FullAgentRoleContent => "RuntimeSkill, RoleComposition, or SharedComposition",
        }
    }

    fn accepts(&self, module_kind: ModuleKind) -> bool {
        match self {
            Self::SkillContent => module_kind == ModuleKind::RuntimeSkill,
            Self::FullAgentSkillContent => matches!(
                module_kind,
                ModuleKind::RuntimeSkill | ModuleKind::SharedComposition
            ),
            Self::FullAgentRoleContent => matches!(
                module_kind,
                ModuleKind::RuntimeSkill
                    | ModuleKind::RoleComposition
                    | ModuleKind::SharedComposition
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ModuleExpansion<'a> {
    module_index: &'a ModuleIndex,
    module_use: ModuleUse,
    output_surface: OutputSurface,
    resolved: BTreeSet<ModuleIdentifier>,
    visiting: Vec<ModuleIdentifier>,
    modules: Vec<AssembledModule>,
}

impl<'a> ModuleExpansion<'a> {
    fn new(
        module_index: &'a ModuleIndex,
        module_use: ModuleUse,
        output_surface: OutputSurface,
    ) -> Self {
        Self {
            module_index,
            module_use,
            output_surface,
            resolved: BTreeSet::new(),
            visiting: Vec::new(),
            modules: Vec::new(),
        }
    }

    fn append_role_source(&mut self, module_identifier: &ModuleIdentifier) -> Result<()> {
        if self.resolved.contains(module_identifier) {
            return Ok(());
        }
        let dependency = self.module_index.dependency(module_identifier)?;
        dependency.require_kind(ModuleKind::RoleSource, "RoleSource")?;
        self.resolved.insert(module_identifier.clone());
        self.modules.push(dependency.assembled_module());
        Ok(())
    }

    fn append(&mut self, module_identifier: &ModuleIdentifier) -> Result<()> {
        self.append_with_target_modules(module_identifier, true)
    }

    fn append_without_target_modules(
        &mut self,
        module_identifier: &ModuleIdentifier,
    ) -> Result<()> {
        self.append_with_target_modules(module_identifier, false)
    }

    fn append_with_target_modules(
        &mut self,
        module_identifier: &ModuleIdentifier,
        include_target_modules: bool,
    ) -> Result<()> {
        if self.resolved.contains(module_identifier) {
            return Ok(());
        }
        if let Some(position) = self
            .visiting
            .iter()
            .position(|visiting_identifier| visiting_identifier == module_identifier)
        {
            let mut module_identifiers: Vec<String> = self.visiting[position..]
                .iter()
                .map(|identifier| identifier.as_ref().to_owned())
                .collect();
            module_identifiers.push(module_identifier.as_ref().to_owned());
            return Err(Error::ModuleDependencyCycle { module_identifiers });
        }
        let dependency = self.module_index.dependency(module_identifier)?;
        dependency.require_accepted(self.module_use)?;
        self.visiting.push(module_identifier.clone());
        for dependency_identifier in dependency.dependency_modules.payload() {
            self.append_with_target_modules(dependency_identifier, include_target_modules)?;
        }
        self.visiting.pop();
        self.resolved.insert(module_identifier.clone());
        self.modules.push(dependency.assembled_module());
        if include_target_modules {
            for target_module_identifier in self
                .module_index
                .target_modules(module_identifier, self.output_surface)
            {
                self.append(&target_module_identifier)?;
            }
        }
        Ok(())
    }

    fn into_modules(self) -> Vec<AssembledModule> {
        self.modules
    }
}

impl TargetModuleInsertion {
    fn applies_to(
        &self,
        module_identifier: &ModuleIdentifier,
        output_surface: OutputSurface,
    ) -> bool {
        &self.module_identifier == module_identifier && self.output_surface == output_surface
    }
}

impl ModuleDependency {
    fn assembled_module(&self) -> AssembledModule {
        AssembledModule {
            module_identifier: self.module_identifier.clone(),
            module_path: self.module_path.clone(),
            module_kind: self.module_kind,
        }
    }

    fn require_accepted(&self, module_use: ModuleUse) -> Result<()> {
        if module_use.accepts(self.module_kind) {
            Ok(())
        } else {
            Err(Error::InvalidModuleKind {
                module_identifier: self.module_identifier.as_ref().to_owned(),
                expected: module_use.expected_description().to_owned(),
                actual: self.module_kind.name().to_owned(),
            })
        }
    }

    fn require_kind(&self, expected_kind: ModuleKind, expected: &str) -> Result<()> {
        if self.module_kind == expected_kind {
            Ok(())
        } else {
            Err(Error::InvalidModuleKind {
                module_identifier: self.module_identifier.as_ref().to_owned(),
                expected: expected.to_owned(),
                actual: self.module_kind.name().to_owned(),
            })
        }
    }
}

impl ModuleKind {
    fn name(&self) -> &'static str {
        match self {
            Self::RuntimeSkill => "RuntimeSkill",
            Self::RoleSource => "RoleSource",
            Self::RoleComposition => "RoleComposition",
            Self::SharedComposition => "SharedComposition",
        }
    }

    fn is_composition(&self) -> bool {
        matches!(self, Self::RoleComposition | Self::SharedComposition)
    }
}

impl SkillRoster {
    fn active_outputs(&self) -> ActiveOutputs {
        ActiveOutputs::new(
            self.skill_modules
                .payload()
                .iter()
                .filter_map(SkillModule::active_output)
                .collect(),
        )
    }

    fn module_dependencies(&self) -> ModuleDependencies {
        ModuleDependencies::new(
            self.skill_modules
                .payload()
                .iter()
                .map(SkillModule::module_dependency)
                .collect(),
        )
    }
}

impl SkillModule {
    fn active_output(&self) -> Option<ActiveOutput> {
        let ModuleLifecycle::Active(metadata) = &self.module_lifecycle else {
            return None;
        };
        let active_output = match self.emission_policy {
            crate::schema::assembly::EmissionPolicy::FirstClassSkill => ActiveOutput::Skill,
            crate::schema::assembly::EmissionPolicy::FullAgentSkill => ActiveOutput::FullAgentSkill,
            crate::schema::assembly::EmissionPolicy::InternalOnly
            | crate::schema::assembly::EmissionPolicy::NoEmission => return None,
        };
        Some(active_output(ActiveSkill {
            output_identifier: crate::schema::assembly::OutputIdentifier::new(
                self.module_name.as_ref(),
            ),
            module_identifier: ModuleIdentifier::new(self.module_name.as_ref()),
            skill_category: metadata.skill_category,
            skill_tier: metadata.skill_tier,
            skill_description: metadata.skill_description.clone(),
            target_surfaces: self.target_surfaces.clone(),
        }))
    }

    fn module_dependency(&self) -> ModuleDependency {
        ModuleDependency {
            module_identifier: ModuleIdentifier::new(self.module_name.as_ref()),
            module_path: self.module_path.clone(),
            dependency_modules: crate::schema::assembly::DependencyModules::new(Vec::new()),
            module_kind: ModuleKind::RuntimeSkill,
        }
    }

    fn first_class_paths(&self) -> Vec<OutputPath> {
        [OutputSurface::AgentsSkill, OutputSurface::ClaudeSkill]
            .into_iter()
            .map(|surface| OutputPath::new(surface.skill_path(self.module_name.payload())))
            .collect()
    }
}

impl From<&TargetSurface> for OutputSurface {
    fn from(surface: &TargetSurface) -> Self {
        match surface {
            TargetSurface::AgentsSkill => Self::AgentsSkill,
            TargetSurface::ClaudeSkill => Self::ClaudeSkill,
        }
    }
}

impl From<&RoleTargetSurface> for OutputSurface {
    fn from(surface: &RoleTargetSurface) -> Self {
        match surface {
            RoleTargetSurface::ClaudeAgent => Self::ClaudeAgent,
            RoleTargetSurface::CodexAgent => Self::CodexAgent,
            RoleTargetSurface::PiAgent => Self::PiAgent,
        }
    }
}

impl SkillMetadata {
    fn frontmatter(&self, module_name: &str) -> Frontmatter {
        Frontmatter::new(vec![
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("name"),
                frontmatter_value: FrontmatterValue::new(module_name),
            },
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("description"),
                frontmatter_value: FrontmatterValue::new(self.skill_description.as_ref()),
            },
        ])
    }
}

impl EntryPoint {
    fn extra_manifests(&self) -> Result<Vec<Manifest>> {
        let mut manifests = Vec::new();
        for extra in self.entry_point_extras.payload() {
            manifests.push(extra.manifest(self.module_name.payload())?);
        }
        Ok(manifests)
    }

    fn extra_paths(&self) -> Vec<OutputPath> {
        self.entry_point_extras
            .payload()
            .iter()
            .map(|extra| extra.output_path(self.module_name.payload()))
            .collect()
    }
}

impl EntryPointExtra {
    fn manifest(&self, module_name: &str) -> Result<Manifest> {
        let output_surface = OutputSurface::from(&self.extra_surface);
        Ok(Manifest {
            output_path: self.output_path(module_name),
            output_kind: OutputKind::Markdown,
            output_surface,
            frontmatter: Frontmatter::new(Vec::new()),
            optional_skills: OptionalSkills::new(Vec::new()),
            modules: Modules::new(vec![AssembledModule {
                module_identifier: ModuleIdentifier::new(module_name),
                module_path: ModulePath::new(self.extra_module_path.as_ref()),
                module_kind: ModuleKind::RuntimeSkill,
            }]),
        })
    }

    fn output_path(&self, module_name: &str) -> OutputPath {
        OutputPath::new(OutputSurface::from(&self.extra_surface).extra_path(module_name))
    }
}

impl From<&ExtraSurface> for OutputSurface {
    fn from(surface: &ExtraSurface) -> Self {
        match surface {
            ExtraSurface::ClaudeCommand => Self::ClaudeCommand,
            ExtraSurface::CodexPrompt => Self::CodexPrompt,
            ExtraSurface::CodexCommand => Self::CodexCommand,
        }
    }
}

impl OutputSurface {
    fn skill_path(&self, module_name: &str) -> String {
        match self {
            Self::AgentsSkill => format!(".agents/skills/{module_name}/SKILL.md"),
            Self::ClaudeSkill => format!(".claude/skills/{module_name}/SKILL.md"),
            Self::Workspace
            | Self::ClaudeCommand
            | Self::CodexPrompt
            | Self::CodexCommand
            | Self::ClaudeAgent
            | Self::CodexAgent
            | Self::PiAgent => unreachable!("not a first-class skill surface"),
        }
    }

    fn extra_path(&self, module_name: &str) -> String {
        match self {
            Self::ClaudeCommand => format!(".claude/commands/{module_name}.md"),
            Self::CodexPrompt => format!(".codex/prompts/{module_name}.md"),
            Self::CodexCommand => format!(".codex/commands/{module_name}.md"),
            Self::Workspace
            | Self::AgentsSkill
            | Self::ClaudeSkill
            | Self::ClaudeAgent
            | Self::CodexAgent
            | Self::PiAgent => unreachable!("not an entrypoint extra surface"),
        }
    }

    fn role_path(&self, role_name: &str) -> String {
        match self {
            Self::ClaudeAgent => format!(".claude/agents/{role_name}.md"),
            Self::CodexAgent => format!(".codex/agents/{role_name}.toml"),
            Self::PiAgent => format!(".pi/agents/{role_name}.md"),
            Self::Workspace
            | Self::AgentsSkill
            | Self::ClaudeSkill
            | Self::ClaudeCommand
            | Self::CodexPrompt
            | Self::CodexCommand => unreachable!("not a role target surface"),
        }
    }

    fn role_output_kind(&self) -> OutputKind {
        match self {
            Self::CodexAgent => OutputKind::Toml,
            Self::ClaudeAgent | Self::PiAgent => OutputKind::Markdown,
            Self::Workspace
            | Self::AgentsSkill
            | Self::ClaudeSkill
            | Self::ClaudeCommand
            | Self::CodexPrompt
            | Self::CodexCommand => unreachable!("not a role target surface"),
        }
    }

    fn role_target_surface(&self) -> RoleTargetSurface {
        match self {
            Self::ClaudeAgent => RoleTargetSurface::ClaudeAgent,
            Self::CodexAgent => RoleTargetSurface::CodexAgent,
            Self::PiAgent => RoleTargetSurface::PiAgent,
            Self::Workspace
            | Self::AgentsSkill
            | Self::ClaudeSkill
            | Self::ClaudeCommand
            | Self::CodexPrompt
            | Self::CodexCommand => unreachable!("not a role target surface"),
        }
    }

    fn is_skill(&self) -> bool {
        matches!(self, Self::AgentsSkill | Self::ClaudeSkill)
    }

    fn is_role(&self) -> bool {
        matches!(self, Self::ClaudeAgent | Self::CodexAgent | Self::PiAgent)
    }
}

impl Manifest {
    fn role_identifier(&self) -> OutputIdentifier {
        self.frontmatter
            .payload()
            .iter()
            .find(|entry| entry.frontmatter_key.as_ref() == "name")
            .map(|entry| OutputIdentifier::new(entry.frontmatter_value.as_ref()))
            .expect("generated role manifest has a name")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SkillName {
    value: String,
}

impl SkillName {
    fn from_frontmatter(frontmatter: &Frontmatter) -> Self {
        let value = frontmatter
            .payload()
            .iter()
            .find(|entry| entry.frontmatter_key.as_ref() == "name")
            .map(|entry| entry.frontmatter_value.as_ref().to_owned())
            .unwrap_or_else(|| "<missing>".to_owned());
        Self { value }
    }

    fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SerializedSkillBlock<'a> {
    skill_name: SkillName,
    location: String,
    body: &'a str,
}

impl<'a> SerializedSkillBlock<'a> {
    fn new(skill_name: SkillName, location: String, body: &'a str) -> Self {
        Self {
            skill_name,
            location,
            body,
        }
    }

    fn validate_size(&self) -> Result<()> {
        let byte_count = self.byte_count();
        if byte_count > GENERATED_SKILL_BLOCK_BYTE_LIMIT {
            return Err(Error::GeneratedSkillBlockTooLarge {
                skill_name: self.skill_name.as_str().to_owned(),
                location: self.location.clone(),
                byte_count,
                limit: GENERATED_SKILL_BLOCK_BYTE_LIMIT,
            });
        }
        Ok(())
    }

    fn byte_count(&self) -> usize {
        format!(
            "<skill name=\"{}\" location=\"{}\">\n{}\n</skill>",
            self.skill_name.as_str(),
            self.location,
            self.body
        )
        .len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetiredCurrentDestinationProse<'a> {
    output_path: WorkspacePath,
    body: &'a str,
}

impl<'a> RetiredCurrentDestinationProse<'a> {
    fn new(output_path: WorkspacePath, body: &'a str) -> Self {
        Self { output_path, body }
    }

    fn validate(&self) -> Result<()> {
        for phrase in RETIRED_CURRENT_DESTINATION_PHRASES {
            if self.body.contains(phrase) {
                return Err(Error::RetiredCurrentDestinationProse {
                    path: self.output_path.full_path(),
                    phrase: (*phrase).to_owned(),
                });
            }
        }
        Ok(())
    }
}

fn validate_agent_execution_limits(output_path: &WorkspacePath, packet: &str) -> Result<()> {
    for line in packet.lines() {
        let Some(field_name) = configured_field_name(line) else {
            continue;
        };
        if is_execution_limit_field(field_name) {
            return Err(Error::GeneratedAgentExecutionLimit {
                path: output_path.full_path(),
                field_name: field_name.to_owned(),
            });
        }
    }
    Ok(())
}

fn configured_field_name(line: &str) -> Option<&str> {
    let line = line.trim();
    let delimiter = line.find([':', '='])?;
    let (field_name, value) = line.split_at(delimiter);
    let field_name = field_name.trim();
    let value = value[1..].trim();
    (!field_name.is_empty()
        && !value.is_empty()
        && field_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
    .then_some(field_name)
}

fn is_execution_limit_field(field_name: &str) -> bool {
    let normalized: String = field_name
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect();
    EXECUTION_LIMIT_FIELD_PARTS
        .iter()
        .any(|part| normalized.contains(part))
        || (normalized.starts_with("max")
            && EXECUTION_LIMIT_MAXIMUMS
                .iter()
                .any(|maximum| normalized.contains(maximum)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestAssembler {
    source_root: PathBuf,
    workspace_root: PathBuf,
    manifest: Manifest,
    generated_fragments: Vec<String>,
}

impl ManifestAssembler {
    fn new(source_root: PathBuf, workspace_root: PathBuf, manifest: Manifest) -> Self {
        Self {
            source_root,
            workspace_root,
            manifest,
            generated_fragments: Vec::new(),
        }
    }

    fn with_generated_fragment(mut self, fragment: String) -> Self {
        self.generated_fragments.push(fragment);
        self
    }

    fn generate(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        let output_path = WorkspacePath::new(
            self.workspace_root.clone(),
            PathBuf::from(self.manifest.output_path.as_ref()),
        )?;
        let rendered = self.render(&output_path)?;
        RenderedOutput::new(
            self.workspace_root.clone(),
            self.manifest.output_path.clone(),
            rendered,
        )?
        .write(mode)
    }

    fn render(&self, output_path: &WorkspacePath) -> Result<String> {
        match self.manifest.output_kind {
            OutputKind::Markdown => self.render_markdown(output_path),
            OutputKind::Toml => self.render_toml(output_path),
            OutputKind::Nota => {
                unreachable!("derived NOTA outputs render without module manifests")
            }
        }
    }

    fn render_markdown(&self, output_path: &WorkspacePath) -> Result<String> {
        let rendered = MarkdownAssembly::new(
            output_path.clone(),
            self.manifest.output_surface,
            self.manifest.frontmatter.payload().to_vec(),
            self.markdown_fragments()?,
        )
        .render()?;
        if self.manifest.output_surface.is_skill() {
            SerializedSkillBlock::new(
                SkillName::from_frontmatter(&self.manifest.frontmatter),
                output_path.relative_path().to_string_lossy().into_owned(),
                &rendered,
            )
            .validate_size()?;
        }
        if self.manifest.output_surface.is_role() {
            RetiredCurrentDestinationProse::new(output_path.clone(), &rendered).validate()?;
            validate_agent_execution_limits(output_path, &rendered)?;
        }
        Ok(rendered)
    }

    fn render_toml(&self, output_path: &WorkspacePath) -> Result<String> {
        let body = MarkdownAssembly::new(
            output_path.clone(),
            self.manifest.output_surface,
            Vec::new(),
            self.markdown_fragments()?,
        )
        .render()?;
        let developer_instructions = body;
        if self.manifest.output_surface.is_role() {
            RetiredCurrentDestinationProse::new(output_path.clone(), &developer_instructions)
                .validate()?;
        }
        let rendered =
            RoleToml::new(&self.manifest.frontmatter, developer_instructions).render()?;
        if self.manifest.output_surface.is_role() {
            validate_agent_execution_limits(output_path, &rendered)?;
        }
        Ok(rendered)
    }

    fn markdown_fragments(&self) -> Result<Vec<MarkdownFragment>> {
        let mut fragments = Vec::new();
        for module in self.manifest.modules.payload() {
            let path = self.module_workspace_path(module)?;
            let fragment = if module.module_kind.is_composition() {
                MarkdownFragment::read_composition(path, &module.module_identifier)?
            } else {
                MarkdownFragment::read(path)?
            };
            fragments.push(fragment);
        }
        for text in &self.generated_fragments {
            fragments.push(MarkdownFragment::from_text(
                WorkspacePath::new(
                    self.source_root.clone(),
                    PathBuf::from("manifests/active-outputs.nota"),
                )?,
                text,
            ));
        }
        if !self.manifest.optional_skills.payload().is_empty() {
            let list = self
                .manifest
                .optional_skills
                .payload()
                .iter()
                .map(|skill| format!("- `{}`", skill.as_ref()))
                .collect::<Vec<_>>()
                .join("\n");
            let text = format!(
                "# Module - optional skills\n\nThese skills are available to load when needed and are not preloaded. Load only entries listed here:\n\n{list}\n"
            );
            fragments.push(MarkdownFragment::from_text(
                WorkspacePath::new(
                    self.source_root.clone(),
                    PathBuf::from("manifests/role-optional-skills.nota"),
                )?,
                text,
            ));
        }
        Ok(fragments)
    }

    fn module_workspace_path(&self, module: &AssembledModule) -> Result<WorkspacePath> {
        WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(module.module_path.as_ref()),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleToml {
    name: String,
    description: String,
    model: String,
    model_reasoning_effort: String,
    developer_instructions: String,
}

impl RoleToml {
    fn new(frontmatter: &Frontmatter, developer_instructions: String) -> Self {
        let mut name = String::new();
        let mut description = String::new();
        let mut model = String::new();
        let mut model_reasoning_effort = String::new();
        for entry in frontmatter.payload() {
            match entry.frontmatter_key.as_ref() {
                "name" => name = entry.frontmatter_value.as_ref().to_owned(),
                "description" => description = entry.frontmatter_value.as_ref().to_owned(),
                "model" => model = entry.frontmatter_value.as_ref().to_owned(),
                "model_reasoning_effort" => {
                    model_reasoning_effort = entry.frontmatter_value.as_ref().to_owned()
                }
                _ => {}
            }
        }
        Self {
            name,
            description,
            model,
            model_reasoning_effort,
            developer_instructions,
        }
    }

    fn render(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("name = ");
        output.push_str(&TomlString::new(&self.name).render());
        output.push('\n');
        output.push_str("description = ");
        output.push_str(&TomlString::new(&self.description).render());
        output.push('\n');
        output.push_str("model = ");
        output.push_str(&TomlString::new(&self.model).render());
        output.push('\n');
        output.push_str("model_reasoning_effort = ");
        output.push_str(&TomlString::new(&self.model_reasoning_effort).render());
        output.push('\n');
        output.push_str("developer_instructions = ");
        output.push_str(&TomlString::new(&self.developer_instructions).render());
        output.push('\n');
        Ok(output)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TomlString<'a> {
    text: &'a str,
}

impl<'a> TomlString<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn render(&self) -> String {
        let mut output = String::from("\"");
        for character in self.text.chars() {
            match character {
                '\\' => output.push_str("\\\\"),
                '"' => output.push_str("\\\""),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                character => output.push(character),
            }
        }
        output.push('"');
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedOutput {
    output_path: WorkspacePath,
    rendered: String,
}

impl RenderedOutput {
    fn new(workspace_root: PathBuf, output_path: OutputPath, rendered: String) -> Result<Self> {
        Ok(Self {
            output_path: WorkspacePath::new(workspace_root, PathBuf::from(output_path.as_ref()))?,
            rendered,
        })
    }

    fn write(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        match mode {
            GenerationMode::Write => self.write_file()?,
            GenerationMode::Check => self.check_file()?,
        }
        Ok(GeneratedFile {
            output_path: OutputPath::new(
                self.output_path
                    .relative_path()
                    .to_string_lossy()
                    .into_owned(),
            ),
            byte_count: ByteCount::new(self.rendered.len() as u64),
        })
    }

    fn write_file(&self) -> Result<()> {
        let path = self.output_path.full_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&path, &self.rendered).map_err(|source| Error::WriteFile { path, source })
    }

    fn check_file(&self) -> Result<()> {
        let path = self.output_path.full_path();
        let current = fs::read_to_string(&path).map_err(|source| Error::ReadFile {
            path: path.clone(),
            source,
        })?;
        if current == self.rendered {
            Ok(())
        } else {
            Err(Error::StaleOutput { path })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleOutputInventory {
    output_paths: Vec<OutputPath>,
}

impl RoleOutputInventory {
    fn new(output_paths: Vec<OutputPath>) -> Self {
        Self { output_paths }
    }

    fn relative_path() -> OutputPath {
        OutputPath::new("skills/generated-role-outputs.nota")
    }

    fn render(&self) -> String {
        GeneratedRoleOutputs::new(self.output_paths.clone()).to_nota()
    }

    fn contains(&self, output_path: &OutputPath) -> bool {
        self.output_paths.contains(output_path)
    }

    fn remove_stale(&self, active: &Self, pruner: &WorkspacePruner) -> Result<()> {
        for output_path in &self.output_paths {
            if !active.contains(output_path) {
                pruner.remove_relative_path(output_path.as_ref())?;
            }
        }
        Ok(())
    }

    fn validate_no_stale_paths(&self, active: &Self, scan: &StaleOutputScan) -> Result<()> {
        for output_path in &self.output_paths {
            if !active.contains(output_path) {
                scan.require_absent(output_path.as_ref())?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleOutputInventoryFile {
    workspace_root: PathBuf,
}

impl RoleOutputInventoryFile {
    fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn read(&self) -> Result<RoleOutputInventory> {
        let workspace_path = WorkspacePath::new(
            self.workspace_root.clone(),
            PathBuf::from(RoleOutputInventory::relative_path().as_ref()),
        )?;
        let path = workspace_path.full_path();
        match fs::read_to_string(&path) {
            Ok(text) => {
                let generated_outputs = NotaSource::new(&text)
                    .parse::<GeneratedRoleOutputs>()
                    .map_err(|source| Error::DecodeNota { path, source })?;
                Ok(RoleOutputInventory::new(generated_outputs.into_payload()))
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                Ok(RoleOutputInventory::new(Vec::new()))
            }
            Err(source) => Err(Error::ReadFile { path, source }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspacePruner {
    workspace_root: PathBuf,
    configuration: GenerationConfiguration,
}

impl WorkspacePruner {
    fn new(workspace_root: PathBuf, configuration: GenerationConfiguration) -> Self {
        Self {
            workspace_root,
            configuration,
        }
    }

    fn prune(&self) -> Result<()> {
        self.remove_relative_path(".agents/skills")?;
        self.remove_relative_path(".claude/skills")?;
        if let Some(roster) = self.configuration.compatibility_roster() {
            for entry_point in roster.entry_points.payload() {
                for output_path in entry_point.extra_paths() {
                    self.remove_relative_path(output_path.as_ref())?;
                }
            }
        }
        RoleOutputInventoryFile::new(self.workspace_root.clone())
            .read()?
            .remove_stale(&self.configuration.role_output_inventory(), self)?;
        Ok(())
    }

    fn remove_relative_path(&self, relative_path: &str) -> Result<()> {
        let workspace_path =
            WorkspacePath::new(self.workspace_root.clone(), PathBuf::from(relative_path))?;
        let full_path = workspace_path.full_path();
        match fs::symlink_metadata(&full_path) {
            Ok(metadata) if metadata.is_dir() => {
                fs::remove_dir_all(&full_path).map_err(|source| Error::RemovePath {
                    path: full_path,
                    source,
                })?;
            }
            Ok(_) => {
                fs::remove_file(&full_path).map_err(|source| Error::RemovePath {
                    path: full_path,
                    source,
                })?;
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(Error::ReadFile {
                    path: full_path,
                    source,
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StaleOutputScan {
    workspace_root: PathBuf,
    configuration: GenerationConfiguration,
}

impl StaleOutputScan {
    fn new(workspace_root: PathBuf, configuration: GenerationConfiguration) -> Self {
        Self {
            workspace_root,
            configuration,
        }
    }

    fn validate(&self) -> Result<()> {
        let expected = self.configuration.expected_outputs()?;
        self.require_no_unexpected_skill_files(".agents/skills", &expected)?;
        self.require_no_unexpected_skill_files(".claude/skills", &expected)?;
        if let Some(roster) = self.configuration.compatibility_roster() {
            for module in roster.skill_modules.payload() {
                for output_path in module.first_class_paths() {
                    if !expected.contains(output_path.as_ref()) {
                        self.require_absent(output_path.as_ref())?;
                    }
                }
            }
        }
        RoleOutputInventoryFile::new(self.workspace_root.clone())
            .read()?
            .validate_no_stale_paths(&self.configuration.role_output_inventory(), self)
    }

    fn require_absent(&self, relative_path: &str) -> Result<()> {
        let workspace_path =
            WorkspacePath::new(self.workspace_root.clone(), PathBuf::from(relative_path))?;
        let full_path = workspace_path.full_path();
        match fs::symlink_metadata(&full_path) {
            Ok(_) => Err(Error::StaleGeneratedOutput { path: full_path }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(Error::ReadFile {
                path: full_path,
                source,
            }),
        }
    }

    fn require_no_unexpected_skill_files(
        &self,
        relative_directory: &str,
        expected: &BTreeSet<String>,
    ) -> Result<()> {
        let workspace_path = WorkspacePath::new(
            self.workspace_root.clone(),
            PathBuf::from(relative_directory),
        )?;
        let full_path = workspace_path.full_path();
        match fs::read_dir(&full_path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.map_err(|source| Error::ReadFile {
                        path: full_path.clone(),
                        source,
                    })?;
                    let skill_path = entry.path().join("SKILL.md");
                    if skill_path.exists() {
                        let relative_path = skill_path
                            .strip_prefix(&self.workspace_root)
                            .expect("skill path comes from workspace root")
                            .to_string_lossy()
                            .into_owned();
                        if !expected.contains(&relative_path) {
                            self.require_absent(&relative_path)?;
                        }
                    }
                }
                Ok(())
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(Error::ReadFile {
                path: full_path,
                source,
            }),
        }
    }
}

impl GenerationReport {
    pub fn to_nota_text(&self) -> String {
        GenerationOutcome::Generated(self.clone()).to_nota()
    }
}
