# Skill — architectural truth tests

## Rules

Use architectural tests when a constraint says one component, layer, actor, or
wire surface must be the path another component uses.

Behavior tests prove the visible outcome. Architectural tests prove the required
path produced that outcome.

For every rule shaped like "A uses B to do C," name the witness B necessarily
leaves and a bypass cannot counterfeit. Then test both: the real path leaves the
witness, and the tempting shortcut fails.

Positive source search is not proof of use. Text presence does not prove a type,
actor, daemon, schema chain, wire frame, or storage layer is live. Use source
search only as a negative guard for retired or forbidden surfaces.

Choose the cheapest sufficient witness:

- static witness: dependency metadata, type identity, trait bounds, compile-fail
  tests, re-export checks;
- runtime witness: integration path, actor trace, recorder hook, process
  boundary, property test;
- artifact witness: golden storage, golden wire bytes, chained write/read checks,
  mutation or removed-code failure.

Default to a runtime witness when the claim is about an execution path. Use a
static witness for purely structural claims. Use an artifact witness when the
claim is durability, compatibility, or "removing this breaks behavior."

A constraint that does not suggest a witness is not precise enough. Rewrite it
until it names the component, operation, boundary, and bypass that must fail.

Name tests after the invariant, not the implementation detail:
`request_cannot_commit_without_store_actor`,
`client_cannot_round_trip_without_contract_frame`,
`query_cannot_touch_writer`.

Actor-ordering constraints start with actor traces. A trace proves the mailbox
path and happens-before relation a direct call skips. Durable artifacts can add
stronger proof, but they do not replace the path witness.

Contract boundaries need negative witnesses: the contract crate compiles without
runtime imports, duplicate local wire types fail review or compile checks, and
round trips use the public codec.

Schema-derived runtimes use schema-emitted objects as witnesses. Do not invent a
test-only command enum or string log to prove a generated trait, root type, or
wire object is used.

Vocabulary widening needs an end-to-end boundary test for a newly admitted
variant. A unit codec round trip is not enough when persistence, daemon routing,
or client rendering may still use the older vocabulary.

Pair-rule audits cover valid and forbidden shapes in the same scope. If the
valid pattern is "data-bearing noun with methods," sweep the adjacent forbidden
pattern, such as empty marker nouns or free functions, before concluding.

A good architecture test fails for the shortcut an agent is most likely to write
while still allowing the intended path to pass.
