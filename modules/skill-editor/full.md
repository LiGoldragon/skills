# Skill — skill editor

## What a skill file is

A skill file is tight, self-contained teaching: what an agent needs
to be effective in one scope. Agents arrive already full of training;
a skill exists to convey the workspace-specific discipline they
*can't* already know. Bloat works against this — every padded
paragraph burns context the agent needs to do the work and buries the
rule it came for.

Two scales:

- **Repo skill** — what an agent needs to be effective in *this
  repo*: project-specific intent, the repo's role, the invariants
  about *how to work* that don't fit in `ARCHITECTURE.md`.
- **Workspace skill** — a cross-cutting capability that applies
  across many repos (e.g. `autonomous-agent`, `skill-editor`).

Skills complement `AGENTS.md` (the canonical workspace contract —
what every agent must do) and `ARCHITECTURE.md` (what a system IS).
A skill is what an agent needs to be *good at* a scope.

## Brevity is load-bearing

Keep a skill as short as fully teaching its discipline allows, and no
shorter. Length itself is not a virtue, but bloat is a real cost: it
spends the agent's context and obscures the rule. Cut throat-clearing,
redundant restatements, second and third examples (keep the single
best one), and padding. Preserve every rule and its WHY — compress the
prose around it, never the normative content.

Prefer canonical positive forms over enumerating failed alternatives.
Teach the shape agents should imitate; mention retired or rejected forms
only when omitting them would create an immediate safety risk. When a
correction is necessary, keep it in prose and return quickly to the
current form.

**One capability per skill.** When a file straddles two distinct
capabilities, split it. The test: *would an agent reaching for one
section be helped or hindered by also having the other in view?* If
helped (one cross-cuts the other), it's one capability. If hindered
(they skim past unrelated rules to find what they came for), it's two.

## Naming and location

| Scale | Source path | Generated / consumed path |
|---|---|---|
| Workspace skill | `/git/github.com/LiGoldragon/skills/modules/<name>/full.md` | `.agents/skills/<name>/SKILL.md`; Claude also receives `.claude/skills/<name>/SKILL.md` |
| Repo skill | `<repo-root>/skills.md` | read in place |

One `skills.md` per repo. Workspace skill names are
lowercase-with-hyphens. Generated role packets carry the curated doctrine
bundle for normal role work; primary `skills/*.md` bodies are not a canonical
target.

## Format

Markdown, present tense throughout. Structure comes from `##`/`###`
headings only — never `---` horizontal rules (allowed solely inside a
fenced code block illustrating markdown). The opening heading marks the
file as a skill and matches its name. In source modules, the skill's
one-line description lives in the manifest/roster metadata, not in the file,
so a source skill body carries no purpose tagline:

```markdown
# Skill — <name>

## <load-bearing sections>

<the actual rules / patterns / how-to>

## See also

<at most 2-3 genuinely-useful sibling-skill pointers, by filename>
```

## The manifest entry is the description

Every workspace skill has manifest metadata that owns its output identity,
tier, target surfaces, and browsing description. Harness `SKILL.md`
frontmatter repeats the name and description only because the harness loaders
require it; source modules do not carry YAML frontmatter or purpose taglines.

Write the `[Description]` as **purpose plus trigger, in at most two
sentences, framed positively**: what decision or task the skill guides,
and when to reach for it — the line a browsing agent reads to decide
whether to open the file, not a summary of the contents and not a bare
label.

## Cross-references — minimize indirection

Prefer inlining the rule over redirecting the reader to another file.
A short "See also" of at most 2-3 genuinely-useful sibling skills is
fine; a sprawling cross-reference web is not.

When you do reference another skill, **use the repo name plus the filename or
module/skill name**, never a full HTTPS URL: "see criome's `skills.md`", "see
this workspace's `abstractions` skill". Deep file URLs silently break when
files move or get renamed; a repo-name or module-name reference stays valid
because the generated surfaces and manifests own the concrete paths. For a
repo-level pointer with no specific file, use the nix-flake form:
`github:<org>/<repo>`.

## Skills cite no reports and no intent records

A skill is permanent discipline; it must stand on its own.

- **No report links.** Reports under `reports/<role>/` are ephemeral
  working surfaces that retire as their substance migrates. A skill
  that says "see report 161" rots the moment 161 is deleted. If a
  report carries load-bearing substance, inline it (copy the rule,
  table, or example into the skill body); otherwise drop the pointer.
  If the rule isn't ready to be inlined as settled discipline, it
  stays a report and the skill doesn't pretend it's settled.
- **No Spirit / intent record citations.** Drop every record
  identifier and number. State the rule directly, in present tense —
  provenance lives in the intent log, not in the skill.

## No correction or changelog banners

Describe what IS, not what changed or what a rule supersedes. When a
section describes an outdated state, just write the correct state. The
path that led there lives in version-control history.

## What goes in a repo skill

A repo's `skills.md` holds only what is specific to this repo:

- **The repo's intent** — what it's for and what's non-negotiable.
- **The thing this repo is the canonical owner of** — the things
  only this repo decides.
- **Invariants about how to work here** — what an agent must not do,
  which conventions are load-bearing.
- **Pointers** to the repo's `ARCHITECTURE.md` and `AGENTS.md`.

It does **not** duplicate the workspace contract or language-agnostic
discipline (those live in `lore/`).

## What goes in a workspace skill

A workspace skill captures patterns that apply across multiple repos.
The test is *audience*: if a fresh agent in an unrelated future repo
would benefit, the rule belongs in primary.

**Component-specific patterns do not.** "How `nota-codec`'s encoder
emits eligible PascalCase strings as bare identifiers" is a
nota-codec rule — it goes in `nota-codec/skills.md`. The trap: on
discovering a pattern, the temptation is to write it as a primary
skill "for future agents." Resist. Ask: *is this about how we work
across the workspace, or how a specific component is built?*
Component-specific goes to the component; workspace skills stay
general.

## When to create a new repo skill

After substantive work in a repo lacking a `skills.md`, create one
before finishing the task, capturing what you just learned. Roll-out
is **incremental, not batch**: a skill written with fresh context —
having just followed the repo's invariants and found its load-bearing
files — is real; one template-stamped across many repos at once is the
smell this rule prevents. If tempted to create skills for many repos
quickly, you don't have enough context for any of them. Pick one, do
real work, then write the skill.

## Editing rules

- Edit a skill in place; don't fork or version it.
- When content turns out wrong, rewrite it; history holds the path.
- Cross-reference, don't duplicate — but inline over redirect (above).
- After a meaningful edit, commit and push immediately.

## Examples never show free functions (only `main`)

The only free function any example shows is `main`. Every other `fn`
is a method on a type (`impl T { fn ... }`) or an associated function
inside an `impl`. This is stricter than `skills/abstractions.md`
(which permits small private helpers) because **examples teach by
imitation**: an example showing `fn parse_query(...)` — even labelled
"Wrong:" — primes the next agent to write a free function. The
Wrong/Right comparison teaches the wrong shape twice.

When discussing an anti-pattern that IS a free function, name it in
**prose** and show the right shape as code:

```rust
// Anti-pattern (in prose): a free `parse_query(text: &str) -> ...`
// is a verb-without-a-noun (see `skills/abstractions.md`).
// The right shape is a method:
//
//   impl QueryParser<'_> {
//       pub fn into_query(self) -> Result<QueryOp, Error> { … }
//   }
```

**Test functions.** Rust's `#[test]` requires a free function — a
cargo constraint, not a choice. Show the test *name* in prose or a
list (`router_cannot_deliver_without_store_commit`), and show the
*body* inside an `impl Fixture { fn ... }` block when it teaches
structure, with prose noting the `#[test]` wrapper calls
`Fixture::router_cannot_deliver_without_store_commit`.

**`main`** is the one free function an example may show — Rust
requires it as the binary entry point.

**Auditing.** When editing a skill, sweep its examples before commit:

```sh
grep -nE '^\s*(pub )?(async )?fn ' <file> | grep -vE 'fn main\b'
```

Every match in an example block belongs inside `impl` or is removed.

## See also

- `autonomous-agent.md` — acting on routine obstacles without asking.
- `naming.md` — naming conventions used inside skill files.
- `abstractions.md` — verb-belongs-to-noun, behind the example rule.
