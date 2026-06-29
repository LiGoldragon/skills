# Skill — engine report

## Rules

Use only when the requested artifact is an engine situation report. The report makes a schema-derived or runtime engine readable quickly; it does not replace source inspection.

The central readability question is: do the types name the work?

## Shape

Include:

1. one-screen current state summary;
2. component ledger with size, responsibility, state owner, and test witness;
3. interface map: CLI, daemon, signal contract, storage, generated schemas, external dependencies;
4. channel ledger: producer, consumer, trigger, payload, transport, reply, state touched, witness;
5. worked flows for the most important operations;
6. trust and state boundaries;
7. gaps and decision questions.

Distinguish hooked, stubbed, contract-only, conceptual, and stale paths. Code and generated schemas outrank prose.

Prefer tables for ledgers and small diagrams for topology. Keep narrative sparse.

## Evidence

Use repository-local tools when available for size, symbol, and dependency ledgers. Keep commands reproducible and name the exact command used.

For dependency snapshots, include consumer count and last-change date when those facts affect whether a dependency is live or stale.

For psyche-facing variants, start from first principles and define project terms before showing ledgers. For implementer-facing variants, lead with file and command locators.
