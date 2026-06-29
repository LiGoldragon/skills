# Skill — architecture editor

## Architecture files state system shape

`ARCHITECTURE.md` records durable structure: components, boundaries, data ownership, wire vocabulary, constraints, invariants, and operational shape. It is not a report index, changelog, roadmap, transcript, or decision log.

Edit the nearest architecture file that owns the component or subsystem. If no owner exists and the architecture is durable, create one next to the code or repo it governs.

## Put each statement in its owning surface

- Agent operating rules go in `AGENTS.md`.
- Architecture and invariants go in `ARCHITECTURE.md`.
- User-facing overview and setup go in `README.md`.
- Work items go in the tracker.
- Historical evidence stays in artifacts only while it is an active pickup surface.

Do not reference reports from architecture. Move the architectural fact into the file and drop the report dependency.

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

## Write constraints as test seeds

A good architecture statement can become a test, review check, or design gate. Prefer precise constraints: ownership, forbidden dependencies, direction of calls, state transitions, persistence boundaries, and protocol compatibility.

## Carry uncertainty without cementing it

If uncertainty must be preserved, mark it as undecided and keep it separate from accepted architecture. Name the decision needed and the evidence that would settle it. Remove uncertainty sections once a decision lands.

Avoid temporal roadmap language. Do not promise planned features in architecture unless the accepted architecture already depends on that extension point.

## Edit when structure changes

Update architecture when a change alters component boundaries, durable data, protocol vocabulary, deployment shape, ownership, invariants, or constraints. Do not edit architecture for ordinary implementation churn that leaves shape unchanged.

## Retire legacy paths

When a replacement interface works, remove architecture that teaches the old path. Preserve only compatibility constraints that remain true.
