# Skill — contract repos

## Rules

A contract repo defines a wire surface. It owns operation types, reply types,
codecs, round-trip tests, and compatibility discipline. It does not own daemon
state, actors, persistence, policy decisions, or runtime orchestration.

Create a contract repo when more than one crate must speak the same typed
vocabulary, when build churn needs isolation, or when authority boundaries must
be visible at dependency level.

Name a component's ordinary public contract `signal-<component>`. Name an owner
policy contract `meta-signal-<component>`. Use another canonical name only when
the contract is an independent base protocol rather than a component surface.

Public operations use contract-local verbs. They name what the peer asks for,
not how the daemon lowers the request internally.

Keep sema classification vocabulary off the public wire. `Append`, `Commit`,
`Transition`, and similar state-machine verbs belong inside the daemon unless
those are truly the user's contract operations.

Lowering is daemon logic: public operation → admission result → runtime work →
state transition → public reply. Do not move that translation into the contract
crate.

Replies state the domain outcome callers can act on. Avoid generic
`Success`/`Error` shells when the caller needs created identity, accepted work,
validation failure, conflict, timeout, or authorization failure.

Use a small stable operation spine. Add a verb only when it changes caller
semantics, authority, durability, or reply shape; do not mirror internal helper
steps as public operations.

Version the wire deliberately. A breaking field, enum, or operation change is a
contract change even when the daemon can adapt locally.

Round-trip examples come first. Keep one compact canonical example per operation
kind and assert binary and human-readable encodings when both are supported.

NOTA is an authored data shape and diagnostic surface, not a substitute for the
binary wire protocol between components.

Contract tests prove no runtime leaked in: generated wire compiles alone,
round-trips pass, public methods expose the intended operations, and the crate
has no daemon runtime dependency.

Common mistakes to delete: daemon imports in contract crates, public verbs named
after sema internals, duplicate local wire types in the daemon, catch-all string
errors, and contract examples that are not asserted by tests.
