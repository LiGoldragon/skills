# Skill — component architecture

## Rules

Keep the component split visible in code, manifests, tests, and dependencies.

A stateful component has one runtime crate and one public signal contract. Add a
meta-signal contract only when owner policy or configuration needs its own
authority boundary.

The runtime crate owns state, logic, daemon process, and thin CLI. Contract
crates own wire vocabulary, codecs, and method surfaces only. They do not own
actors, runtime state, daemon startup, Tokio orchestration, or policy decisions.

Name the public contract `signal-<component>`. Name the owner-policy contract
`meta-signal-<component>`. Repos may be named canonically; do not teach paths or
hosting locations in doctrine.

The daemon imports public and meta contract types from their contract crates. It
does not re-declare those wire shapes locally. Local schema files inside the
daemon emit runtime traits and glue over imported contract types.

Use three runtime planes inside the daemon:

- Signal admits framed requests, authenticates the caller shape, and returns the
  typed reply envelope.
- Nexus owns async mail, external effects, fanout, timeouts, and translation
  between request intent and executable work.
- SEMA owns durable single-writer state and commits the legal transition.

A root request flows Signal → Nexus → SEMA and returns on the same typed path.
Do not bypass a plane with helper calls that make admission, effects, or state
mutation invisible.

The CLI is the daemon's first client. It talks to exactly one peer: its own
daemon, through the public signal contract. It does not link daemon internals or
send owner-policy messages unless it is explicitly an owner-policy client.

The daemon's external surface is binary protocol frames carrying contract
messages. Use NOTA for authored configuration, seeds, tests, and diagnostics;
do not put NOTA between components as the transport contract.

Verb layers stay separate:

- contract verbs are the public operation vocabulary;
- runtime-plane verbs are implementation mechanics;
- sema verbs are state transitions.

Lower public verbs into runtime and sema verbs inside the daemon. Never expose
sema classification vocabulary as the public wire API.

Policy state and working state share the daemon's durable store when they must
commit atomically. Separate their authority in the wire contracts, not by
splitting the state owner.

Configuration is data. Model daemon configuration and bootstrap policy as typed
NOTA records, then encode them through the same admission and validation path as
other inputs.

Trace identity is typed and schema-derived. A trace that cannot name component,
contract operation, plane, request identity, and terminal outcome is not a useful
witness.

Tests prove the boundaries: contract round trips in contract crates, CLI-through-daemon
operation, Signal rejection before work, Nexus timeout or effect failure, SEMA
single-writer commit, and no local duplicate of imported wire types.
