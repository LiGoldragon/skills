use std::{collections::BTreeSet, env, fs, io, path::PathBuf};

use nota::{NotaEncode, NotaSource};

use crate::{
    error::{Error, Result},
    markdown::{MarkdownAssembly, MarkdownFragment},
    schema::assembly::{
        ByteCount, EntryPoint, EntryPointExtra, ExtraSurface, Frontmatter, FrontmatterEntry,
        FrontmatterKey, FrontmatterValue, GeneratedFile, GeneratedFiles, GenerationMode,
        GenerationOutcome, GenerationReport, GenerationRequest, Manifest, ModuleLifecycle,
        ModulePath, Modules, Operation, OutputKind, OutputPath, OutputSurface, RosterPath,
        SkillCategory, SkillMetadata, SkillModule, SkillRoster, SkillTier, TargetSurface,
    },
    workspace_path::WorkspacePath,
};

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
        let roster = RosterFile::new(source_root.clone(), self.roster_path.clone()).read()?;
        if self.generation_mode == GenerationMode::Write {
            WorkspacePruner::new(workspace_root.clone(), roster.clone()).prune()?;
        }
        let jobs =
            GenerationJobs::new(source_root, workspace_root.clone(), roster.clone()).jobs()?;
        let mut files = Vec::new();
        for job in jobs {
            files.push(job.generate(self.generation_mode)?);
        }
        StaleOutputScan::new(workspace_root, roster).validate()?;
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
struct RosterFile {
    source_root: PathBuf,
    roster_path: RosterPath,
}

impl RosterFile {
    fn new(source_root: PathBuf, roster_path: RosterPath) -> Self {
        Self {
            source_root,
            roster_path,
        }
    }

    fn read(&self) -> Result<SkillRoster> {
        let roster_workspace_path = WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(self.roster_path.as_ref()),
        )?;
        let path = roster_workspace_path.full_path();
        let text = fs::read_to_string(&path).map_err(|source| Error::ReadFile {
            path: path.clone(),
            source,
        })?;
        NotaSource::new(&text)
            .parse::<SkillRoster>()
            .map_err(|source| Error::DecodeNota { path, source })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationJobs {
    source_root: PathBuf,
    workspace_root: PathBuf,
    roster: SkillRoster,
}

impl GenerationJobs {
    fn new(source_root: PathBuf, workspace_root: PathBuf, roster: SkillRoster) -> Self {
        Self {
            source_root,
            workspace_root,
            roster,
        }
    }

    fn jobs(&self) -> Result<Vec<GenerationJob>> {
        let mut jobs = Vec::new();
        jobs.push(GenerationJob::Rendered(RenderedOutput::new(
            self.workspace_root.clone(),
            OutputPath::new("skills/skills.nota"),
            SkillIndex::new(self.roster.clone()).render(),
        )?));
        for module in self.roster.skill_modules.payload() {
            for manifest in module.first_class_manifests() {
                jobs.push(GenerationJob::Manifest(ManifestAssembler::new(
                    self.source_root.clone(),
                    self.workspace_root.clone(),
                    manifest,
                )));
            }
        }
        for entry_point in self.roster.entry_points.payload() {
            for manifest in entry_point.extra_manifests()? {
                jobs.push(GenerationJob::Manifest(ManifestAssembler::new(
                    self.source_root.clone(),
                    self.workspace_root.clone(),
                    manifest,
                )));
            }
        }
        Ok(jobs)
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
}

impl SkillModule {
    fn first_class_manifests(&self) -> Vec<Manifest> {
        if self.emission_policy != crate::schema::assembly::EmissionPolicy::FirstClassSkill {
            return Vec::new();
        }
        let ModuleLifecycle::Active(metadata) = &self.module_lifecycle else {
            return Vec::new();
        };
        self.target_surfaces
            .payload()
            .iter()
            .map(|surface| self.first_class_manifest(surface, metadata))
            .collect()
    }

    fn first_class_manifest(&self, surface: &TargetSurface, metadata: &SkillMetadata) -> Manifest {
        let module_name = self.module_name.payload();
        let output_surface = OutputSurface::from(surface);
        Manifest {
            output_path: OutputPath::new(output_surface.skill_path(module_name)),
            output_kind: OutputKind::Markdown,
            output_surface,
            frontmatter: metadata.frontmatter(module_name),
            modules: Modules::new(vec![self.module_path.clone()]),
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
            Self::Workspace | Self::ClaudeCommand | Self::CodexPrompt | Self::CodexCommand => {
                unreachable!("not a first-class skill surface")
            }
        }
    }

    fn extra_path(&self, module_name: &str) -> String {
        match self {
            Self::ClaudeCommand => format!(".claude/commands/{module_name}.md"),
            Self::CodexPrompt => format!(".codex/prompts/{module_name}.md"),
            Self::CodexCommand => format!(".codex/commands/{module_name}.md"),
            Self::Workspace | Self::AgentsSkill | Self::ClaudeSkill => {
                unreachable!("not an entrypoint extra surface")
            }
        }
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
        MarkdownAssembly::new(
            output_path.clone(),
            self.manifest.output_surface,
            self.manifest.frontmatter.payload().to_vec(),
            fragments,
        )
        .render()
    }

    fn module_workspace_path(&self, module_path: &ModulePath) -> Result<WorkspacePath> {
        WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(module_path.as_ref()),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SkillIndex {
    roster: SkillRoster,
}

impl SkillIndex {
    fn new(roster: SkillRoster) -> Self {
        Self { roster }
    }

    fn render(&self) -> String {
        let mut output = String::from(
            ";; NOTA records are positional. Type, then fields, no keywords.\n\
             ;; The `(key value)` shape from Lisp/Clojure/JSON is not NOTA.\n\
             ;; If you're sketching a new NOTA record, read .agents/skills/nota-design/SKILL.md first.\n\
             ;;\n\
             ;; tier values (fourth positional field below):\n\
             ;;   apex      — read once; recognise everywhere\n\
             ;;   keystroke — applies on every keystroke and every report\n\
             ;;   topic     — consulted when the topic comes up\n\
             ;;   mechanism — procedural; consulted when the named mechanism is in play\n\n[\n",
        );
        let mut previous_category = None;
        for module in self.roster.skill_modules.payload() {
            let ModuleLifecycle::Active(metadata) = &module.module_lifecycle else {
                continue;
            };
            if previous_category
                .as_ref()
                .is_some_and(|category| category != &metadata.skill_category)
            {
                output.push('\n');
            }
            previous_category = Some(metadata.skill_category);
            output.push_str("  ");
            output.push_str(&metadata.index_record(module.module_name.payload()));
            output.push('\n');
        }
        output.push_str("]\n");
        output
    }
}

impl SkillMetadata {
    fn index_record(&self, module_name: &str) -> String {
        format!(
            "({} {module_name} .agents/skills/{module_name}/SKILL.md {} {})",
            self.skill_category.as_str(),
            self.skill_tier.as_str(),
            self.skill_description.to_nota()
        )
    }
}

impl SkillCategory {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Architecture => "Architecture",
            Self::Craft => "Craft",
            Self::Programming => "Programming",
            Self::Workflow => "Workflow",
            Self::Meta => "Meta",
        }
    }
}

impl SkillTier {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Apex => "Apex",
            Self::Keystroke => "Keystroke",
            Self::Topic => "Topic",
            Self::Mechanism => "Mechanism",
        }
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
struct WorkspacePruner {
    workspace_root: PathBuf,
    roster: SkillRoster,
}

impl WorkspacePruner {
    fn new(workspace_root: PathBuf, roster: SkillRoster) -> Self {
        Self {
            workspace_root,
            roster,
        }
    }

    fn prune(&self) -> Result<()> {
        self.remove_relative_path(".agents/skills")?;
        self.remove_relative_path(".claude/skills")?;
        for entry_point in self.roster.entry_points.payload() {
            for output_path in entry_point.extra_paths() {
                self.remove_relative_path(output_path.as_ref())?;
            }
        }
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
    roster: SkillRoster,
}

impl StaleOutputScan {
    fn new(workspace_root: PathBuf, roster: SkillRoster) -> Self {
        Self {
            workspace_root,
            roster,
        }
    }

    fn validate(&self) -> Result<()> {
        let expected = self.expected_outputs();
        for module in self.roster.skill_modules.payload() {
            for output_path in module.first_class_paths() {
                if !expected.contains(output_path.as_ref()) {
                    self.require_absent(output_path.as_ref())?;
                }
            }
        }
        Ok(())
    }

    fn expected_outputs(&self) -> BTreeSet<String> {
        let mut expected = BTreeSet::new();
        expected.insert("skills/skills.nota".to_owned());
        for module in self.roster.skill_modules.payload() {
            for manifest in module.first_class_manifests() {
                expected.insert(manifest.output_path.as_ref().to_owned());
            }
        }
        for entry_point in self.roster.entry_points.payload() {
            for output_path in entry_point.extra_paths() {
                expected.insert(output_path.as_ref().to_owned());
            }
        }
        expected
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
}

impl GenerationReport {
    pub fn to_nota_text(&self) -> String {
        GenerationOutcome::Generated(self.clone()).to_nota()
    }
}
