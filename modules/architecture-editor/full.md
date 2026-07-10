# Skill — architecture editor

## Architecture files state system shape and direction

`ARCHITECTURE.md` is a repository's primary guidance surface, read on entry before code. It records durable structure: components, boundaries, data ownership, wire vocabulary, constraints, invariants, and operational shape. It also carries the repository's durable direction — what the psyche wants the project to be and why — when that direction is not better expressed as a code stub with an explanatory comment. It is not an artifact index, changelog, transcript, or decision log.

Edit the nearest architecture file that owns the component or subsystem. If no owner exists and the architecture or direction is durable, create one next to the code or repo it governs.

## Put each statement in its owning surface

- Agent operating rules go in `AGENTS.md`.
- Architecture, invariants, and durable project direction go in `ARCHITECTURE.md` (or a code stub with an explanatory comment).
- User-facing overview and setup go in `README.md`.
- Speculative future project ideas go in `IDEAS.md` until they become accepted
  architecture or active work.
- Required hacks and workaround instructions go in `NON_IDEAL_AGENTS.md` so they
  remain visible as debt, not architecture.
- Work items go in the tracker.
- Historical evidence stays in artifacts only while it is an active pickup surface.

Do not reference transient artifacts from architecture. Move the architectural fact into the file and drop the external dependency.

## Preferred shape

Use headings that match the component, not a fixed template. Common sections are:

- overview;
- components and boundaries;
- wire vocabulary;
- state and ownership;
- constraints;
- invariants;
- code map.

Keep sections short. Use bullets for constraints and invariants. Avoid prose that only says the file is important.

## Keep architecture prose, not code

Architecture files hold only prose architectural design and constraints. Do not
put implementation code or pseudo-code in them. The sole exception is example
surface syntax of a language the document is actually designing.

## Write constraints as test seeds

A good architecture statement can become a test, review check, or design gate. Prefer precise constraints: ownership, forbidden dependencies, direction of calls, state transitions, persistence boundaries, and protocol compatibility.

## Carry uncertainty without cementing it

If uncertainty must be preserved, mark it as undecided and keep it separate from accepted architecture. Name the decision needed and the evidence that would settle it. Move speculative future projects to `IDEAS.md` when they are not accepted structure yet. Remove uncertainty sections once a decision lands.

State only accepted structure. Include extension points only when the accepted architecture depends on them.

## Edit when structure changes

Update architecture when a change alters component boundaries, durable data, protocol vocabulary, deployment shape, ownership, invariants, or constraints. Do not edit architecture for ordinary implementation churn that leaves shape unchanged.

## Retire legacy paths

When a replacement interface works, remove architecture that teaches the old path. Preserve only compatibility constraints that remain true.
