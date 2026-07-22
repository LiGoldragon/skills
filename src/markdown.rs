use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use pathdiff::diff_paths;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::{
    error::{Error, Result},
    schema::assembly::{FrontmatterEntry, ModuleIdentifier, OutputSurface},
    workspace_path::WorkspacePath,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownAssembly {
    output_path: WorkspacePath,
    output_surface: OutputSurface,
    frontmatter: Vec<FrontmatterEntry>,
    fragments: Vec<MarkdownFragment>,
}

impl MarkdownAssembly {
    pub fn new(
        output_path: WorkspacePath,
        output_surface: OutputSurface,
        frontmatter: Vec<FrontmatterEntry>,
        fragments: Vec<MarkdownFragment>,
    ) -> Self {
        Self {
            output_path,
            output_surface,
            frontmatter,
            fragments,
        }
    }

    pub fn render(&self) -> Result<String> {
        let mut output = String::new();
        let frontmatter = FrontmatterBlock::new(self.output_path.full_path(), &self.frontmatter);
        output.push_str(&frontmatter.render()?);
        let title = self.title();
        for (index, fragment) in self.fragments.iter().enumerate() {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push('\n');
            }
            output.push_str(&fragment.normalized_text(
                index,
                title.as_deref(),
                &self.output_path,
            )?);
            if !output.ends_with('\n') {
                output.push('\n');
            }
        }
        HarnessSkillFrontmatter::new(
            self.output_path.clone(),
            &self.output_surface,
            &self.frontmatter,
            &output,
        )
        .validate()?;
        MarkdownStructure::new(self.output_path.full_path(), output).validate()
    }

    fn title(&self) -> Option<String> {
        self.fragments
            .iter()
            .find_map(MarkdownFragment::first_title)
            .map(|title| HeadingName::new(&title).normalized())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownFragment {
    path: WorkspacePath,
    text: String,
    composition_heading: Option<String>,
}

impl MarkdownFragment {
    pub fn from_text(path: WorkspacePath, text: impl Into<String>) -> Self {
        Self {
            path,
            text: text.into(),
            composition_heading: None,
        }
    }

    pub fn read(path: WorkspacePath) -> Result<Self> {
        let full_path = path.full_path();
        fs::read_to_string(&full_path)
            .map(|text| Self {
                path,
                text,
                composition_heading: None,
            })
            .map_err(|source| Error::ReadFile {
                path: full_path,
                source,
            })
    }

    pub fn read_composition(
        path: WorkspacePath,
        module_identifier: &ModuleIdentifier,
    ) -> Result<Self> {
        let full_path = path.full_path();
        let text = fs::read_to_string(&full_path).map_err(|source| Error::ReadFile {
            path: full_path.clone(),
            source,
        })?;
        CompositionSource::new(module_identifier, &full_path, &text).validate()?;
        Ok(Self {
            path,
            text,
            composition_heading: Some(
                HumanizedIdentifier::new(module_identifier.as_ref()).render(),
            ),
        })
    }

    pub fn normalized_text(
        &self,
        index: usize,
        title: Option<&str>,
        output_path: &WorkspacePath,
    ) -> Result<String> {
        let body =
            MarkdownBody::new(self.path.full_path(), self.text.as_str()).without_frontmatter()?;
        let runtime_body = SourceMaintenanceNotes::new(body).runtime_text();
        let links_rebased = MarkdownLinks::new(&runtime_body, &self.path, output_path).rebased();
        if let Some(heading) = &self.composition_heading {
            return Ok(format!("## {heading}\n\n{links_rebased}"));
        }
        let heading_normalized = HeadingNormalizer::new(links_rebased, index, title).normalized();
        Ok(heading_normalized)
    }

    fn first_title(&self) -> Option<String> {
        MarkdownBody::new(self.path.full_path(), self.text.as_str())
            .without_frontmatter()
            .ok()
            .map(SourceMaintenanceNotes::new)
            .map(SourceMaintenanceNotes::runtime_text)
            .and_then(|body| HeadingText::from_markdown(&body).first_title())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompositionSource<'a> {
    module_identifier: &'a ModuleIdentifier,
    path: &'a Path,
    text: &'a str,
}

impl<'a> CompositionSource<'a> {
    fn new(module_identifier: &'a ModuleIdentifier, path: &'a Path, text: &'a str) -> Self {
        Self {
            module_identifier,
            path,
            text,
        }
    }

    fn validate(&self) -> Result<()> {
        let body = MarkdownBody::new(self.path.to_path_buf(), self.text).without_frontmatter()?;
        let headings = HeadingText::from_markdown(&body);
        if let Some(heading) = headings.headings.first() {
            return Err(Error::CompositionSourceHeading {
                module_identifier: self.module_identifier.as_ref().to_owned(),
                path: self.path.to_path_buf(),
                heading: heading.text.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HumanizedIdentifier<'a> {
    identifier: &'a str,
}

impl<'a> HumanizedIdentifier<'a> {
    fn new(identifier: &'a str) -> Self {
        Self { identifier }
    }

    fn render(&self) -> String {
        self.identifier
            .split('-')
            .map(|word| match word {
                "nixos" => "NixOS".to_owned(),
                "nota" => "NOTA".to_owned(),
                "pi" => "Pi".to_owned(),
                "vm" => "VM".to_owned(),
                word => {
                    let mut characters = word.chars();
                    match characters.next() {
                        Some(first) => {
                            first.to_uppercase().collect::<String>() + characters.as_str()
                        }
                        None => String::new(),
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FrontmatterBlock<'a> {
    path: PathBuf,
    entries: &'a [FrontmatterEntry],
}

impl<'a> FrontmatterBlock<'a> {
    fn new(path: PathBuf, entries: &'a [FrontmatterEntry]) -> Self {
        Self { path, entries }
    }

    fn render(&self) -> Result<String> {
        if self.entries.is_empty() {
            return Ok(String::new());
        }
        let mut output = String::from("---\n");
        for entry in self.entries {
            let key = entry.frontmatter_key.as_ref();
            FrontmatterKey::new(self.path.clone(), key).validate()?;
            let scalar = YamlScalar::new(
                self.path.clone(),
                key.to_owned(),
                entry.frontmatter_value.as_ref(),
            );
            scalar.validate()?;
            output.push_str(key);
            output.push_str(": ");
            output.push_str(&scalar.rendered());
            output.push('\n');
        }
        output.push_str("---\n\n");
        Ok(output)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HarnessSkillFrontmatter<'a> {
    output_path: WorkspacePath,
    output_surface: &'a OutputSurface,
    entries: &'a [FrontmatterEntry],
    text: &'a str,
}

impl<'a> HarnessSkillFrontmatter<'a> {
    fn new(
        output_path: WorkspacePath,
        output_surface: &'a OutputSurface,
        entries: &'a [FrontmatterEntry],
        text: &'a str,
    ) -> Self {
        Self {
            output_path,
            output_surface,
            entries,
            text,
        }
    }

    fn validate(&self) -> Result<()> {
        if !self.is_harness_skill_file() {
            return Ok(());
        }
        HarnessSkillFrontmatterEntries::new(self.output_path.full_path(), self.entries)
            .validate()?;
        let body = RenderedFrontmatter::new(self.output_path.full_path(), self.text).body()?;
        MarkdownDelimiterScan::new(self.output_path.full_path(), body).validate()
    }

    fn is_harness_skill_file(&self) -> bool {
        let relative = self.output_path.relative_path().to_string_lossy();
        matches!(
            self.output_surface,
            OutputSurface::AgentsSkill | OutputSurface::ClaudeSkill
        ) && ((relative.starts_with(".agents/skills/") && relative.ends_with("/SKILL.md"))
            || (relative.starts_with(".claude/skills/") && relative.ends_with("/SKILL.md")))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HarnessSkillFrontmatterEntries<'a> {
    path: PathBuf,
    entries: &'a [FrontmatterEntry],
}

impl<'a> HarnessSkillFrontmatterEntries<'a> {
    fn new(path: PathBuf, entries: &'a [FrontmatterEntry]) -> Self {
        Self { path, entries }
    }

    fn validate(&self) -> Result<()> {
        if self.entries.is_empty() {
            return Err(Error::MissingHarnessFrontmatter {
                path: self.path.clone(),
            });
        }
        self.require_key("name")?;
        self.require_key("description")
    }

    fn require_key(&self, key: &str) -> Result<()> {
        if self
            .entries
            .iter()
            .any(|entry| entry.frontmatter_key.as_ref() == key)
        {
            Ok(())
        } else {
            Err(Error::MissingHarnessFrontmatterKey {
                path: self.path.clone(),
                key: key.to_owned(),
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedFrontmatter<'a> {
    path: PathBuf,
    text: &'a str,
}

impl<'a> RenderedFrontmatter<'a> {
    fn new(path: PathBuf, text: &'a str) -> Self {
        Self { path, text }
    }

    fn body(&self) -> Result<&'a str> {
        if !self.text.starts_with("---\n") {
            return Err(Error::MissingHarnessFrontmatter {
                path: self.path.clone(),
            });
        }
        let Some(closing_start) = self.text[4..].find("\n---\n") else {
            return Err(Error::MissingHarnessFrontmatter {
                path: self.path.clone(),
            });
        };
        let body_start = 4 + closing_start + "\n---\n".len();
        Ok(&self.text[body_start..])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownDelimiterScan<'a> {
    path: PathBuf,
    text: &'a str,
}

impl<'a> MarkdownDelimiterScan<'a> {
    fn new(path: PathBuf, text: &'a str) -> Self {
        Self { path, text }
    }

    fn validate(&self) -> Result<()> {
        let mut fence = Option::<FenceMarker>::None;
        for line in self.text.lines() {
            let trimmed = line.trim_start();
            if let Some(marker) = FenceMarker::opening(trimmed) {
                match fence {
                    Some(active) if active == marker => fence = None,
                    None => fence = Some(marker),
                    _ => {}
                }
            } else if fence.is_none() && trimmed == "---" {
                return Err(Error::NestedFrontmatter {
                    path: self.path.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FenceMarker {
    Backtick,
    Tilde,
}

impl FenceMarker {
    fn opening(line: &str) -> Option<Self> {
        if line.starts_with("```") {
            Some(Self::Backtick)
        } else if line.starts_with("~~~") {
            Some(Self::Tilde)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct YamlScalar<'a> {
    path: PathBuf,
    key: String,
    text: &'a str,
}

impl<'a> YamlScalar<'a> {
    fn new(path: PathBuf, key: String, text: &'a str) -> Self {
        Self { path, key, text }
    }

    fn validate(&self) -> Result<()> {
        if self.text.contains('\n') || self.text.contains('\r') {
            Err(Error::InvalidFrontmatterValue {
                path: self.path.clone(),
                key: self.key.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn rendered(&self) -> String {
        if self.is_plain() {
            self.text.to_owned()
        } else {
            format!("'{}'", self.text.replace('\'', "''"))
        }
    }

    fn is_plain(&self) -> bool {
        matches!(self.text, "true" | "false")
            || self
                .text
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FrontmatterKey<'a> {
    path: PathBuf,
    text: &'a str,
}

impl<'a> FrontmatterKey<'a> {
    fn new(path: PathBuf, text: &'a str) -> Self {
        Self { path, text }
    }

    fn validate(&self) -> Result<()> {
        let mut characters = self.text.chars();
        let starts_lowercase = characters
            .next()
            .is_some_and(|character| character.is_ascii_lowercase());
        if starts_lowercase
            && characters.all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            Ok(())
        } else {
            Err(Error::InvalidFrontmatterKey {
                path: self.path.clone(),
                key: self.text.to_owned(),
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownBody<'a> {
    path: PathBuf,
    text: &'a str,
}

impl<'a> MarkdownBody<'a> {
    fn new(path: PathBuf, text: &'a str) -> Self {
        Self { path, text }
    }

    fn without_frontmatter(&self) -> Result<String> {
        let mut lines = self.text.lines();
        if matches!(lines.next(), Some("---")) {
            let mut body = Vec::new();
            let mut closing_seen = false;
            for line in lines.by_ref() {
                if line == "---" {
                    closing_seen = true;
                    break;
                }
            }
            for line in lines {
                if body.is_empty() && line.is_empty() {
                    continue;
                }
                body.push(line);
            }
            if closing_seen {
                return Ok(Self::join_lines(body));
            }
        }
        Ok(self.text.to_owned())
    }

    fn join_lines(lines: Vec<&str>) -> String {
        if lines.is_empty() {
            String::new()
        } else {
            let mut output = lines.join("\n");
            output.push('\n');
            output
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceMaintenanceNotes {
    text: String,
}

impl SourceMaintenanceNotes {
    fn new(text: String) -> Self {
        Self { text }
    }

    fn runtime_text(self) -> String {
        let mut offset = 0;
        for line in self.text.split_inclusive('\n') {
            if self.is_source_maintenance_heading(line) {
                let retained = self.text[..offset].trim_end();
                if retained.is_empty() {
                    return String::new();
                }
                return format!("{retained}\n");
            }
            offset += line.len();
        }
        self.text
    }

    fn is_source_maintenance_heading(&self, line: &str) -> bool {
        AtxHeading::parse(line).is_some_and(|heading| {
            heading.level() == 2 && heading.text() == "Source Maintenance Notes"
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadingNormalizer<'a> {
    text: String,
    fragment_index: usize,
    title: Option<&'a str>,
}

impl<'a> HeadingNormalizer<'a> {
    fn new(text: String, fragment_index: usize, title: Option<&'a str>) -> Self {
        Self {
            text,
            fragment_index,
            title,
        }
    }

    fn normalized(&self) -> String {
        let mut output = String::new();
        let mut first_heading_seen = false;
        let lines = self.text.lines().collect::<Vec<_>>();
        for (line_index, line) in lines.iter().enumerate() {
            if let Some(heading) = AtxHeading::parse(line) {
                let visible_text = CompositionHeadingTitle::new(heading.text()).visible_text();
                let normalized = HeadingName::new(heading.text()).normalized();
                if !first_heading_seen
                    && self.fragment_index > 0
                    && heading.level() == 1
                    && (self.title == Some(normalized.as_str())
                        || self.leading_title_duplicates_next_heading(line_index, &lines))
                {
                    first_heading_seen = true;
                    continue;
                }
                first_heading_seen = true;
                output.push_str(&heading.rendered_line(visible_text.as_str(), self.fragment_index));
            } else {
                output.push_str(line);
            }
            output.push('\n');
        }
        output
    }

    fn leading_title_duplicates_next_heading(&self, line_index: usize, lines: &[&str]) -> bool {
        let Some(heading) = AtxHeading::parse(lines[line_index]) else {
            return false;
        };
        let title = CompositionHeadingTitle::new(heading.text());
        if !title.has_composition_prefix() {
            return false;
        }
        let normalized_title = HeadingName::new(heading.text()).normalized();
        lines
            .iter()
            .skip(line_index + 1)
            .find_map(|line| AtxHeading::parse(line))
            .is_some_and(|next_heading| {
                HeadingName::new(next_heading.text()).normalized() == normalized_title
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AtxHeading<'a> {
    level: usize,
    text: &'a str,
}

impl<'a> AtxHeading<'a> {
    fn parse(line: &'a str) -> Option<Self> {
        let trimmed = line.trim_start();
        let level = trimmed
            .chars()
            .take_while(|character| *character == '#')
            .count();
        if level == 0 || level > 6 || !trimmed.chars().nth(level).is_some_and(char::is_whitespace) {
            return None;
        }
        Some(Self {
            level,
            text: trimmed[level..].trim(),
        })
    }

    fn level(&self) -> usize {
        self.level
    }

    fn text(&self) -> &str {
        self.text
    }

    fn rendered_line(&self, visible_text: &str, fragment_index: usize) -> String {
        let level = if fragment_index == 0 {
            self.level
        } else {
            (self.level + 1).min(6)
        };
        format!("{} {}", "#".repeat(level), visible_text)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownLinks<'a> {
    text: &'a str,
    module_path: &'a WorkspacePath,
    output_path: &'a WorkspacePath,
}

impl<'a> MarkdownLinks<'a> {
    fn new(text: &'a str, module_path: &'a WorkspacePath, output_path: &'a WorkspacePath) -> Self {
        Self {
            text,
            module_path,
            output_path,
        }
    }

    fn rebased(&self) -> String {
        let mut output = String::new();
        for line in self.text.lines() {
            output.push_str(
                &MarkdownLineLinks::new(line, self.module_path, self.output_path).rebased(),
            );
            output.push('\n');
        }
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownLineLinks<'a> {
    line: &'a str,
    module_path: &'a WorkspacePath,
    output_path: &'a WorkspacePath,
}

impl<'a> MarkdownLineLinks<'a> {
    fn new(line: &'a str, module_path: &'a WorkspacePath, output_path: &'a WorkspacePath) -> Self {
        Self {
            line,
            module_path,
            output_path,
        }
    }

    fn rebased(&self) -> String {
        let mut output = String::new();
        let mut remaining = self.line;
        let mut consumed = 0;
        while let Some(opening) = remaining.find("](") {
            let (before, after_opening) = remaining.split_at(opening + 2);
            output.push_str(before);
            let global_opening = consumed + opening;
            if MarkdownCodeSpans::new(&self.line[..global_opening]).inside_code_span() {
                remaining = after_opening;
                consumed += opening + 2;
                continue;
            }
            if let Some(closing) = after_opening.find(')') {
                let (target, rest) = after_opening.split_at(closing);
                output.push_str(
                    &LinkTarget::new(target, self.module_path, self.output_path).rebased(),
                );
                output.push(')');
                remaining = &rest[1..];
                consumed += opening + 2 + closing + 1;
            } else {
                output.push_str(after_opening);
                remaining = "";
            }
        }
        output.push_str(remaining);
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownCodeSpans<'a> {
    prefix: &'a str,
}

impl<'a> MarkdownCodeSpans<'a> {
    fn new(prefix: &'a str) -> Self {
        Self { prefix }
    }

    fn inside_code_span(&self) -> bool {
        self.prefix
            .chars()
            .filter(|character| *character == '`')
            .count()
            % 2
            == 1
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LinkTarget<'a> {
    text: &'a str,
    module_path: &'a WorkspacePath,
    output_path: &'a WorkspacePath,
}

impl<'a> LinkTarget<'a> {
    fn new(text: &'a str, module_path: &'a WorkspacePath, output_path: &'a WorkspacePath) -> Self {
        Self {
            text,
            module_path,
            output_path,
        }
    }

    fn rebased(&self) -> String {
        if self.is_external() {
            return self.text.to_owned();
        }
        let module_directory = self
            .module_path
            .relative_path()
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default();
        let source_target = RelativePath::new(module_directory.join(self.text)).normalized();
        let output_directory = self
            .output_path
            .relative_path()
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default();
        diff_paths(source_target, output_directory)
            .unwrap_or_else(|| PathBuf::from(self.text))
            .to_string_lossy()
            .into_owned()
    }

    fn is_external(&self) -> bool {
        self.text.starts_with('#')
            || self.text.starts_with('/')
            || self.text.contains("://")
            || self.text.starts_with("mailto:")
            || self.text.starts_with("tel:")
            || self.text.contains(' ')
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RelativePath {
    path: PathBuf,
}

impl RelativePath {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn normalized(&self) -> PathBuf {
        let mut parts = Vec::new();
        for component in self.path.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    parts.pop();
                }
                std::path::Component::Normal(part) => parts.push(part.to_owned()),
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
            }
        }
        parts.into_iter().collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownStructure {
    path: PathBuf,
    text: String,
}

impl MarkdownStructure {
    fn new(path: PathBuf, text: String) -> Self {
        Self { path, text }
    }

    fn validate(self) -> Result<String> {
        let headings = HeadingText::from_markdown(&self.text);
        headings.validate(&self.path)?;
        Ok(self.text)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadingText {
    headings: Vec<Heading>,
}

impl HeadingText {
    fn from_markdown(text: &str) -> Self {
        let mut headings = Vec::new();
        let mut active = Option::<HeadingCapture>::None;
        for event in Parser::new(text) {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    active = Some(HeadingCapture::new(level));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(capture) = active.take() {
                        headings.push(capture.into_heading());
                    }
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some(capture) = &mut active {
                        capture.push_text(text.as_ref());
                    }
                }
                _ => {}
            }
        }
        Self { headings }
    }

    fn first_title(&self) -> Option<String> {
        self.headings
            .iter()
            .find(|heading| heading.level == 1)
            .map(|heading| heading.text.clone())
    }

    fn validate(&self, path: &Path) -> Result<()> {
        let mut seen = BTreeSet::new();
        let mut previous = 0;
        let mut title_count = 0;
        for heading in &self.headings {
            if heading.level == 1 {
                title_count += 1;
            }
            let normalized = HeadingName::new(&heading.text).normalized();
            if !seen.insert(normalized.clone()) {
                return Err(Error::DuplicateHeading {
                    path: path.to_path_buf(),
                    heading: heading.text.clone(),
                });
            }
            if previous > 0 && heading.level > previous + 1 {
                return Err(Error::HeadingLevelJump {
                    path: path.to_path_buf(),
                    previous,
                    current: heading.level,
                    heading: heading.text.clone(),
                });
            }
            previous = heading.level;
        }
        if title_count <= 1 {
            Ok(())
        } else {
            Err(Error::InvalidTitleCount {
                path: path.to_path_buf(),
                count: title_count,
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadingCapture {
    level: HeadingLevel,
    text: String,
}

impl HeadingCapture {
    fn new(level: HeadingLevel) -> Self {
        Self {
            level,
            text: String::new(),
        }
    }

    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    fn into_heading(self) -> Heading {
        Heading {
            level: HeadingLevelNumber::new(self.level).as_usize(),
            text: self.text.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadingLevelNumber {
    level: HeadingLevel,
}

impl HeadingLevelNumber {
    fn new(level: HeadingLevel) -> Self {
        Self { level }
    }

    fn as_usize(&self) -> usize {
        match self.level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Heading {
    level: usize,
    text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadingName<'a> {
    text: &'a str,
}

impl<'a> HeadingName<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn normalized(&self) -> String {
        CompositionHeadingTitle::new(self.text)
            .visible_text()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompositionHeadingTitle<'a> {
    text: &'a str,
}

impl<'a> CompositionHeadingTitle<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn visible_text(&self) -> String {
        self.composition_prefix()
            .and_then(|prefix| self.text.strip_prefix(prefix))
            .unwrap_or(self.text)
            .to_owned()
    }

    fn has_composition_prefix(&self) -> bool {
        self.composition_prefix().is_some()
    }

    fn composition_prefix(&self) -> Option<&'static str> {
        [
            "Skill - ",
            "Skill – ",
            "Skill — ",
            "Skill: ",
            "Module - ",
            "Module – ",
            "Module — ",
            "Module: ",
            "Role - ",
            "Role – ",
            "Role — ",
            "Role: ",
        ]
        .into_iter()
        .find(|prefix| self.text.starts_with(prefix))
    }
}
