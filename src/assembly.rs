use std::{env, fs, path::PathBuf};

use nota::{NotaEncode, NotaSource};

use crate::{
    error::{Error, Result},
    markdown::{MarkdownAssembly, MarkdownFragment, NotaAssembly, NotaFragment},
    schema::assembly::{
        ByteCount, GeneratedFile, GeneratedFiles, GenerationMode, GenerationOutcome,
        GenerationReport, GenerationRequest, Manifest, ManifestPath, ModulePath, Operation,
        OutputKind,
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
        let mode = self.generation_mode;
        let mut files = Vec::new();
        for manifest_path in self.manifests.payload() {
            let report = ManifestFile::new(
                source_root.clone(),
                workspace_root.clone(),
                manifest_path.clone(),
            )
            .generate(mode)?;
            files.push(report);
        }
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
struct ManifestFile {
    source_root: PathBuf,
    workspace_root: PathBuf,
    manifest_path: ManifestPath,
}

impl ManifestFile {
    fn new(source_root: PathBuf, workspace_root: PathBuf, manifest_path: ManifestPath) -> Self {
        Self {
            source_root,
            workspace_root,
            manifest_path,
        }
    }

    fn generate(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        let manifest = self.read_manifest()?;
        ManifestAssembler::new(
            self.source_root.clone(),
            self.workspace_root.clone(),
            manifest,
        )
        .generate(mode)
    }

    fn read_manifest(&self) -> Result<Manifest> {
        let manifest_workspace_path = WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(self.manifest_path.as_ref()),
        )?;
        let path = manifest_workspace_path.full_path();
        let text = fs::read_to_string(&path).map_err(|source| Error::ReadFile {
            path: path.clone(),
            source,
        })?;
        NotaSource::new(&text)
            .parse::<Manifest>()
            .map_err(|source| Error::DecodeNota { path, source })
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
        OutputWriter::new(output_path, rendered).write(mode)
    }

    fn render(&self, output_path: &WorkspacePath) -> Result<String> {
        match self.manifest.output_kind {
            OutputKind::Markdown => self.render_markdown(output_path),
            OutputKind::Nota => self.render_nota(),
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
            self.manifest.output_surface.clone(),
            self.manifest.frontmatter.payload().to_vec(),
            fragments,
        )
        .render()
    }

    fn render_nota(&self) -> Result<String> {
        let mut fragments = Vec::new();
        for module_path in self.manifest.modules.payload() {
            fragments.push(NotaFragment::read(
                self.module_workspace_path(module_path)?,
            )?);
        }
        NotaAssembly::new(fragments).render()
    }

    fn module_workspace_path(&self, module_path: &ModulePath) -> Result<WorkspacePath> {
        WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(module_path.as_ref()),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutputWriter {
    output_path: WorkspacePath,
    rendered: String,
}

impl OutputWriter {
    fn new(output_path: WorkspacePath, rendered: String) -> Self {
        Self {
            output_path,
            rendered,
        }
    }

    fn write(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        match mode {
            GenerationMode::Write => self.write_file()?,
            GenerationMode::Check => self.check_file()?,
        }
        Ok(GeneratedFile {
            output_path: crate::schema::assembly::OutputPath::new(
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

impl GenerationReport {
    pub fn to_nota_text(&self) -> String {
        GenerationOutcome::Generated(self.clone()).to_nota()
    }
}
