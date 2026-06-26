# Skill — Nix-backed testing

Every test lives in Nix; constraints become named witnesses exposed through the component flake.

## What this skill is for

Adding, reviewing, or debugging tests in any workspace component: where tests live, how they run, and how stateful or multi-step behavior becomes inspectable instead of vanishing inside one end-to-end loop. For the witness catalogue see `architectural-truth-tests.md`; this skill answers the runner question — how the witness is made reproducible through Nix.

## All tests live in Nix

- `nix flake check` is the canonical pure test gate for a repo.
- Pure tests run as `checks.<system>.<name>` or via `checks.<system>.default`.
- Stateful tests are still exposed by the flake. If they cannot run inside the pure Nix builder, expose a named script or binary through `apps.<system>.<name>` or a package output and run it with `nix run .#<name>`.
- A recurring manual command is not a test contract until it is a versioned script and a named flake output.
- Bare `cargo test`, ad hoc shell commands, and local one-off scripts are inner-loop conveniences only — never evidence for review.

The point is not to force every stateful experiment into a pure builder; it is that the test command, its environment, and its artifacts are owned by the repo and entered through Nix.

## No positive grep deployment checks

A broad positive `grep` (e.g. `grep -R "SemaWriteInput" src/schema/lib.rs`) only proves text exists in a file — not that the runtime uses the generated type, the daemon crosses the binary boundary, or the schema chain is live. Don't use grep as deployment or architecture proof.

Grep in Nix checks is allowed only as a narrow negative — proving a retired or forbidden surface is absent, e.g. `! grep -R "NexusMail" src tests`. For positive proof write a real witness: compile generated types, execute the trait path, round-trip NOTA or rkyv, reject bad socket bytes, run the process-boundary test, or consume a produced artifact in a later derivation.

## Constraint → witness → Nix

For every load-bearing behavior or architecture constraint:

1. Name the constraint in plain English.
2. Name the observable witness that proves the intended path happened.
3. Choose the Nix shape: pure check, stateful runner, or chained derivations.
4. Expose the shape as a flake output.
5. Name the test after the constraint.

Good test names read like constraints: `router_cannot_deliver_without_commit`, `message_cannot_persist_without_sema`, `query_cannot_touch_writer_state`, `handler_cannot_block_mailbox`. If the same visible result can pass through a shortcut, the witness is too weak.

The witness must exercise the production code path it claims to protect. Fixture code is allowed only as outside stimulus or observer — a fake child process, temporary database directory, socket peer, test harness, or artifact reader. The load-bearing action still crosses the real boundary: the real daemon, actor, socket protocol, Sema writer, parser, contract type, or CLI binary. A test that builds a miniature copy of the logic and proves the copy works is not a witness; it is a self-contained story. Delete it or reroute it through the component's production API.

## Pure tests

Pure tests are the default. They run in the Nix build sandbox and are reachable from `nix flake check`. Use them for:

- Rust unit, integration, doc, and compile-fail tests.
- Source scans and dependency-graph assertions.
- Cargo metadata boundary checks.
- rkyv, NOTA, Signal, and other golden byte or text fixtures.
- Actor topology manifests and actor-trace pattern checks that don't need a live terminal or host daemon.

Rust tests follow `rust-discipline.md`: tests live under `tests/` at the crate root, not in large inline `#[cfg(test)]` blocks. The Nix check owns the runner.

Schema source used by Rust tests lives in real `.schema` fixture files, not
inline Rust string literals. The test should read the fixture by path or
`include_str!`, then decode it through the same schema-source artifact path the
component uses. Inline strings are acceptable only for non-schema primitives
or deliberately invalid one-token parser probes where creating a fixture would
hide the point of the test.

## Stateful tests

Stateful tests touch a database, terminal, socket, daemon, external tool, or host-visible harness. They still live in Nix:

- Put the command in a versioned script or binary owned by the repo.
- Expose it through the flake, normally as `nix run .#test-<name>`.
- Use explicit environment variables and arguments; don't depend on a user's home directory, ambient daemon, or untracked local setup.
- Use a fresh state directory unless the point of the test is to read a supplied fixture.
- Emit inspectable artifacts: transcript, redb file, actor trace, topology manifest, frame bytes, rendered output, or log bundle.
- Prefer a pure check that validates the artifact shape when the live run cannot happen in the builder.

### QEMU-backed VM checks

QEMU-backed NixOS VM checks run only on hosts explicitly designated or authorized for VM testing. Before invoking `runNixOSTest`, `microvm`, or any flake check known to boot a VM, verify the host/builder is a VM-testing host or ask for the correct host. Ordinary hosts run compile, eval, projection, source-scan, and other non-VM checks only.

Repos that expose VM checks name them clearly (`vm-*`, `*-deploy-smoke`, or a documented VM-check family) and put the VM-testing host requirement next to the check definitions. If a VM check cannot prove it is on an authorized VM-testing host, it reports that boundary instead of starting QEMU.

For stateful daemon components, drive the production daemon through its thin CLI control surface. The CLI is part of the component's test/control API even when no human-facing command is promised. The test proves the real daemon path; the CLI must not open the durable database directly or recreate the daemon's state machine in-process. Read-only inspection CLIs may open the component Sema database to render artifacts, but keep them named as inspection surfaces and pair them with daemon-driven writers so the test still proves where state came from.

A stateful runner that only prints "passed" is weak. It should leave evidence another step, tool, or human can inspect.

## Chained tests

Use chained tests when a monolithic end-to-end test could hide a stub, mock, in-memory shortcut, or unused phase:

1. A first Nix derivation or runner produces an artifact.
2. A second Nix derivation consumes only that artifact and validates it with the authoritative reader for that layer.
3. Further derivations repeat for each real phase.

For example: a writer step emits `state.redb`; the reader step opens `state.redb` with the real Sema/redb reader and asserts typed rows. Or a parser step emits `frame.bin`, a handler step consumes it and emits `reply.bin`, a renderer step consumes that and emits `output.txt`. Or a daemon step emits a shutdown state directory and a restart step copies it into a fresh sandbox to prove the next process can read it.

The artifact is the boundary. A later step must not share process memory, mocks, or private helper APIs with the earlier step. If the writer did not actually write, the reader has nothing to read. The concrete chained-derivation pattern lives in `repos/lore/nix/integration-tests.md`.

## Artifact discipline

Artifacts are part of the test design, not leftovers.

- Name artifacts after the constraint or phase they witness.
- Keep them small enough to inspect.
- Prefer stable binary or text formats already owned by the component.
- Don't record raw store hashes in docs; let Nix produce them.
- When a check consumes a previous check's output, reference the flake output path instead of copying through an ambient temporary location.
- When copying a store artifact into a writable state directory, set writable permissions explicitly.

Make the correct path visible and the shortcut path boring to reject.

## Test-only binaries — the `-test` suffix

A repo may legitimately ship binaries that are not on the production path — standalone test harnesses for a library primitive, deterministic-input fixtures driven by witness scripts, validator helpers. They name themselves with the `-test` suffix so the production surface is unambiguous at a glance from `Cargo.toml`.

- A `[[bin]]` without the `-test` suffix is a production-path binary. It may be invoked by deployment units, supervised process trees, service catalogs, persona engine manager spawn configs, and operator workflows.
- A binary with the `-test` suffix is for tests, witnesses, or isolated experimentation only. It may not be referenced by any production component's `[dependencies]`, deployment unit, supervised process tree, or service catalog. Witness scripts under `<repo>/scripts/` and Nix `check` derivations may invoke it freely.

A name without `-test` is read by every other agent (and by `nix flake show`, deployment derivations, and supervision configs) as "this is production." The convention is forward-looking: existing unsuffixed binaries migrate as their repos are next touched. Example: `terminal-cell-daemon-test` is a standalone PTY-primitive daemon for exercising the `terminal-cell` library without standing up `persona-terminal-daemon`.

Enforce it filesystem-side with a per-repo witness `<repo>-test-only-binaries-have-test-suffix` — source-scan `[[bin]]` entries against the production-surface allowlist; any production-named binary not on the allowlist (and not named with `-test`) fails the check.

The convention applies to `[[bin]]` entries that produce a runnable. It does not apply to Cargo test files under `tests/` (already test-only by convention), Nix `check` derivations (already named by the constraint they witness), or per-test child processes spawned via `tokio::process::Command::new(env!("CARGO_BIN_EXE_*"))` against a `-test` binary.

When production-vs-test status is ambiguous (e.g. a validator an operator might invoke during a build), prefer the `-test` suffix plus a Nix `check` derivation that wraps it for the operator path. The Nix wrapper carries the production status; the underlying binary stays clearly test-only.

## Multi-repo remote override tests

The detailed flake-ref and override-input policy lives in
`skills/nix-discipline.md`; this section only states the testing shape.

When a feature spans several sibling repositories, create a central test
runner in the consumer repo that rebuilds the whole stack together. Commit
and push each participating repo first; the runner uses Nix input overrides
pointing at remote refs, never local filesystem paths:

```sh
nix flake check \
  --override-input nota-next-source github:LiGoldragon/nota-next?ref=operator/my-feature \
  --override-input schema-next-source github:LiGoldragon/schema-next?ref=operator/my-feature \
  --override-input schema-rust-next-source github:LiGoldragon/schema-rust-next?ref=operator/my-feature
```

The committed flake still uses portable `github:` inputs. The override
runner is for integration pressure while developing several repos at once:
edit the codec or schema emitter, push the branch refs, run the consumer's
central test, and prove the generated Rust still compiles, serializes, and
crosses the process boundary. `--override-input ... path:...` and
`git+file://` are forbidden because they make the result depend on local
checkout state and can copy huge ignored build directories.

For schema-derived repos the central runner proves the whole chain:

```text
schema files -> assembled schema -> generated Rust -> hand-written
methods on generated objects -> rkyv signal frame -> CLI/daemon test
```

A test that only checks the schema engine in isolation is not the central
integration test; one that only checks the consumer with default locked
dependencies is not the remote override test.

Schema-derived runtime tests must use the generated plane traits in the consumer. A pilot does not prove the stack by calling a primitive store helper or recording strings in an observer; it proves the stack by constructing generated `Input`, `NexusInput`, and `SemaInput` values, invoking generated traits such as `NexusEngine` and `SemaEngine`, observing generated mail/rejection values, and asserting generated `NexusOutput`, `SemaOutput`, and Signal `Output` replies. If a needed boundary type is still hand-written, move it into schema first or mark that as the next component-development gap.

## Schema-typed observer state and per-plane chain typing

Tests in the schema-derived stack use schema-emitted data types end-to-end — both in observer state and in the execution chain the test exercises.

### Observer state stays typed

When a test attaches an observer (a `MessageSentHook`, `MessageProcessedHook`, or similar hook-trait impl), the observer's accumulated state IS the schema-emitted enum. It holds `Vec<MailLedgerEvent>` (or the equivalent typed enum for the surface under test), NOT `Vec<String>` with tokens like `flow:sent:1`. The token-string anti-pattern (`format!("{label}:sent:{id}")`) bypasses the type system the design relies on — the test exercises the engine but the assertion runs on text.

Two acceptable shapes. Direct typed assertion — observer holds typed events, assertion compares typed values:

```rust
let expected: Vec<MailLedgerEvent> = vec![
    MailLedgerEvent::Sent(SentMail {
        mail_identifier: MailIdentifier(1),
        short_header: ShortHeader(0),
    }),
    // ...
];
assert_eq!(observer.lock().expect("observer").events(), &expected);
```

Or NOTA round-trip — typed events lowered through their schema-emitted `to_nota()`, compared against an inline NOTA fixture (readable for multi-event sequences):

```rust
let expected_nota = "(Sent (1 0))\n(Processed (1 (1 39)))";
assert_eq!(events_to_nota(&recorded), expected_nota);
```

### Per-plane chain typing

Tests exercise the actual execution chain through the engine trait surfaces, using the right schema-emitted type at each plane crossing. For the Signal / Nexus / SEMA triad:

- Signal-engine operations take `Input` and produce `Output`. `SignalActor::accept(Input)` returns `SignalAccepted`; the witness is a Signal-plane type.
- Nexus-engine operations take `NexusMail<Payload>` and produce `NexusOutput`. The Nexus engine also performs the Signal→Nexus and Nexus→SEMA translations through schema-emitted methods like `NexusInput::into_nexus_output` and `NexusOutput::into_sema_input`.
- SEMA-engine operations take `SemaInput` and produce `SemaOutput`. `Store::apply` is the SEMA engine trait surface.

Each step uses the right schema-object type for its plane; tests invoke the engines through their trait surfaces with typed values, then assert on typed output values. Where a test crosses a plane boundary, make the crossing visible — inspect the `NexusOutput` produced from a `NexusMail<Entry>` and thread its `SemaInput` into the next engine. The chain typing is visible in test code, not hidden behind a single `engine.handle()` call:

```rust
// Step 1: typed NexusMail<Entry> at the Nexus engine trait surface.
let nexus_mail: NexusMail<Entry> =
    NexusMail::new(MessageIdentifier(42), entry("..."));
let nexus_reply: NexusOutput = engine.record(nexus_mail).expect("infallible");

// Step 2: explicit Nexus → SEMA translation, typed all the way.
let sema_input: SemaInput = nexus_reply.into_sema_input();

// Step 3: SEMA engine trait surface — SemaInput in, SemaOutput out.
let sema_output: SemaOutput = sema_engine.apply(sema_input);
```

A test that exercises only one plane (e.g. a focused SEMA-engine test) uses that plane's schema-emitted types throughout — `SemaInput` in, `SemaOutput` asserted; it does not construct `Input` (Signal) values. A test that bypasses the engine trait surface (testing private helpers, or constructing intermediate values without going through trait dispatch) is a discoverable shortcut — note it for follow-up; not necessarily blocking.

## Anti-patterns

- A README command that is not a flake output.
- A recurring debug script that is not versioned.
- `cargo test` as the only claimed verification.
- A test that invents a parallel implementation inside the test and proves only that the invented logic works.
- A test fixture that replaces the production boundary it claims to witness instead of driving that boundary from the outside.
- One huge integration loop that round-trips through the same code and exposes no intermediate artifacts.
- Writer and reader using the same mock, cache, or in-memory object.
- A stateful test that depends on an existing home directory, daemon, socket path, database, or credential unless that dependency is the explicit subject of the test.
- A Nix check that only builds the binary but never executes the witness.
- Sleeps or polling used to pretend a push-based event happened. Push behavior needs a pushed witness.
- An ignored test without a tracked reason.
- Stringly-typed observer state (`Vec<String>` with tokens like `flow:sent:1`) when the schema emits a typed enum — the assertion runs on text and bypasses the type system.
- Collapsing multiple plane crossings into one `engine.handle()` call when the invariant under test IS the per-plane chain typing.
- Constructing intermediate per-plane values without going through the engine trait surface (`InputNexus::record`, `Store::apply`, etc.). The trait surface is the production path the test must prove.

## Review checklist

- Can a clean checkout run the canonical suite with `nix flake check`?
- If stateful, is the command exposed through `nix run .#<name>` or another named flake output?
- Does each load-bearing constraint have a named witness?
- Does the witness prove the intended component or phase was used?
- Does the witness call, drive, or observe the production code path, not just test-local logic?
- Are intermediate artifacts inspectable?
- Would a shortcut, stub, or bypass fail?
- Is the test name the constraint it protects?
- For schema-derived stack tests: does observer state hold schema-emitted typed values (`Vec<MailLedgerEvent>`) rather than string tokens? Does the test exercise each plane's engine trait surface with the right plane's schema-emitted types? Are the Signal→Nexus and Nexus→SEMA crossings visible in the test code?

## See also

- `architectural-truth-tests.md` — witness catalogue and architecture-test patterns.
- `rust-discipline.md` — Rust test layout and crate rules.
- `nix-discipline.md` — Nix command and flake discipline.
