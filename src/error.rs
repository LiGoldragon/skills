use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("expected exactly one NOTA argument or manifest file path, found {count}")]
    ArgumentCount { count: usize },

    #[error("read {path}: {source}")]
    ReadFile { path: PathBuf, source: io::Error },

    #[error("write {path}: {source}")]
    WriteFile { path: PathBuf, source: io::Error },

    #[error("remove generated path {path}: {source}")]
    RemovePath { path: PathBuf, source: io::Error },

    #[error("create directory {path}: {source}")]
    CreateDirectory { path: PathBuf, source: io::Error },

    #[error("decode NOTA from {path}: {source}")]
    DecodeNota {
        path: PathBuf,
        source: nota::NotaDecodeError,
    },

    #[error("decode NOTA argument: {0}")]
    DecodeNotaArgument(nota::NotaDecodeError),

    #[error("environment variable {variable} must name a generation root")]
    MissingEnvironmentRoot { variable: String },

    #[error("module `{module_identifier}` is listed more than once in module dependencies")]
    DuplicateModule { module_identifier: String },

    #[error("module `{module_identifier}` is missing from module dependencies")]
    MissingModule { module_identifier: String },

    #[error("module dependency cycle: {}", module_identifiers.join(" -> "))]
    ModuleDependencyCycle { module_identifiers: Vec<String> },

    #[error(
        "generated output path `{relative_path}` resolves to duplicate physical path {physical_path}"
    )]
    DuplicateOutputPath {
        relative_path: String,
        physical_path: PathBuf,
    },

    #[error("duplicate markdown heading `{heading}` in {path}")]
    DuplicateHeading { path: PathBuf, heading: String },

    #[error("markdown output {path} must contain exactly one level-one title; found {count}")]
    InvalidTitleCount { path: PathBuf, count: usize },

    #[error("markdown heading jumps from level {previous} to {current} in {path}: `{heading}`")]
    HeadingLevelJump {
        path: PathBuf,
        previous: usize,
        current: usize,
        heading: String,
    },

    #[error("harness skill {path} must define YAML frontmatter")]
    MissingHarnessFrontmatter { path: PathBuf },

    #[error("harness skill {path} frontmatter must define `{key}`")]
    MissingHarnessFrontmatterKey { path: PathBuf, key: String },

    #[error("frontmatter is allowed only at the start of {path}")]
    NestedFrontmatter { path: PathBuf },

    #[error("frontmatter key `{key}` in {path} contains unsupported characters")]
    InvalidFrontmatterKey { path: PathBuf, key: String },

    #[error("frontmatter value for `{key}` in {path} must be a single line")]
    InvalidFrontmatterValue { path: PathBuf, key: String },

    #[error("relative path {path} escapes the workspace root {root}")]
    PathEscapesRoot { root: PathBuf, path: PathBuf },

    #[error(
        "generated output is stale: {path}. Update the locked `skills` input, run `nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>`, then rerun `nix run github:LiGoldragon/skills#check-skills -- <workspace-root>`."
    )]
    StaleOutput { path: PathBuf },

    #[error(
        "stale generated archived/deleted skill output remains: {path}. Update the locked `skills` input, run `nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>`, then rerun `nix run github:LiGoldragon/skills#check-skills -- <workspace-root>`."
    )]
    StaleGeneratedOutput { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;
