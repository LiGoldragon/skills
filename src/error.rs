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

    #[error("module `{module_identifier}` has kind `{actual}`, expected {expected}")]
    InvalidModuleKind {
        module_identifier: String,
        expected: String,
        actual: String,
    },

    #[error("module dependency cycle: {}", module_identifiers.join(" -> "))]
    ModuleDependencyCycle { module_identifiers: Vec<String> },

    #[error("model `{model_identifier}` is listed more than once in the model catalog")]
    DuplicateModelCatalogEntry { model_identifier: String },

    #[error("model `{model_identifier}` lists effort `{effort}` more than once")]
    DuplicateModelCatalogEffort {
        model_identifier: String,
        effort: String,
    },

    #[error("active role `{role_identifier}` has no model assignment")]
    MissingRoleModelAssignment { role_identifier: String },

    #[error("role `{role_identifier}` is listed more than once in role model assignments")]
    DuplicateRoleModelAssignment { role_identifier: String },

    #[error("model assignment names inactive role `{role_identifier}`")]
    StaleRoleModelAssignment { role_identifier: String },

    #[error("role `{role_identifier}` assigns unsupported model `{model_identifier}`")]
    UnsupportedRoleModel {
        role_identifier: String,
        model_identifier: String,
    },

    #[error(
        "role `{role_identifier}` assigns `{model_identifier}` as {expected_family}, but the catalog marks it {actual_family}"
    )]
    RoleModelFamilyMismatch {
        role_identifier: String,
        model_identifier: String,
        expected_family: String,
        actual_family: String,
    },

    #[error(
        "role `{role_identifier}` assigns unsupported effort `{effort}` to model `{model_identifier}`"
    )]
    UnsupportedRoleModelEffort {
        role_identifier: String,
        model_identifier: String,
        effort: String,
    },

    #[error("active role `{role_identifier}` has no optional-skill metadata")]
    MissingRoleOptionalSkills { role_identifier: String },

    #[error("role `{role_identifier}` is listed more than once in optional-skill metadata")]
    DuplicateRoleOptionalSkills { role_identifier: String },

    #[error("optional-skill metadata names inactive role `{role_identifier}`")]
    StaleRoleOptionalSkills { role_identifier: String },

    #[error("role `{role_identifier}` lists optional skill `{skill_identifier}` more than once")]
    DuplicateOptionalSkill {
        role_identifier: String,
        skill_identifier: String,
    },

    #[error(
        "role `{role_identifier}` lists inactive or renamed optional skill `{skill_identifier}`"
    )]
    MissingOptionalSkill {
        role_identifier: String,
        skill_identifier: String,
    },

    #[error(
        "optional skill `{skill_identifier}` for role `{role_identifier}` does not support role surface `{role_surface}`"
    )]
    TargetIncompatibleOptionalSkill {
        role_identifier: String,
        skill_identifier: String,
        role_surface: String,
    },

    #[error("nested role `{role_identifier}` is listed more than once")]
    DuplicateNestedRoleRelation { role_identifier: String },

    #[error("nested-role metadata names inactive role `{role_identifier}`")]
    InactiveNestedRole { role_identifier: String },

    #[error("Manager is the root Manager and cannot be a NestedRole")]
    ManagerCannotBeNestedRole,

    #[error("nested role `{role_identifier}` has no allowed leaf role")]
    MissingNestedRoleChild { role_identifier: String },

    #[error("nested role `{role_identifier}` lists child `{child_identifier}` more than once")]
    DuplicateNestedRoleChild {
        role_identifier: String,
        child_identifier: String,
    },

    #[error("nested role `{role_identifier}` cannot delegate to itself")]
    NestedRoleSelfEdge { role_identifier: String },

    #[error("nested role `{role_identifier}` cannot name Manager as a child")]
    ManagerCannotBeNestedChild { role_identifier: String },

    #[error("nested role `{role_identifier}` cannot delegate to nested role `{child_identifier}`")]
    NestedRoleChildCannotBeNested {
        role_identifier: String,
        child_identifier: String,
    },

    #[error("nested role `{role_identifier}` names inactive leaf role `{child_identifier}`")]
    InactiveNestedRoleChild {
        role_identifier: String,
        child_identifier: String,
    },

    #[error(
        "leaf role `{child_identifier}` for nested role `{role_identifier}` does not support role surface `{role_surface}`"
    )]
    TargetIncompatibleNestedRoleChild {
        role_identifier: String,
        child_identifier: String,
        role_surface: String,
    },

    #[error(
        "nested role `{role_identifier}` has no minimum model for role surface `{role_surface}`"
    )]
    MissingNestedRoleMinimumModel {
        role_identifier: String,
        role_surface: String,
    },

    #[error(
        "nested role `{role_identifier}` has more than one minimum model for role surface `{role_surface}`"
    )]
    DuplicateNestedRoleMinimumModel {
        role_identifier: String,
        role_surface: String,
    },

    #[error(
        "nested role `{role_identifier}` sets a minimum model for inactive role surface `{role_surface}`"
    )]
    NestedRoleMinimumForInactiveTarget {
        role_identifier: String,
        role_surface: String,
    },

    #[error(
        "nested role `{role_identifier}` minimum `{model_identifier}` for `{role_surface}` must be {expected_family}, but the catalog marks it {actual_family}"
    )]
    NestedRoleMinimumModelFamilyMismatch {
        role_identifier: String,
        model_identifier: String,
        role_surface: String,
        expected_family: String,
        actual_family: String,
    },

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

    #[error(
        "generated skill `{skill_name}` serialized block at `{location}` is {byte_count} bytes, exceeding the {limit} byte limit"
    )]
    GeneratedSkillBlockTooLarge {
        skill_name: String,
        location: String,
        byte_count: usize,
        limit: usize,
    },

    #[error("retired current-destination prose `{phrase}` appears in generated role output {path}")]
    RetiredCurrentDestinationProse { path: PathBuf, phrase: String },

    #[error(
        "generated agent packet {path} configures forbidden execution limit field `{field_name}`"
    )]
    GeneratedAgentExecutionLimit { path: PathBuf, field_name: String },

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

    #[error(
        "skills source checkout {source_root} is not a descendant of the fetched remote trunk; regenerating from it would silently revert corrections already landed on trunk. Rebase your checkout onto the latest trunk first (jj: `jj git fetch`, then rebase your work onto `trunk()`), then retry."
    )]
    SourceNotDescendantOfTrunk { source_root: PathBuf },

    #[error("verify source trunk descent in {source_root}: run `jj {command}`: {source}")]
    TrunkGuardCommand {
        command: String,
        source_root: PathBuf,
        source: io::Error,
    },

    #[error("verify source trunk descent in {source_root}: `jj {command}` failed: {stderr}")]
    TrunkGuardCommandFailed {
        command: String,
        source_root: PathBuf,
        stderr: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
