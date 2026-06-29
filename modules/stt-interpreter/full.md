# Skill — speech-to-text interpreter

## What this skill is for

The psyche often dictates prompts through a speech-to-text tool that
mis-transcribes workspace-specific words (repo names, library names,
language tokens) — they aren't in its dictionary or they collide with
common English. When a prompt contains a phonetic near-miss for a
known workspace word, **guess the intended word**, act on the guess,
and ask only if the guess turns out wrong.

The tool itself lives in the **CriomOS-home** repo; its `skills.md` /
`ARCHITECTURE.md` are canonical for the specific tool, model, and
config. Don't name the vendor or model here — that drifts.

## How to read STT output

1. **Read for intent first.** STT errors are at the word level, not
   the structural level — grammar survives even when nouns get
   mangled.
2. **When a word looks suspicious**, check the table below before
   treating it as English.
3. **When a real English word appears where a project name fits**
   (e.g. "ASCII" where you'd expect a project name), default to the
   project — STT often "corrects" a real name into a familiar word.
4. **Don't ask the user to spell things out** — STT auto-corrects
   spelled-out letters anyway. Instead look up the canonical spelling
   on the filesystem.

## Canonical spellings live on the filesystem

The canonical spelling of a repo **is the directory name** under
`/git/github.com/<org>/<repo>/`; the `repos/` index is symlinks to
those paths. Read the directory name; never reconstruct spelling from
the spoken form. This applies any time you're unsure how a project
name is spelled.

## Table: transcribed form → canonical

The **transcribed form** is what STT might emit; the **canonical
form** is the workspace's name. When a transcribed form doesn't make
sense as English, try the canonical.

### Repositories (sema-ecosystem)

| Heard / transcribed as | Canonical |
|---|---|
| "Creom" / "Cree-ome" / "Krio" / "Criom" / "Cry-om" | **Criome** — universal validator/coordinator daemon at the center of the sema-ecosystem; long-term replaces Git, code editor, SSH, web server, etc. Written name `Criome` (capital C, trailing `e`); filesystem path lowercase `criome`. |
| "Sema" / "Seema" | the canonical-store repo |
| "Nota" / "Nodda" | the text-format repo |
| "Nexus" | the request-language repo |
| "Signal" | the binary-IR repo (overlaps the English word — context decides) |
| "Arca" / "Ar-ka" | the content-addressed-store repo |
| "Forge" | the executor repo (overlaps the English word — context decides) |
| "Prism" | the projector repo (overlaps the English word — context decides) |
| "Hexis" / "Hex-is" | the host-config-reconciler repo |
| "Lore" | the docs repo (overlaps the English word — context decides) |
| "Workspace" | the workspace meta-repo |
| "ASCII" / "asci" / "askii" | the retired `aski` repo; pronounced like the encoding |

### Repositories (system / tooling layer)

| Heard / transcribed as | Canonical |
|---|---|
| "Crio-OS" / "Creom OS" / "Krio-O-S" | the host-OS repo |
| "Crio home" / "Krio-home" | the home-manager repo |
| "Crio Emacs" | the Emacs config repo |
| "Lojix" / "Logix" / "Logics" / "Logic CLI" | the deploy CLI repo |
| "Horizon RS" / "Horizon Rust" | the horizon Rust crate repo |
| "Goldragon" / "Gold dragon" | the cluster-proposal repo |
| "Gascity" / "Gas city" / "Gas-city" | the orchestration repo (LiGoldragon fork) |
| "Menchie" / "Men-chee" / "Menchi" / "Menci" / "Menschie" / "Menchy" / "Mentchy" | **Mentci** — the psyche-facing approval component and repo family. Product/interface name `Mentci`; filesystem paths lowercase `mentci-*` such as `mentci-lib` and `mentci-egui`. |
| "The Book of Sol" / "Sol" / "Sole" | the writing-project repo |
| "Persona" | the operator's current scaffolding repo |

### Programming-language terms

| Heard / transcribed as | Canonical |
|---|---|
| "Rust" / "rest" | the Rust language |
| "Nix" / "Nicks" / "Nyx" | the Nix language / build system |
| "Cargo" | Rust's build tool / dependency manager |
| "Rkyv" / "ar-keev" / "Archive" (Rust context) | rkyv (binary serialization crate) |
| "Ractor" / "Reactor" (Rust+actor context) | ractor (the actor framework) |
| "Tokio" / "Toki-yo" | tokio (async runtime) |
| "Serde" / "Sir-day" | serde (serialization) |
| "Thiserror" / "This error" | thiserror (error-derive crate) |
| "Anyhow" / "Eyre" | anyhow / eyre (forbidden in this workspace's Rust crates) |
| "Crane" / "Crain" | crane (Nix Rust packaging) |
| "Fenix" / "Phoenix" (Nix-toolchain context) | fenix (Rust toolchain in Nix) |
| "Blueprint" | numtide/blueprint (Nix flake layout helper) |
| "Flake" / "Flakes" | Nix flakes |
| "Nixpkgs" / "Nix packages" / "Nix peekages" | nixpkgs |
| "Home-manager" / "Home manager" | home-manager (Nix HM) |

### VCS and tooling

| Heard / transcribed as | Canonical |
|---|---|
| "Jujutsu" / "Jujitsu" / "JJ" / "Jay-jay" | jj (version control) |
| "Dolt" / "Dolt SQL" | Dolt (git-for-data) |
| "Beads" / "BD" / "Bee-dee" | bd / beads (issue tracker) |
| "Anna's archive" / "Annas" | annas (Anna's Archive CLI) |
| "Linkup" | linkup (search CLI) |
| "Substack" | substack (publishing CLI) |

### Other workspace terms

| Heard / transcribed as | Canonical |
|---|---|
| "Ouranos" / "Uranus" | the user's primary node name |
| "Prom" / "Prometheus" | the binary cache server |
| "Polecat" / "Mayor" / "Refinery" / "Witness" | gas-city role names from the gastown pack |
| "Operator" / "Designer" | lock-coordinated agent roles (Codex / Claude) |
| "ESSENCE" / "Intention" | the workspace intent doc, `~/primary/ESSENCE.md` |
| "Lock" / "Lockfile" | next to "operator"/"designer" means the coordination protocol; otherwise normal English |

## Caveat — ASCII / aski lineage

`aski` formally disclaims being an ancestor of nota/nexus (shared
surface features are coincidence, not lineage). The psyche's lived
sense is that aski's design instincts — delimiter-first, no keywords,
position defines meaning, names are meaningful, no opaque strings —
inspired the current work. Honor the lived sense in conversation;
flag the formal disclaimer only when load-bearing.

## When the table doesn't help

1. List repos: `ls ~/primary/repos/` or
   `~/primary/RECENT-REPOSITORIES.md`. Most workspace-specific words
   are repo names; the listing is exhaustive.
2. List CLI tools: `compgen -c | sort -u`. Some tools have
   phonetic-misheard names (`bd`, `jj`, `gh`).
3. If a candidate emerges, act on it and note the match in your reply
   so the user can correct a wrong guess.
4. Only if no candidate emerges, ask — framing the question by
   listing the closest matches you considered.

## Keeping this skill current

The table is workspace-state; it grows as new repos and libraries
land. When you hit a new STT mishearing not in the table, add the row
before continuing, then commit and push.

## See also

- `autonomous-agent.md` — when to act on a guess vs ask.
- CriomOS-home's `skills.md` — the STT tool's setup (model, config,
  where it runs).
