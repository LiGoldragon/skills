use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspacePath {
    root: PathBuf,
    relative: PathBuf,
}

impl WorkspacePath {
    pub fn new(root: impl Into<PathBuf>, relative: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let relative = relative.into();
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(Error::PathEscapesRoot {
                root,
                path: relative,
            });
        }
        Ok(Self { root, relative })
    }

    pub fn full_path(&self) -> PathBuf {
        self.root.join(&self.relative)
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
}
