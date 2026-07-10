use std::{path::PathBuf, process::Command};

use crate::error::{Error, Result};

/// Refuses regeneration from a mutable source checkout that is not a descendant
/// of the freshly fetched remote trunk, so generation cannot silently revert
/// corrections already landed on trunk. An immutable source (a checkout with no
/// Jujutsu working copy, such as a pinned Nix store copy) is inherently safe and
/// passes without a fetch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrunkDescendantGuard {
    source_root: PathBuf,
}

impl TrunkDescendantGuard {
    pub fn new(source_root: impl Into<PathBuf>) -> Self {
        Self {
            source_root: source_root.into(),
        }
    }

    pub fn verify(&self) -> Result<()> {
        match self.working_copy() {
            SourceWorkingCopy::Immutable => Ok(()),
            SourceWorkingCopy::Jujutsu => self.verify_descendant(),
        }
    }

    fn working_copy(&self) -> SourceWorkingCopy {
        if self.source_root.join(".jj").is_dir() {
            SourceWorkingCopy::Jujutsu
        } else {
            SourceWorkingCopy::Immutable
        }
    }

    fn verify_descendant(&self) -> Result<()> {
        self.fetch_trunk()?;
        if self.trunk_divergence()?.requires_refusal() {
            Err(Error::SourceNotDescendantOfTrunk {
                source_root: self.source_root.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn fetch_trunk(&self) -> Result<()> {
        self.run_jujutsu(&["git", "fetch"]).map(|_| ())
    }

    fn trunk_divergence(&self) -> Result<TrunkDivergence> {
        let stdout = self.run_jujutsu(&[
            "log",
            "--no-graph",
            "-r",
            "trunk() ~ ::@",
            "-T",
            "change_id ++ \"\\n\"",
        ])?;
        Ok(TrunkDivergence::from_revset_output(&stdout))
    }

    fn run_jujutsu(&self, subcommand: &[&str]) -> Result<String> {
        let command = subcommand.join(" ");
        let output = Command::new("jj")
            .arg("--repository")
            .arg(&self.source_root)
            .arg("--ignore-working-copy")
            .args(subcommand)
            .output()
            .map_err(|source| Error::TrunkGuardCommand {
                command: command.clone(),
                source_root: self.source_root.clone(),
                source,
            })?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(Error::TrunkGuardCommandFailed {
                command,
                source_root: self.source_root.clone(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceWorkingCopy {
    Jujutsu,
    Immutable,
}

/// The trunk commits that are not ancestors of the source working copy. When any
/// remain, the working copy is a sibling or is behind trunk rather than a
/// descendant, and regeneration must be refused.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrunkDivergence {
    divergent_change_ids: Vec<String>,
}

impl TrunkDivergence {
    pub fn from_revset_output(stdout: &str) -> Self {
        Self {
            divergent_change_ids: stdout
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect(),
        }
    }

    pub fn requires_refusal(&self) -> bool {
        !self.divergent_change_ids.is_empty()
    }
}
