# Skill — engine report

## What this skill is for

Use it when the psyche asks for an "engine situation", codebase situation,
component-size overview, interface map, or fast-readable report of how a
schema-derived/runtime engine currently works. `skills/engine-analysis.md`
is the deep architecture lens; this is the standard measurement-and-
presentation pass that makes the current code readable quickly.

The report's central readability question: do the types name the work?
Schema should name the interface; generated Rust should name the objects and
traits; handwritten code should mostly match typed input, decide, call the
next typed interface, and return typed output.

## Standard report sections

1. **Current understanding.** One compact statement of what the engine is
   now, not its history.
2. **Component ledger.** Repos, role, process/binary status, storage, and
   whether each part is hooked, stubbed, contract-only, conceptual, or
   stale. Carry a **dependency snapshot** per repo (see below): its key
   forward deps, its reverse-dep count, and its last-commit date, cited
   inline as `← N consumers, last commit MM-DD`. Reverse-dep count is the
   cheapest live-vs-dead signal; last-commit disambiguates *stale* from
   *legacy-but-shipping*.
3. **Size ledger.** Production Rust, generated Rust, test Rust, authored
   schema, generated fixtures, public type count, test count.
4. **Schema-to-code ledger.** For every `.schema`, show where it
   lowers/emits and how much Rust it generates. A `.schema` file is full
   NOTA that deserializes straight into schema-in-Rust — there is no
   separate assembled-schema artifact.
5. **Interface ledger.** Root enums, engine traits, contract traits,
   trace/help/config interfaces, and exact method signatures with file
   links.
6. **Runtime path.** One or more small diagrams showing the live call path.
7. **Witness ledger.** Tests by architectural claim and proof layer.
   Positive grep is never proof; use compile, runtime, process-boundary,
   trace, or artifact witnesses.
8. **Tooling state.** Which introspection tools were used, what worked,
   what failed, and what to install or configure next.

## Psyche-variant reports

When written for the psyche to read directly, the report is a `Psyche`
report in the `skills/reporting.md` sense: it starts from first principles
before naming gaps —

- what the engine is;
- what the schema defines;
- what Rust was generated from that schema;
- what handwritten code remains;
- what tests prove live use;
- what still needs to move into schema emission or shared runtime code.

Show the actual code for the important interfaces and paths, not only line
counts. Quote central intent by its bracket-quoted summary
(`skills/intent-log.md`).

## Measurement definitions

- **Production Rust**: `src/**/*.rs`, excluding generated `src/schema/**`.
- **Generated Rust**: checked-in `src/schema/**/*.rs`.
- **Test Rust**: `tests/**/*.rs`.
- **Authored schema**: `.schema` files (full NOTA; no separate assembled
  artifact).
- **Generated fixtures**: checked-in `*.generated.rs` or `*_generated.rs`
  outside `src/schema/**`.
- **Public type count**: a rough inventory of `pub struct`, `pub enum`, and
  `pub trait` declarations. A size signal, not a proof.
- **Test count**: `#[test]`, `#[tokio::test]`, and `#[test_case]` markers.
- **Dependency snapshot**: per repo, forward deps (LiGoldragon repos it
  imports), reverse-dep count (LiGoldragon repos importing it), and
  last-commit date. Build the graph mechanically from each `Cargo.toml`'s
  git/path deps plus `git -C <repo> log -1 --format=%cs`; cite inline as
  `← N consumers, last commit MM-DD`. Keep the extraction scripted so the
  graph is a cheap re-run, not a manual walk.

Use `tools/engine-situation` for the first size ledger:

```sh
tools/engine-situation
tools/engine-situation /git/github.com/LiGoldragon/spirit-next
```

## Tool pattern

Use tools in this order:

1. `tools/engine-situation` — quick size/type/test ledger.
2. `tokei` — language-level size when a repo has mixed languages.
3. `leta workspace add` — once per repo before LSP-backed inspection.
4. `leta files` — readable file tree with line counts.
5. `leta show <Symbol>` — exact symbol body and signatures.
6. `leta calls --from <Symbol>` — call hierarchy for a live path.
7. `leta refs <Symbol>` — use sites when checking whether an interface is
   referenced.
8. `rust-analyzer symbols < path/to/file.rs` — fallback symbol inventory
   for one file.
9. `rust-analyzer analysis-stats <repo>` — optional semantic stats; useful
   but noisy, so quote only the summary.

If `leta` loses connection, run `leta daemon restart` and repeat a narrower
query. Broad regex searches help discovery but are less reliable than
specific `leta show` and `leta calls` queries.

## Visual rules

Use multiple small diagrams, not one giant graph. Each graph answers one
question and stays around four to eight nodes. Node labels are single-line,
two to five words, with no manual `\n` or `<br/>` breaks. Put file paths,
commit IDs, and long type names in nearby prose or a table, not inside
Mermaid nodes.

## Proof discipline

Inventory commands (`rg`, `leta grep`, public type counts) tell the reader
what exists; they do not prove the architecture is live. A live-use claim
needs one witness from `skills/architectural-truth-tests.md`: a type-system
assertion, a runtime test, a trace-socket event, a process-boundary test, a
database artifact reader, or a removal-breaks-behavior witness.

## See also

- `skills/engine-analysis.md` — deeper engine analysis passes.
- `skills/architectural-truth-tests.md` — proof-of-usage ladder.
- `skills/mermaid.md` — graph readability and syntax.
