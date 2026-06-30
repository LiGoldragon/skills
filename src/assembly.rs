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
        ActiveOutput, ActiveOutputs, ActiveRole, ActiveSkill, ByteCount, EntryPoint,
        EntryPointExtra, ExtraSurface, Frontmatter, FrontmatterEntry, FrontmatterKey,
        FrontmatterValue, GeneratedFile, GeneratedFiles, GeneratedRoleOutputs, GenerationMode,
        GenerationOutcome, GenerationReport, GenerationRequest, Manifest, ManifestPath,
        ModuleDependencies, ModuleDependency, ModuleIdentifier, ModuleKind, ModuleLifecycle,
        ModulePath, Modules, Operation, OutputKind, OutputPath, OutputSurface, RoleTargetSurface,
        SkillMetadata, SkillModule, SkillRoster, TargetSurface,
    },
    workspace_path::WorkspacePath,
};

const CODEX_SKILL_READ_DEDUPLICATION_INSTRUCTION: &str = "Skill-read de-duplication: A pasted <skill ...>...</skill> block is complete when it has matching opening and closing <skill> tags, a skill name, a location, and non-empty body text. Treat a complete pasted skill block as already loaded for this session. Read the same skill location again only when the block is structurally missing content, the user asks to verify source or freshness, or a higher-priority instruction explicitly requires verification.";
const GENERATED_SKILL_BLOCK_BYTE_LIMIT: usize = 32 * 1024;
const RETIRED_CURRENT_DESTINATION_PHRASES: &[&str] =
    &["Repo Operator", "Weave Operator", "Intent Maintainer"];

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
        let request = RequestArgument::new(self.arguments.clone())
            .read()?
            .parse()?;
        request.execute()
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
    pub fn execute(self) -> Result<GenerationOutcome> {
        match self {
            Self::Generate(request) => request.generate().map(GenerationOutcome::Generated),
        }
    }
}

impl GenerationRequest {
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
        Ok(GenerationConfiguration::active(
            active_outputs,
            module_dependencies,
        ))
    }

    fn read_legacy_roster(&self) -> Result<GenerationConfiguration> {
        let roster: SkillRoster = SourceFile::new(
            self.source_root.clone(),
            PathBuf::from(self.manifest_path.as_ref()),
        )
        .read()?;
        Ok(GenerationConfiguration::legacy(roster))
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
            jobs.push(GenerationJob::Manifest(ManifestAssembler::new(
                self.source_root.clone(),
                self.workspace_root.clone(),
                manifest,
            )));
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
struct GenerationConfiguration {
    active_outputs: ActiveOutputs,
    module_dependencies: ModuleDependencies,
    compatibility_roster: Option<SkillRoster>,
}

impl GenerationConfiguration {
    fn active(active_outputs: ActiveOutputs, module_dependencies: ModuleDependencies) -> Self {
        Self {
            active_outputs,
            module_dependencies,
            compatibility_roster: None,
        }
    }

    fn legacy(roster: SkillRoster) -> Self {
        Self {
            active_outputs: roster.active_outputs(),
            module_dependencies: roster.module_dependencies(),
            compatibility_roster: Some(roster),
        }
    }

    fn uses_active_manifest(&self) -> bool {
        self.compatibility_roster.is_none()
    }

    fn compatibility_roster(&self) -> Option<&SkillRoster> {
        self.compatibility_roster.as_ref()
    }

    fn active_skills(&self) -> Vec<ActiveSkill> {
        self.active_outputs
            .payload()
            .iter()
            .filter_map(|output| match output {
                ActiveOutput::Skill(skill) => Some(skill.clone()),
                ActiveOutput::Role(_) => None,
            })
            .collect()
    }

    fn active_roles(&self) -> Vec<ActiveRole> {
        self.active_outputs
            .payload()
            .iter()
            .filter_map(|output| match output {
                ActiveOutput::Role(role) => Some(role.clone()),
                ActiveOutput::Skill(_) => None,
            })
            .collect()
    }

    fn skill_manifests(&self) -> Result<Vec<Manifest>> {
        let module_index = ModuleIndex::new(self.module_dependencies.clone())?;
        let mut manifests = Vec::new();
        for skill in self.active_skills() {
            for manifest in skill.first_class_manifests(&module_index)? {
                manifests.push(manifest);
            }
        }
        Ok(manifests)
    }

    fn role_manifests(&self) -> Result<Vec<Manifest>> {
        let module_index = ModuleIndex::new(self.module_dependencies.clone())?;
        let mut manifests = Vec::new();
        for role in self.active_roles() {
            for manifest in role.manifests(&module_index)? {
                manifests.push(manifest);
            }
        }
        Ok(manifests)
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
    fn first_class_manifests(&self, module_index: &ModuleIndex) -> Result<Vec<Manifest>> {
        let mut manifests = Vec::new();
        let modules = Modules::new(module_index.expanded_paths(
            std::slice::from_ref(&self.module_identifier),
            ModuleUse::SkillContent,
        )?);
        for surface in self.target_surfaces.payload() {
            let output_surface = OutputSurface::from(surface);
            manifests.push(Manifest {
                output_path: OutputPath::new(
                    output_surface.skill_path(self.output_identifier.as_ref()),
                ),
                output_kind: OutputKind::Markdown,
                output_surface,
                frontmatter: self.frontmatter(),
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
    fn manifests(&self, module_index: &ModuleIndex) -> Result<Vec<Manifest>> {
        let mut manifests = Vec::new();
        for surface in self.role_target_surfaces.payload() {
            let output_surface = OutputSurface::from(surface);
            manifests.push(Manifest {
                output_path: OutputPath::new(
                    output_surface.role_path(self.output_identifier.as_ref()),
                ),
                output_kind: output_surface.role_output_kind(),
                output_surface,
                frontmatter: self.frontmatter(),
                modules: Modules::new(self.assembled_modules(module_index)?),
            });
        }
        Ok(manifests)
    }

    fn assembled_modules(&self, module_index: &ModuleIndex) -> Result<Vec<ModulePath>> {
        let mut expansion = ModuleExpansion::new(module_index, ModuleUse::RoleContent);
        expansion.append_role_source(&self.module_identifier)?;
        for module_identifier in self.included_modules.payload() {
            expansion.append(module_identifier)?;
        }
        Ok(expansion.into_paths())
    }

    fn frontmatter(&self) -> Frontmatter {
        Frontmatter::new(vec![
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("name"),
                frontmatter_value: FrontmatterValue::new(self.output_identifier.as_ref()),
            },
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("description"),
                frontmatter_value: FrontmatterValue::new(self.role_description.as_ref()),
            },
        ])
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
}

impl ModuleIndex {
    fn new(module_dependencies: ModuleDependencies) -> Result<Self> {
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
        Ok(Self { entries })
    }

    fn expanded_paths(
        &self,
        module_identifiers: &[ModuleIdentifier],
        module_use: ModuleUse,
    ) -> Result<Vec<ModulePath>> {
        let mut expansion = ModuleExpansion::new(self, module_use);
        for module_identifier in module_identifiers {
            expansion.append(module_identifier)?;
        }
        Ok(expansion.into_paths())
    }

    fn dependency(&self, module_identifier: &ModuleIdentifier) -> Result<&ModuleDependency> {
        self.entries
            .get(module_identifier)
            .ok_or_else(|| Error::MissingModule {
                module_identifier: module_identifier.as_ref().to_owned(),
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleUse {
    SkillContent,
    RoleContent,
}

impl ModuleUse {
    fn expected_description(&self) -> &'static str {
        match self {
            Self::SkillContent => "RuntimeSkill",
            Self::RoleContent => "RuntimeSkill or RoleComposition",
        }
    }

    fn accepts(&self, module_kind: ModuleKind) -> bool {
        match self {
            Self::SkillContent => module_kind == ModuleKind::RuntimeSkill,
            Self::RoleContent => matches!(
                module_kind,
                ModuleKind::RuntimeSkill | ModuleKind::RoleComposition
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ModuleExpansion<'a> {
    module_index: &'a ModuleIndex,
    module_use: ModuleUse,
    resolved: BTreeSet<ModuleIdentifier>,
    visiting: Vec<ModuleIdentifier>,
    paths: Vec<ModulePath>,
}

impl<'a> ModuleExpansion<'a> {
    fn new(module_index: &'a ModuleIndex, module_use: ModuleUse) -> Self {
        Self {
            module_index,
            module_use,
            resolved: BTreeSet::new(),
            visiting: Vec::new(),
            paths: Vec::new(),
        }
    }

    fn append_role_source(&mut self, module_identifier: &ModuleIdentifier) -> Result<()> {
        if self.resolved.contains(module_identifier) {
            return Ok(());
        }
        let dependency = self.module_index.dependency(module_identifier)?;
        dependency.require_kind(ModuleKind::RoleSource, "RoleSource")?;
        self.resolved.insert(module_identifier.clone());
        self.paths.push(dependency.module_path.clone());
        Ok(())
    }

    fn append(&mut self, module_identifier: &ModuleIdentifier) -> Result<()> {
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
            self.append(dependency_identifier)?;
        }
        self.visiting.pop();
        self.resolved.insert(module_identifier.clone());
        self.paths.push(dependency.module_path.clone());
        Ok(())
    }

    fn into_paths(self) -> Vec<ModulePath> {
        self.paths
    }
}

impl ModuleDependency {
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
        }
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
        if self.emission_policy != crate::schema::assembly::EmissionPolicy::FirstClassSkill {
            return None;
        }
        Some(ActiveOutput::Skill(ActiveSkill {
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
            modules: Modules::new(vec![ModulePath::new(self.extra_module_path.as_ref())]),
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

    fn is_skill(&self) -> bool {
        matches!(self, Self::AgentsSkill | Self::ClaudeSkill)
    }

    fn is_role(&self) -> bool {
        matches!(self, Self::ClaudeAgent | Self::CodexAgent | Self::PiAgent)
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestAssembler {
    source_root: PathBuf,
    workspace_root: PathBuf,
    manifest: Manifest,
}

impl ManifestAssembler {
    fn new(source_root: PathBuf, workspace_root: PathBuf, manifest: Manifest) -> Self {
        Self {
            source_root,
            workspace_root,
            manifest,
        }
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
        let mut fragments = Vec::new();
        for module_path in self.manifest.modules.payload() {
            fragments.push(MarkdownFragment::read(
                self.module_workspace_path(module_path)?,
            )?);
        }
        let rendered = MarkdownAssembly::new(
            output_path.clone(),
            self.manifest.output_surface,
            self.manifest.frontmatter.payload().to_vec(),
            fragments,
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
        let developer_instructions = match self.manifest.output_surface {
            OutputSurface::CodexAgent => {
                format!("{body}\n\n{CODEX_SKILL_READ_DEDUPLICATION_INSTRUCTION}")
            }
            _ => body,
        };
        if self.manifest.output_surface.is_role() {
            RetiredCurrentDestinationProse::new(output_path.clone(), &developer_instructions)
                .validate()?;
        }
        RoleToml::new(&self.manifest.frontmatter, developer_instructions).render()
    }

    fn markdown_fragments(&self) -> Result<Vec<MarkdownFragment>> {
        let mut fragments = Vec::new();
        for module_path in self.manifest.modules.payload() {
            fragments.push(MarkdownFragment::read(
                self.module_workspace_path(module_path)?,
            )?);
        }
        Ok(fragments)
    }

    fn module_workspace_path(&self, module_path: &ModulePath) -> Result<WorkspacePath> {
        WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(module_path.as_ref()),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleToml {
    name: String,
    description: String,
    developer_instructions: String,
}

impl RoleToml {
    fn new(frontmatter: &Frontmatter, developer_instructions: String) -> Self {
        let mut name = String::new();
        let mut description = String::new();
        for entry in frontmatter.payload() {
            match entry.frontmatter_key.as_ref() {
                "name" => name = entry.frontmatter_value.as_ref().to_owned(),
                "description" => description = entry.frontmatter_value.as_ref().to_owned(),
                _ => {}
            }
        }
        Self {
            name,
            description,
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
