# Skill — micro-components

## Rules

One coherent capability gets one independently buildable crate or repository. If a new noun does not belong to the current crate's stated capability, split it.

A component should fit in a single fresh-agent context with its tests and architecture. If understanding it requires broad partial reads, the boundary is too large.

Components communicate through typed protocols, not shared mutable state or leaked internals. Consumers depend on contracts, not implementation crates.

Every component builds and tests on its own. If a boundary requires sibling checkouts, hidden scripts, or workspace-only state to pass, the boundary is fiction.

Stateful components normally follow the daemon + thin CLI + typed contract shape. The CLI drives the daemon path; it does not duplicate daemon state transitions.

## Dependency discipline

Cross-repository Cargo dependencies use named portable identities: published versions, tags, branches, or other named references. Do not commit sibling `path = "../..."` dependencies; they assume a local layout and fail in clones or sandboxes.

Intra-repository workspace paths are fine because they travel with the repository.

Use local overrides only as uncommitted developer configuration while iterating.

## Deployment independence

Component logic is deployment- and environment-agnostic. It functions independent of any specific host, cluster, user, network, or environment, and never depends on being run in a particular environment to work.

Deployment-, cluster-, environment-, and host-specific data — identities, hostnames, cluster, trust, and source references, secret locations — enters as configuration injected at the edge. Logic renders that data; it never bakes it in as a literal.

Test clusters and fixtures are the sole exception and live only in test code.

## Split test

Start a new component when the change introduces a distinct noun, a separate bounded vocabulary, a separate state owner, a new parser or codec, or a test surface that should run independently.

Grow the existing crate only when the new behavior is clearly part of the same capability and does not widen the crate's vocabulary or ownership.
