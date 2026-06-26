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

    #[error("create directory {path}: {source}")]
    CreateDirectory { path: PathBuf, source: io::Error },

    #[error("decode NOTA from {path}: {source}")]
    DecodeNota {
        path: PathBuf,
        source: nota::NotaDecodeError,
    },

    #[error("decode NOTA argument: {0}")]
    DecodeNotaArgument(nota::NotaDecodeError),

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

    #[error("frontmatter is allowed only at the start of {path}")]
    NestedFrontmatter { path: PathBuf },

    #[error("frontmatter key `{key}` in {path} contains unsupported characters")]
    InvalidFrontmatterKey { path: PathBuf, key: String },

    #[error("relative path {path} escapes the workspace root {root}")]
    PathEscapesRoot { root: PathBuf, path: PathBuf },

    #[error("generated output is stale: {path}")]
    StaleOutput { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;
