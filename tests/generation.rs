use std::{fs, path::Path};

use nota::NotaSource;
use skills::{
    Error,
    schema::assembly::{
        EmissionPolicy, EntryPointKind, GenerationMode, GenerationRequest, HarnessTarget,
        ManifestPath, Manifests, MigrationPlan, ModuleStatus, SourceRoot, WorkspaceRoot,
    },
};
use tempfile::TempDir;

#[test]
fn generation_writes_manifest_frontmatter_and_strips_module_frontmatter() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "modules/example.md",
        "---\nname: stale\n---\n\n# Skill — example\n\n## Rule\n\nKeep the prose.\n",
    );
    fixture.write_source_file(
        "manifests/example.nota",
        "(skills/example.md Markdown (Harness Pi) [(name example) (description [Example skill.])] [modules/example.md])\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");

    let generated = fixture.read_workspace_file("skills/example.md");
    assert_eq!(
        generated,
        "---\nname: example\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\nKeep the prose.\n"
    );
}

#[test]
fn generation_fails_on_duplicate_headings() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "modules/example.md",
        "# Skill — example\n\n## Repeat\n\nFirst.\n\n## Repeat\n\nSecond.\n",
    );
    fixture.write_source_file(
        "manifests/example.nota",
        "(skills/example.md Markdown Workspace [] [modules/example.md])\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("duplicate headings fail");

    assert!(matches!(error, Error::DuplicateHeading { .. }), "{error:?}");
}

#[test]
fn migration_model_covers_current_skills_and_marks_deleted_worker_surface() {
    let text = include_str!("../manifests/migration/current-skills.nota");
    let plan = NotaSource::new(text)
        .parse::<MigrationPlan>()
        .expect("migration model parses");

    assert_eq!(plan.archive_root.as_ref(), "skills/archive");
    assert_eq!(plan.migration_modules.payload().len(), 76);

    let active = plan
        .migration_modules
        .payload()
        .iter()
        .find(|module| module.module_name.payload() == "intent-led-orchestration")
        .expect("intent-led-orchestration modeled");
    assert_eq!(active.module_status, ModuleStatus::Active);
    assert_eq!(active.emission_policy, EmissionPolicy::FirstClassSkill);
    assert_eq!(
        active.target_set.payload(),
        &[
            HarnessTarget::Pi,
            HarnessTarget::Claude,
            HarnessTarget::Codex
        ]
    );

    let deleted = plan
        .migration_modules
        .payload()
        .iter()
        .find(|module| module.module_name.payload() == "subagent-session-workflow")
        .expect("deleted subagent workflow modeled");
    assert_eq!(deleted.module_status, ModuleStatus::Deleted);
    assert_eq!(deleted.emission_policy, EmissionPolicy::NoEmission);
    assert!(deleted.target_set.payload().is_empty());

    let entry_point = plan
        .entry_points
        .payload()
        .iter()
        .find(|entry_point| entry_point.module_name.payload() == "intent-led-orchestration")
        .expect("entrypoint modeled");
    assert_eq!(entry_point.entry_point_extras.payload().len(), 3);
    assert!(
        entry_point
            .entry_point_extras
            .payload()
            .iter()
            .any(|extra| {
                extra.harness_target == HarnessTarget::Claude
                    && extra.entry_point_kind == EntryPointKind::Command
            })
    );
}

#[test]
fn check_mode_reports_stale_output() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "modules/example.md",
        "# Skill — example\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_source_file(
        "manifests/example.nota",
        "(skills/example.md Markdown Workspace [] [modules/example.md])\n",
    );
    fixture.write_workspace_file(
        "skills/example.md",
        "# Skill — example\n\n## Rule\n\nOld.\n",
    );

    let error = fixture
        .generate(GenerationMode::Check)
        .expect_err("stale output fails check mode");

    assert!(matches!(error, Error::StaleOutput { .. }), "{error:?}");
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
            manifests: Manifests::new(vec![ManifestPath::new("manifests/example.nota")]),
            generation_mode,
        }
        .generate()
    }
}
