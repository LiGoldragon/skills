# Skill — architectural truth tests

## What this skill is for

Apply it when an architecture constraint says *"component A uses
component B to do C"* and you're writing tests for C.

Behavior tests prove C succeeds. Architectural-truth tests prove **B
was the path** — not a local shortcut producing the same output.
Without them, an agent can ship code that satisfies behavior while
routing around the intended component, and no test fires. The
discipline applies to every architectural assertion: wire contracts,
storage layers, actor protocols, deploy chains.

## The principle

If a rule says "A uses B to do C", the tests must make bypassing B
fail even when C's visible output still succeeds. Treat architecture
as a contract with **observable witnesses**: dependency graph, type
identity, actor messages, storage table identity, wire format, state
transitions, negative compile/runtime cases.

Prefer weird tests over trusting implementation prose. Agents write
code that looks aligned while secretly reimplementing the component
next door; a correct test forces the intended path to be the only
passing path. Every architectural constraint gets at least one
**positive** witness (the intended component is used) and one
**negative** witness (the tempting shortcut fails).

## No positive grep as proof

A positive architectural witness must exercise the real path. A source
scan ("this string exists") only proves text is present — never that a
daemon, schema chain, actor path, wire frame, or deploy path uses it.
So `grep -R "SemaWriteInput" src` is not architecture proof.

Grep is legitimate ONLY as a **negative guard** for retired/forbidden
surfaces: `! grep -R "NexusMail" src tests`, `! grep -R "git+file://"
flake.nix`. Grep proves absence; it never proves live use. Positive
proof must compile, execute, round-trip, or observe the real boundary.

## Proof-of-usage ladder — choose cheapest sufficient

Every claim about USAGE picks the cheapest witness strong enough.
Don't over-witness; don't under-witness. Strength and cost both rise
from Layer 1 to Layer 3.

### Layer 1 — STATIC (compile-time type-system reference)

Proves the code exists AND the type system links it. Does NOT prove it
runs at runtime — a method can be `use`d and linked without ever being
called (dead code the compiler keeps).

Witnesses (all free): `use T` compiles;
`static_assertions::assert_impl_all!(T: Trait)`; `let _: T = expr;` in
a test; `trybuild` compile-fail on a shortcut; `cargo metadata`
dependency assertion; type-alias / re-export check.

**Grep is NOT a Layer 1 witness** — it traverses strings, not the
type system. The type-system witness uses `use`, `assert_impl_all!`,
or a constructor call that forces the compiler to resolve the
reference. Strength: LOW-MEDIUM.

### Layer 2 — RUNTIME (execution path taken)

Proves the code actually runs under test/production and the call path
is taken on specific inputs. Witnesses: unit test calling the function
(cheap); integration test through the wire (medium); actor-trace
assertion via recorder actor (medium); process-boundary test spawning
the real binary + round-trip (expensive); property test (medium); code
coverage (expensive).

Strength: STRONG. Proves execution at the witness's call site, not
that production exercises the path — only the test that ran. A Layer 2
witness proves the path is EXECUTABLE; for most architectural claims
that is sufficient, and Layer 2 is the default.

#### Testing-trace — the canonical Layer 2 witness for engine-trait usage

The testing-build logging socket is the workspace canonical Layer 2
runtime witness for engine-trait usage. When the testing-build is
active and logs flow back through the CLI's translation surface, the
agent observes proof that Signal / Nexus / SEMA engine traits are
actually called by the runtime — not just present in source.

The trace hook is part of the trait's **schema-emitted** code: the
recorder is the trait's own override, not a hand-written
instrumentation layer beside it. Default no-op bodies; live bodies
under a `testing-trace` Cargo feature that routes records to a trace
socket. An integration test reads the emitted records and asserts the
engine-trait method was called for a fixture; a process-boundary test
proves the same round-trip across CLI to daemon to trace socket.

The witness can't be counterfeited without re-emitting through schema:
a bypass that re-implements the engine outside the trait loses the
trace surface as a consequence of leaving the trait. Generalises to
every component whose runtime engines emit through the schema
toolchain — records on the trace socket mean the engine-trait method
was used; absence on a call path means the path is dead or bypassed.

### Layer 3 — BEHAVIORAL (removal breaks observable behavior)

Proves this code carries an observable effect — removing it changes
externally visible behavior. Witnesses: mutation testing
(`cargo-mutants`, very expensive); manual removed-code test
(deliberately break, assert failure — cheap per-instance);
negative-presence test on output; backward-compat test against
checked-in fixtures; differential testing against a known-good.

Strength: STRONGEST. If removing X breaks an observable behavior the
test asserts, X is genuinely USED, not just present. Reserved for
high-stakes invariants where dead-code-passing-as-live is costly; most
claims don't need it.

### The discipline

For each claim, ask: structural reference, runtime execution, or
observable behavior? Pick the cheapest layer whose witness shape
matches. Default Layer 2 (a path being EXECUTABLE — cheapest witness
is an integration test through the wire). Layer 1 for purely
structural claims (the trait surface exposes only these methods).
Layer 3 when "removing this breaks behavior" is the real claim. The
forbidden positive grep sits BELOW Layer 1 — it claims structural
reference but doesn't prove it.

### Worked examples

**"Type T is emitted by the schema chain"** (Layer 1) — compiles only
if the emitter emitted the type with the expected name and
constructor; grep can't tell "the name appears" from "the type is
emitted with this exact shape":

```rust
#[test]
fn sema_write_input_type_emitted() {
    use my_crate::schema::lib::SemaWriteInput;
    let _: SemaWriteInput = SemaWriteInput::default();
}
```

**"Runtime X uses trait method Y"** (Layer 2 via recorder) — the
witness sinks call records, proving Nexus actually called the trait
method during execution:

```rust
#[test]
fn nexus_calls_sema_engine_apply_via_trait() {
    let recorded = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let fake_sema = FakeSemaEngine::new(recorded.clone());
    let mut nexus = Nexus::with_sema_engine(fake_sema);
    let _ = nexus.execute(test_nexus_input());
    assert!(recorded.lock().unwrap().iter()
        .any(|call| matches!(call, RecordedCall::SemaApply(_))));
}
```

**"Daemon round-trips through the wire"** (Layer 2 via process
boundary) — the full real binary runs, the wire round-trips, the reply
shape proves end-to-end execution. The strongest cheap Layer 2 witness
for "the daemon's runtime path is live":

```rust
#[test]
fn daemon_round_trip_through_socket() {
    let fixture = DaemonFixture::start();
    let reply = fixture.invoke_cli("(Record ...)");
    assert!(reply.contains("RecordAccepted"));
}
```

## Constraints first

The `Constraints` section of a component `ARCHITECTURE.md` is the seed
list. Write each constraint as a short plain sentence, then name at
least one test after it.

| Constraint | Test name |
|---|---|
| `mind` CLI accepts exactly one NOTA record | `mind_cli_accepts_exactly_one_nota_record` |
| `mind` CLI sends Signal frames to the daemon | `mind_cli_cannot_reply_without_daemon_signal_frame` |
| queries never send write intents | `query_path_cannot_touch_sema_writer` |
| daemon owns `mind.redb` | `mind_redb_cannot_be_opened_by_cli` |
| contract crates contain no runtime actors | `contract_crate_cannot_spawn_actor_runtime` |

A constraint that doesn't suggest a witness isn't precise enough yet.
Rewrite it until it names the component, the operation, and the
boundary that must not be bypassed.

## The shape

The hardest step is naming the **witness** — the artifact B
necessarily produces and a bypass necessarily doesn't. From the
constraint "A uses B to do C", name that witness, then write a
positive test (C succeeds AND witness present) and a negative test
(the local shortcut is compiled-out or runtime-rejected). The
witnesses are the load-bearing design move; the tests are mechanical
once the witness is named.

## Witness catalogue

| Witness | Catches |
|---|---|
| `cargo metadata` dependency assertions | wrong repo reached across a boundary |
| `compile-fail` tests (`trybuild`) | local duplicate types, string shortcuts, missing trait contracts |
| Fake actor handles | direct method calls disguised as actor code |
| Typed event traces (recorder actor) | wrong ordering of effects (push-before-commit) |
| Actor topology manifest | missing actors, collapsed phases, unsupervised children |
| Actor trace pattern | request bypasses a required actor plane |
| Forbidden actor-edge trace | query writes, CLI opens database, domain actor bypasses store actor |
| redb fixture files (golden) | schema/version lies; missing table writes |
| rkyv byte fixtures (golden) | incompatible wire or disk encoding |
| Nix-chained derivations | runtime memory faking what should be filesystem |
| Process-tree witnesses (`/proc/<pid>/maps`, `lsof`) | claimed-open files that aren't open |
| Length-prefixed-frame parser on the wire | text/JSON snuck into a Signal channel |
| Schema-version golden | undocumented schema drift |
| Legacy-surface absence witness | lock-file / BEADS reinvestment in new components |
| Network namespace test | hidden cross-machine calls |

## Pair-rule sweeps — valid patterns and adjacent anti-patterns

When a discipline has a positive shape AND a negative shape (this
pattern is valid; this look-alike is forbidden), audit sweeps must
cover **both shapes in the same scope**. A valid pattern coexists with
an anti-pattern nearby; an audit that grepped only one shape misses
the other.

The recurring failure: an audit looks for the load-bearing valid
pattern (single-field wrappers, real actors, data-bearing nouns),
confirms each pays its way, and writes "no violations found." The
adjacent anti-pattern in the same file — a zero-sized method holder, a
fake actor, a free function dressed as an associated function — never
gets swept because the scope was "valid patterns" not "the whole
rule." Example: `^struct \w+ \{$` finds single-field wrappers but
skips unit structs; the ZST namespaces need a separate `^struct
\w+;$`. Both sweeps land in the same audit pass.

| Positive shape | Negative shape (sweep simultaneously) |
|---|---|
| `struct Wrapper { field: T }` with methods | `struct Wrapper;` with associated functions (ZST namespace) |
| Real data-bearing actor noun | Empty `pub struct ActorMarker;` ZST masquerading as a noun |
| Method on `impl Type` block | Free function outside any `impl` |
| Trait with multiple impls | One-impl trait (extension trait disguising a free function) |
| Closed enum match arm | String predicate or boolean flag soup branch |

The audit conclusion names what was found in BOTH sweeps: "Sweep A
(valid, N instances) all legitimate. Sweep B (anti-pattern, M
instances) — N violations." A report naming only Sweep A leaves Sweep
B as a cleanup the auditor invited by omitting it.

## Actor trace first, artifacts later

For actor-system ordering constraints, start with the mailbox path. An
actor trace is the first witness: it proves the required actors saw
the message and records the happens-before relation a direct call or
shortcut would skip.

Don't wait for durable storage to exist before testing an ordering
claim. If the component has only in-memory state, write the
actor-trace witness now — e.g.
`router_cannot_emit_delivery_before_commit` records the commit event
before any delivery event in the trace stream; the test fails if
delivery appears first. When the durable substrate lands, add the
stronger stateful or Nix-chained artifact witness on top; it proves
the redb/table write happened before delivery across boundaries but
does not replace the actor trace, which still proves the intended
mailbox path was used.

## Schema-chain witnesses use schema objects

For schema-derived runtimes, witnesses must be schema-emitted objects
flowing through schema-type traits. Every component runtime in the
triad architecture conducts core logic through schema-emitted traits —
`SignalEngine`, `NexusEngine`, `SemaEngine` — whose methods take and
return root types of the concerned interfaces; the traits ARE the
witness surface. Do not invent a test-only enum to stand in for the
runtime language being proved.

For a Signal -> Nexus -> SEMA chain, build the witness from generated
objects:

- `MailLedgerEvent` for hookable Signal/Nexus lifecycle events.
- `NexusInput` / `NexusOutput` for execution-plane ingress and egress.
- `SemaInput` / `SemaOutput` for state-plane operations and replies.

The SEMA engine contract is especially strict: the operation method
takes a SEMA schema object and returns a SEMA schema object. A test
calling a store or engine with a primitive, helper enum, or test-local
command type proves the wrong surface even when visible behavior
succeeds. The strongest in-process shape: Signal admission produces a
typed accepted object from generated `Input`; Nexus is invoked through
generated `NexusEngine` (or per-root dispatch traits) on
`NexusInput`/`NexusOutput`; SEMA through generated `SemaEngine` on
`SemaInput`/`SemaOutput`; rejections and lifecycle events are
generated values like `Output::Rejected(SignalRejection)` and
`MailLedgerEvent`, never hand-written test enums or string logs. Name
the test after the chain invariant, e.g.
`schema_emitted_traits_drive_the_full_plane_chain`.

## Live boundary witness for vocabulary widening

When a closed-vocabulary enum widens (Certainty's three variants
become Magnitude's seven; ItemPriority collapses onto Magnitude), the
load-bearing witness is **a live test round-tripping a newly-admitted
variant end-to-end through the actual wire path** — Record it, Observe
it back. Unit tests on the type prove rkyv round-trips in isolation;
they do not prove the wider vocabulary persists through the daemon and
reads back through the client.

```rust
#[test]
fn client_accepts_high_magnitude_and_observes_it_back() {
    let fixture = StoreFixture::new("high-magnitude");
    fixture.reply_text(
        "(Record ([(Information Documentation)] Decision [high magnitude witness] High Minimum Zero []))",
    ).expect("high-magnitude entry persisted");
    let reply = fixture.reply_text(
        "(Observe ((Full [(Information Documentation)]) Any Any Any (Some Decision) (Exact Zero) (AtLeastCertainty High) Any))",
    )
        .expect("records observed");
    assert!(reply.contains("High"));
}
```

The witness fails if the daemon silently downgrades the variant, if
the codec round-trips locally but the daemon never sees it, or if the
observe reply re-renders an older form. Every vocabulary-widening
pilot ships one such test in `tests/boundary.rs`.

## Nix-chained tests — the strongest witness

When a rule says *"this writes to the database"*, the strongest
witness **separates the write from the read across two Nix
derivations**. The first runs the code-under-test and **emits the
database file as its output**; the second **reads the file with the
authoritative reader** and asserts content. Nothing in-process can
fake the chain: if the database wasn't actually written, the second
derivation has nothing to read.

Why Nix specifically:

- **Pure environment** — no host home-dir carry-over, no `tmpfs`
  collusion; the writer's output is the only path to the reader.
- **Reproducible** — the chain runs identically everywhere; the chain
  *is* the test, not a flaky integration script.
- **Content-addressed output** — `state.redb` becomes
  `/nix/store/<hash>-state.redb`; the hash changes if any byte
  changes, so drift surfaces as a hash mismatch.
- **Reader can't be the writer's mock** — it depends only on the file
  artifact, not the writer's source, so it can't accept in-memory
  state.

```nix
{
  outputs = { self, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        checks = {
          # Step A: run the write code, output the redb file.
          message-stack-write = pkgs.runCommand "message-stack-write.redb" {
            buildInputs = [ self.packages.${system}.message-cli
                            self.packages.${system}.persona-router-daemon ];
          } ''
            export STATE_DIR=$out
            mkdir -p $STATE_DIR
            persona-router-daemon $STATE_DIR/router.sock &
            ROUTER_PID=$!
            sleep 1
            message designer "stack test message" --socket $STATE_DIR/router.sock
            sleep 1
            kill $ROUTER_PID
            test -f $STATE_DIR/persona.redb
          '';

          # Step B: read with the authoritative reader; assert the message landed.
          message-stack-read = pkgs.runCommand "message-stack-read" {
            buildInputs = [ self.packages.${system}.persona-router-reader ];
          } ''
            persona-router-reader \
              --db ${self.checks.${system}.message-stack-write}/persona.redb \
              --table messages --expect "stack test message"
            touch $out
          '';
        };
      });
}
```

The chain forces the writer to actually create the file (or step A
fails), the reader to actually find the message in the typed table (or
step B fails), and the reader to be a separate binary depending only
on the file artifact. If the agent shortcuts the durable router-owned
store and keeps state in memory, step A produces an empty file and
`message-stack-read` fails on the witness file.

## Examples (persona messaging stack)

| Constraint | Architectural-truth test |
|---|---|
| Component-owned Sema tables store Signal contract types | Insert and read a Signal contract record through the owning component's typed Sema table; no local duplicate type can satisfy the value type. |
| Router commits before delivery | Fake store + harness actors; assert the router emits `CommitMessage` before any `DeliverToHarness`. |
| Router does not own terminal bytes | `cargo metadata` test fails if `persona-router/Cargo.toml` depends on `persona-terminal`. |
| Signal is the component wire | Integration test sends a length-prefixed `signal_core::Frame`; NOTA strings on the socket are rejected. |
| No private durable queue | Restart router after a queued message; it survives only if committed through the router-owned Sema table. (Nix-chained: writer queues + crashes; reader opens the redb and looks for it.) |
| Sema schema guard is real | Existing redb with no schema version hard-fails on `open_with_schema`; fresh file writes the version; mismatch hard-fails. |
| Guard facts are pushed | Fake system actor sends focus/prompt facts; router retries only on pushed observation, never on a timer. (`tokio-test` clock-pause shows zero retries during paused time.) |
| Prompt guard blocks injection | Nonempty prompt fact → `DeliveryBlocked(PromptOccupied)` and zero terminal-input frames. |
| Focus guard blocks injection | Focused target → `DeliveryBlocked(HumanOwnsFocus)` and zero terminal-input frames. |
| Actor model is real | Router test communicates through actor handles/mailboxes only; direct method calls aren't public API (compile-fail test against the bypass). |
| Actor density is real | Runtime topology contains the named phase actors from the manifest; a request trace passes through each in order. |
| Actor handler does not block | Failure-injection actor holds an IO/command/clock plane; domain actor mailbox stays responsive while that plane waits. |
| Actor nouns carry data | Static witness rejects public empty actor marker types; adapter ZSTs are private framework glue only. |

## Rule of thumb — the test name pattern

If the rule is *"X must go through Y"*, name a test
`x_cannot_happen_without_y`, then ensure `Y` leaves a typed witness a
bypass cannot counterfeit. The test does the action, then checks the
witness exists. Examples:

- `message_cannot_persist_without_component_owned_sema`
- `router_cannot_deliver_without_commit`
- `injection_cannot_happen_without_focus_observation`
- `query_cannot_touch_sema_writer`
- `cli_cannot_open_component_database`
- `handler_cannot_block_mailbox`
- `claim_normalizer_cannot_be_empty_marker`

When the body needs to teach structure, put it on a fixture method;
the `#[test]` wrapper only calls the fixture.

## Actor-density tests

When an architecture says a component is actor-based, behavior tests
aren't enough — the tests must prove the expected actor planes exist
and were used. Two witness families: a topology dump (from the running
actor tree) checked against the manifest, and an `ActorTrace` (from a
typed request) checked against the required actor sequence.

| Rule | Witness |
|---|---|
| Actor exists | topology dump contains the actor path |
| Actor is supervised | topology dump shows the expected parent |
| Request used actor | trace contains actor received/replied events |
| Query stayed read-only | trace contains read actors, excludes writer actors |
| Mutation used store actor | trace contains writer, event-append, and commit actors |
| Handler did not block | while one plane actor waits, a sibling request actor still replies |
| No hidden shared lock | static scan rejects `Arc<Mutex<...>>` ownership between actors |
| Actor noun carries qualities | compile-time/static witness rejects public empty actor structs |

The failure mode to catch is "one actor with helper methods." If the
architecture names `ClaimNormalizeActor` and `ClaimConflictActor`, a
single `ClaimActor` with private helpers is not equivalent; the
topology and trace tests should fail that implementation.

## When to use which witness

| Rule shape | Use |
|---|---|
| "Component A depends on B" | `cargo metadata` test |
| "Type X is the wire form" | rkyv byte fixture or compile-fail on alternatives |
| "Effects happen in order" | typed event trace via recorder actor |
| "State is durable across restarts" | nix-chained writer/reader derivations |
| "Inputs are pushed, not polled" | `tokio-test` clock pause + assert zero work |
| "Schema version is checked" | golden redb fixtures (one matching, one mismatched) |
| "Component A doesn't directly call C" | compile-fail on the direct call + cargo metadata exclusion |
| "Actor X holds state Y" | snapshot the actor's `State` struct after stimulus |
| "Every logical plane is an actor" | topology manifest + runtime topology dump |
| "Request went through actor X" | ordered actor trace pattern |
| "Actor handler does not block" | responsiveness test with a blocked plane actor and a live sibling |

## What this skill is NOT

- **Not a replacement for behavior tests.** Witnesses and behavior
  tests are complementary; a test that proves architecture but never
  asserts the user-facing outcome misses obvious bugs.
- **Not over-engineering for one-off scripts.** A short shell script
  doesn't need a witness; the budget is for the parts the
  architecture rules govern.
- **Not silver-bullet anti-fraud.** A determined adversary defeats any
  test. Witnesses make it *substantially harder* to ship
  architecture-violating code without catching it in review.

## See also

- `skills/actor-systems.md` — actor-density, blocking-handler,
  topology, and trace rules.
- `skills/push-not-pull.md` — the `tokio-test` clock-pause pattern for
  proving no-polling.
- `skills/nix-discipline.md` — the chained-derivation pattern lives in
  nix.
