# Skill — engine analysis

## Rules

Use when asked to explain how an engine works: what talks to what, what state each component owns, which paths are wired, and which claims are only architectural.

Code beats diagrams. Mark every claim as hooked, stubbed, contract-only, conceptual, or stale.

## Map

List components, processes, binaries, sockets, state roots, and sandbox or deployment entrypoints. For each channel, record producer, consumer, trigger, payload type, transport, state touched, reply or event, and witness.

Follow concrete flows end to end: inbound payload, transport and contract, actor or direct-call path, durable writes, in-memory mutations, emitted reply or event, downstream effects, and breakpoint where the path stops.

Map trust boundaries separately from payloads: caller identity, file or socket permissions, runtime directory, cryptographic checks, replay or revocation handling, and which checks are implemented.

## Evidence

Prefer source files, tests, generated schemas, runtime commands, and small diagrams that answer one question. Name missing witnesses instead of filling gaps with architecture wishes.

Use status words consistently:

- hooked: code path is wired and has a witness;
- stubbed: callable but not substantively implemented;
- contract-only: types exist but no runtime path reaches them;
- conceptual: docs or design only;
- stale: docs, skills, or code names contradict present truth.

## Output

Return a compact analysis with a current-state summary, channel ledger, component notes, worked flows, trust/state summary, witness inventory, and gaps with decision questions. Use tables or diagrams only when they make the engine easier to inspect.
