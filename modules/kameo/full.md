# Skill — kameo

How to express the workspace's actor discipline in Kameo: `Self` IS
the actor, messages are typed per-kind, supervision is declarative.

## What this skill is for

Use it when you write or edit Rust that defines, spawns, supervises,
or messages an actor. Kameo 0.20 is the workspace's actor runtime.

The architectural discipline — when a plane deserves an actor, what
counts as actor-shape, no-blocking-handler, no-public-ZST-actor-noun
— lives in `skills/actor-systems.md`. This skill is *how* you land
that discipline in Kameo. Kameo's shape agrees with it; no carve-outs.

Kameo is pre-1.0 and small enough that minor-version API churn is
real. Pin the version per-crate and expect minor breaks. Kameo 0.20
declares `rust-version = "1.88.0"` — bump any crate on an older
toolchain before adopting it.

## The core shape

Kameo's load-bearing fact: **`Self` IS the actor.** Not a behavior
marker plus a separate `State`, not a wrapper crate. The struct that
carries your actor's data is the type you `impl Actor` on.

```rust
use kameo::actor::{ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use kameo::Actor;

pub struct ClaimNormalizer {
    in_flight:     HashMap<RequestId, WirePath>,
    max_in_flight: usize,
    metrics:       ClaimNormalizerMetrics,
}

impl Actor for ClaimNormalizer {
    type Args  = Self;          // the documented common case
    type Error = Infallible;

    async fn on_start(args: Self, _ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }
}

pub struct Normalize { pub operation: OperationId, pub path: WirePath }

impl Message<Normalize> for ClaimNormalizer {
    type Reply = Result<NormalizedScope, ClaimNormalizerFailure>;

    async fn handle(
        &mut self,
        msg: Normalize,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.in_flight.insert(msg.operation, msg.path.clone());
        let scope = self.validate_and_collapse(msg.path)?;
        self.metrics.normalize_count += 1;
        self.in_flight.remove(&msg.operation);
        Ok(scope)
    }
}

let normalizer = ClaimNormalizer::spawn(ClaimNormalizer { /* … */ });
let scope = normalizer.ask(Normalize { operation, path }).await?;
```

The actor owns its data; methods on that data (`fn
validate_and_collapse(&mut self, …)`) live on the actor. The
no-public-ZST-actor-noun rule is satisfied for free: the actor type
IS the data-bearing noun.

## Naming actor types

Drop framework-category suffixes — `*Actor`, `*Message`, `*Msg`,
`*Handler`. Let the role-shaped name carry meaning; the trait impl
(`impl Actor for Counter`) already makes framework participation
explicit. `Self IS the actor` makes this naturally enforceable: no
second behavior-marker type to disambiguate against. (Full cross-
language rule: `skills/naming.md`.)

| Concept | Wrong | Right |
|---|---|---|
| Actor type | `ClaimNormalizerActor`, `CounterActor` | `ClaimNormalizer`, `Counter` |
| Message type | `IncMessage`, `IncMsg`, `Inc` | `Increment` |
| Message type | `SubmitMessage` | `ClaimSubmission` |
| Reply type | `SubmitReply` | `SubmissionReceipt` |
| Handle type | `CounterHandle` wrapping `ActorRef<Counter>` for nothing | `ActorRef<Counter>` directly |

Role-shaped suffixes (`*Supervisor`, `*Resolver`, `*Normalizer`,
`*Tracker`, `*Ledger`, `*Store`) describe what the type DOES and
stay. `*Handle` is relationship-naming (same shape as `JoinHandle`)
and earns its place when the wrapper carries domain content (see
below).

## Public consumer surface — `ActorRef<A>` or domain wrapper

`ActorRef<A>` is statically typed against the actor; sending the
wrong message is a compile error. No wrapper newtype buys safety —
the only question is what API makes sense for the consumer.

### `ActorRef<A>` directly — when the actor IS the public API

Default for actors whose message types ARE the consumer surface.
Consumer is handed an `ActorRef<A>` and calls `ask`/`tell` directly.
Re-export `kameo::actor::ActorRef` from the crate root if it cleans
up consumer imports. Most workspace actors fit this: small, single
message vocabulary, in-workspace consumers, no orchestration to hide.

### Domain wrapper — when the public API is a domain abstraction

When the consumer surface is a domain abstraction *over* one or more
actors, wrap. Name it a bare domain noun (`Mind`, `Router`) when the
wrapper IS the conceptual surface; use the `*Handle` suffix only when
the bare noun would shadow a sibling data type (`LedgerHandle` when
`Ledger` is the data record). Never `*ActorHandle`. For remote-network
services `*Client` may beat `*Handle`.

```rust
pub struct Mind {
    root:   ActorRef<MindRoot>,
    reader: ActorRef<SemaReader>,
}

impl Mind {
    pub async fn claim(
        &self,
        role:   ActorName,
        scope:  WirePath,
        reason: ScopeReason,
    ) -> Result<ClaimAcceptance, MindError> {
        self.root.ask(MindRequest::Claim { role, scope, reason }).await
            .map_err(MindError::from)?
            .into_acceptance()
    }
}
```

A wrapper earns its place when **at least one** of these holds:

1. **Lifecycle ownership** — `start(config)` / `stop()` naming "I own
   this live service," not "I hold a ref."
2. **Topology insulation** — if `Ledger` later splits into
   writer/reader/index, the public `Ledger.append()` / `.read()`
   surface stays stable.
3. **Fallible-`tell` prevention** — the wrapper exposes only the safe
   method (`mind.claim(...)` does `ask` internally), removing the
   consumer's option to `tell` a `Result`-returning handler and crash
   the actor. (See "The tell-of-fallible-handler trap".)
4. **Capability narrowing** — `LedgerReader` / `LedgerWriter` as
   distinct wrappers over one actor, each exposing only read or only
   append.
5. **Domain error vocabulary** — `Result<T, MindError>` instead of
   `Result<T, SendError<Submit, SubmitError>>` at every call site.
6. **Domain verbs over Message construction** — `mind.claim(role,
   scope, reason)` instead of `mind_ref.ask(MindRequest::Claim {
   … })`.
7. **Library publication** — consumed by code that shouldn't
   construct Kameo Message values directly, wanting a stable surface
   that survives Kameo version churn.

If the wrapper just holds an `ActorRef<A>` and delegates
method-by-method with no transformation, error mapping, lifecycle
ownership, or capability narrowing, it's type laundering — drop it
and expose `ActorRef<A>` directly. Don't pre-pay a wrapper cost for a
runtime swap that may never come.

```rust
// Wrong — wrapper adds nothing the type system isn't already enforcing
pub struct CounterHandle {
    counter: ActorRef<Counter>,
}
impl CounterHandle {
    pub async fn increment(&self) -> Result<i64, SendError<Increment>> {
        self.counter.ask(Increment).await
    }
}
```

When a justified wrapper exists, expose raw `ActorRef` access for
advanced consumers (tests, custom orchestration) deliberately, not
implicitly:

```rust
impl ClaimNormalizerHandle {
    /// Escape hatch for tests and advanced orchestration.
    pub fn actor_ref(&self) -> &ActorRef<ClaimNormalizer> {
        &self.normalizer
    }
}
```

## Module map

The one source of confusion is the `kameo::actor::*` vs
`kameo::error::*` split. Memorise it:

| Symbol | Path |
|---|---|
| `Actor`, `Spawn`, `ActorRef`, `WeakActorRef`, `ActorId`, `PreparedActor`, `Recipient`, `ReplyRecipient` | `kameo::actor::*` |
| `Message`, `Context`, `StreamMessage` | `kameo::message::*` |
| `Reply`, `ReplyError`, `ReplySender`, `DelegatedReply`, `ForwardedReply` | `kameo::reply::*` |
| `ActorStopReason`, `PanicError`, `PanicReason`, `SendError`, `RegistryError`, `HookError`, `Infallible` | `kameo::error::*` |
| `bounded(n)`, `unbounded()`, `MailboxSender`, `MailboxReceiver`, `Signal` | `kameo::mailbox::*` |
| `RestartPolicy`, `SupervisionStrategy`, `SupervisedActorBuilder` | `kameo::supervision::*` |
| `ACTOR_REGISTRY`, `ActorRegistry` | `kameo::registry::*` (only without `feature = "remote"`) |

Default cargo features are `["macros", "tracing"]`. Leave `remote`
off — Persona is local-process, libp2p is heavy, and the registry
API changes signatures under `remote`. If you turn it on, record an
explicit decision in the consumer crate's `ARCHITECTURE.md`.

`use kameo::prelude::*;` is the convenience import. Add
`use kameo::message::StreamMessage;` for `attach_stream`, and
`use kameo::error::Infallible;` if you write `type Error` by hand
(`#[derive(Actor)]` covers both).

## Lifecycle hooks

| Hook | Default | When to override |
|---|---|---|
| `on_start(args, ref) -> Result<Self, Error>` | required | Always; constructs the actor. |
| `on_message(...)` | dispatches via `BoxMessage::handle_dyn` | Almost never — only custom buffering/scheduling. |
| `on_panic(&mut self, ref, err) -> ControlFlow<ActorStopReason>` | `Break(Panicked(err))` — actor stops | When the actor should survive specific panic kinds. Inspect `err.reason()`. |
| `on_link_died(&mut self, ref, id, reason) -> ControlFlow<ActorStopReason>` | `Continue` for `Normal`/`SupervisorRestart`, else `Break(LinkDied{..})` | When peer death should be visible without stopping. |
| `on_stop(&mut self, ref, reason) -> Result<(), Error>` | `Ok(())` | When the actor must persist or clean up before drop. |
| `next(&mut self, ref, mailbox_rx) -> Result<Option<Signal>, Error>` | `mailbox_rx.recv()` | When the actor merges other input via `tokio::select!`. |

Three load-bearing details:

- **`on_start` failure short-circuits.** An `Err` or panic wraps as
  `PanicError { reason: PanicReason::OnStart }`, the `JoinHandle`
  resolves to `Err`, and **`on_stop` is not called**. Under
  supervision it's restartable like any `Panicked` reason.
- **`on_stop` panics propagate.** Kameo does *not* `catch_unwind`
  around `on_stop`; a panic there ends the actor's tokio task as a
  panicked task. Return `Err` instead — errors land in
  `shutdown_result` for `wait_for_shutdown_result()` to surface.
- **`PanicReason` names the source**: `HandlerPanic`, `OnMessage`,
  `OnStart`, `OnPanic`, `OnLinkDied`, `OnStop`, `Next`. Inspect via
  `err.reason()`; downcast via `err.downcast::<MyError>()` or
  `err.with_str(|s| …)`.

## Messages and replies

Each message kind is a separate `Message<T>` impl; impls compose
freely on one actor and dispatch is resolved statically at the call
site. Names are full English (`Increment` not `Inc`; `ReadCount` not
`Read`, which would shadow `std::io::Read`).

```rust
impl Message<Increment> for Calculator { type Reply = i64; async fn handle(/*…*/) -> i64 { /*…*/ } }
impl Message<Multiply>  for Calculator { type Reply = i64; async fn handle(/*…*/) -> i64 { /*…*/ } }
impl Message<ReadCount> for Calculator { type Reply = i64; async fn handle(/*…*/) -> i64 { /*…*/ } }
```

The `#[messages]` macro on an `impl` block generates these; hand-
rolled impls are also fine and often clearer.

### `ask` vs `tell`

| Form | Returns | Use when |
|---|---|---|
| `actor_ref.ask(msg).await` | `Result<R::Ok, SendError<M, R::Error>>` | The reply matters. |
| `actor_ref.tell(msg).await` | `Result<(), SendError<M>>` | Fire-and-forget. |

Both work directly via `IntoFuture`. Builder methods
(`mailbox_timeout`, `reply_timeout` (ask only), `try_send`,
`blocking_send`, `send_after` (tell only)) are there when needed.

For `type Reply = Result<T, MyError>`: the Ok path returns `Ok(T)`;
the Err path returns `Err(SendError::HandlerError(MyError))`. Match
on the variant — don't `unwrap_or` past it:

```rust
match actor_ref.ask(Divide { /*…*/ }).await {
    Ok(value)                                           => use_value(value),
    Err(SendError::HandlerError(DivisionError::ByZero)) => { /*…*/ }
    Err(SendError::ActorNotRunning(_))                  => { /*…*/ }
    Err(SendError::Timeout(_))                          => { /*…*/ }
    Err(other)                                          => panic!("unexpected: {other:?}"),
}
```

### The `tell`-of-fallible-handler trap

A handler whose `Reply = Result<_, _>` returning `Err(_)` to a `tell`
becomes `ActorStopReason::Panicked(PanicError { reason:
PanicReason::OnMessage })`, and the default `on_panic` stops the
actor. This is the most common Kameo footgun. **Never `tell` a
fallible handler unless you've overridden `on_panic` to recover from
`PanicReason::OnMessage`.** When in doubt, `ask` and ignore the reply
— the error routes to the caller as `SendError::HandlerError` and the
actor lives.

### `DelegatedReply<R>`

Use when the handler must defer the reply to a spawned task — the
work is async/IO/long-running and the mailbox should not block on it.

```rust
impl Message<DoSlow> for Worker {
    type Reply = DelegatedReply<String>;

    async fn handle(&mut self, msg: DoSlow, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let (delegated, sender) = ctx.reply_sender();
        if let Some(tx) = sender {
            tokio::spawn(async move {
                let result = expensive_io(msg).await;
                tx.send(result);
            });
        }
        delegated
    }
}
```

The actor returns immediately; the spawned task replies later. The
caller's `ask().await` blocks until `tx.send(...)` fires (or the task
drops). Without `DelegatedReply` the mailbox would block on the slow
work — re-creating the hidden-lock failure mode actor-systems warns
against.

## Spawning

| Form | Returns | Notes |
|---|---|---|
| `MyActor::spawn(args)` | `ActorRef<MyActor>` | Sync. Default mailbox capacity 64. |
| `MyActor::spawn_with_mailbox(args, mailbox::bounded(256))` | `ActorRef<MyActor>` | Sync. Custom mailbox. |
| `MyActor::spawn_with_mailbox(args, mailbox::unbounded())` | `ActorRef<MyActor>` | Sync. No backpressure. |
| `MyActor::spawn_in_thread(args)` | `ActorRef<MyActor>` | Sync. Dedicated OS thread; **panics on `current_thread` runtime**. |
| `MyActor::spawn_link(&peer, args).await` | `ActorRef<MyActor>` | **Async.** Linked before the run loop starts (avoids the spawn-then-link race). |
| `MyActor::supervise(&parent, args).restart_policy(...).restart_limit(n, dur).spawn().await` | `ActorRef<MyActor>` | **Async.** Supervised. `Args: Clone + Sync` (or `supervise_with(factory)`). |
| `MyActor::prepare()` → `prepared.actor_ref()` → `prepared.spawn(args)` | `PreparedActor<MyActor>` | The `ActorRef` exists *before* the run loop — useful for pre-registering or pre-enqueueing. |

`PreparedActor::run(args).await` is the clean test shape for
"messages changed actor state and I need to assert on the final actor
value" — it hands the actor value back after shutdown:

```rust
let prepared_actor = Ledger::prepare();
let ledger_ref = prepared_actor.actor_ref().clone();
ledger_ref.tell(OpenItem { title }).await?;
let stop_task = tokio::spawn(async move { ledger_ref.ask(StopAndRead).await });
let (final_ledger, stop_reason) = prepared_actor.run(Ledger::new()).await?;
assert!(matches!(stop_reason, ActorStopReason::Normal));
assert_eq!(final_ledger.snapshot(), stop_task.await??);
```

Default mailbox capacity is **64** (macro docs claim 1000; stale).
Size deliberately when traffic warrants it.

## Test patterns

Prefer push witnesses over sleeps. To know a handler started, a
restart happened, or a link death was observed, have the actor send
on a `oneshot` (one-shot event) or `watch` (repeated events) at the
exact moment:

```rust
let (started_sender, started_receiver) = tokio::sync::oneshot::channel();
let (release_sender, release_receiver) = tokio::sync::oneshot::channel();

gate.tell(HoldUntilReleased { started: started_sender, release: release_receiver }).await?;
started_receiver.await?;
gate.tell(QueuedBehindHeldMessage).await?;
release_sender.send(())?;
```

A bounded `timeout(...).await.is_err()` is acceptable only when the
test proves a should-not-fire condition — never as a substitute for
waiting "long enough." When asserting shutdown, match the structured
`ActorStopReason`, not just a counter:

```rust
let stop_reason = peer.wait_for_shutdown_result().await?;
assert!(matches!(
    stop_reason,
    ActorStopReason::LinkDied { reason, .. } if matches!(*reason, ActorStopReason::Killed)
));
```

For final-state assertions, use `PreparedActor::run` (see "Spawning")
rather than exposing test-only shared locks.

## Supervision

Declarative — no manual restart wiring.

```rust
use kameo::supervision::{RestartPolicy, SupervisionStrategy};

// Parent supervisor carries data so it isn't a public ZST.
struct StoreSupervisor {
    children:    HashMap<ActorId, ChildSpec>,
    failure_log: Vec<RestartEvent>,
}

impl Actor for StoreSupervisor {
    type Args = Self;
    type Error = Infallible;
    async fn on_start(args: Self, _: ActorRef<Self>) -> Result<Self, Self::Error> { Ok(args) }

    fn supervision_strategy() -> SupervisionStrategy {
        SupervisionStrategy::OneForAll   // default is OneForOne
    }
}

let supervisor = StoreSupervisor::spawn(StoreSupervisor {
    children: HashMap::new(), failure_log: Vec::new(),
});
let child = Worker::supervise(&supervisor, WorkerArgs { /*…*/ })
    .restart_policy(RestartPolicy::Permanent)
    .restart_limit(5, Duration::from_secs(10))
    .spawn()
    .await;
```

Defaults: `RestartPolicy::Permanent`, `SupervisionStrategy::OneForOne`,
`restart_limit` 5 restarts per 5 seconds.

| Policy | On panic | On handler error | On normal exit |
|---|---|---|---|
| `Permanent` (default) | restart | restart | restart |
| `Transient` | restart | restart | no restart |
| `Never` | no restart | no restart | no restart |

| Strategy | Behavior |
|---|---|
| `OneForOne` (default) | Only the failed child restarts. |
| `OneForAll` | All children restart together when any fails. |
| `RestForOne` | Failed child + all younger siblings (spawned later) restart. |

`restart_limit(n, window)` is reset-after-quiet, not sliding. Past
the limit the supervisor's `on_link_died` fires for the dead child;
default behavior stops the supervisor.

**Restart reconstructs Self from Args, not from memory** — the load-
bearing supervision rule. On restart:

- The mailbox survives; queued messages reach the new instance, but
  the message currently being processed is lost.
- **Mutable state does not survive.** `on_start` runs again with the
  original `Args` (or a fresh value from `supervise_with(factory)`).
  A counter the crashed instance bumped to 12 reads back as whatever
  `Args` rebuilds to.
- Anything that *must* survive restart belongs outside the actor: in
  the component's `sema-db`-backed redb (durable state), in shared
  `Arc<AtomicU32>` (cheap counters), or in `Args` itself.

Kameo makes restart policy easy to express; it does **not** make
restart semantics automatically safe. Design with reconstruction in
mind.

### `OneForAll` / `RestForOne` can bypass `RestartPolicy::Never`

When a sibling failure triggers `OneForAll` or `RestForOne`, Kameo's
coordinated restart paths can call sibling factories directly,
bypassing each child's individual `RestartPolicy::Never`. A child set
to `Never` may still be respawned when a sibling failure invokes a
group-restarting strategy.

| Strategy | Per-child Policy | Behavior |
|---|---|---|
| `OneForOne` | Any | Each child's policy honored independently. |
| `OneForAll` / `RestForOne` | All share one policy | Predictable. |
| `OneForAll` / `RestForOne` | Mixed | **May bypass `Never`; test explicitly.** |

## Mailbox

Two module-level factories — there is no `Mailbox` type with methods:

```rust
use kameo::mailbox;

let (tx, rx) = mailbox::bounded(64);    // backpressure on full
let (tx, rx) = mailbox::unbounded();    // unlimited; OOM risk under load
```

Bounded is the default. `tell().await` waits when full;
`tell().try_send()` returns `SendError::MailboxFull(msg)`;
`tell().mailbox_timeout(d).send().await` waits up to `d` then fails
with `Timeout`. Pick the form at the call site; there are no built-in
overflow policies. `ask().await` blocks twice — on enqueue, then on
the reply oneshot; `ask().reply_timeout(d).send().await` caps the
reply wait.

## Local registry

Without `feature = "remote"`, the registry is a process-global
`Mutex<HashMap>` at `kameo::registry::ACTOR_REGISTRY`.

```rust
let actor_ref = MyActor::spawn(MyActor { /*…*/ });
actor_ref.register("namespace::name")?;        // sync; RegistryError on collision

let found: Option<ActorRef<MyActor>> = ActorRef::<MyActor>::lookup("namespace::name")?;
```

| Behavior | Detail |
|---|---|
| Collision | `Err(RegistryError::NameAlreadyRegistered)` — never overwrites. |
| Unknown name | `Ok(None)`. |
| Actor death | Entry auto-removed. |
| Strong/weak | Local registry holds **strong** refs — registration keeps the actor alive. |

With `feature = "remote"`, `register`/`lookup` become async, take
`Arc<str>`, require `A: RemoteActor`, and use libp2p Kademlia — a
different shape, named here so consumers don't call the local form on
a remote build.

## Streams

`actor_ref.attach_stream(stream, started_value, finished_value)`
spawns a Tokio task that sends
`StreamMessage::Started(started_value)`, then
`StreamMessage::Next(item)` per item, then
`StreamMessage::Finished(finished_value)` once the stream ends. The
actor implements `Message<StreamMessage<M, T, F>>` (handler typically
`type Reply = ();`). The returned `JoinHandle<Result<S,
SendError<…>>>` resolves with the unconsumed stream if the actor
stops mid-stream — useful for recovery. Backpressure on the mailbox
naturally throttles the producer.

## Links

`actor_ref.link(&peer_ref).await` creates a bidirectional link; when
either dies, the survivor's `on_link_died(id, reason)` fires. Default
continues for `Normal` / `SupervisorRestart`, breaks (stops the
survivor) for `Killed` / `Panicked` / `LinkDied`.
`actor_ref.unlink(&peer_ref).await` removes it. Use `spawn_link`
instead of `spawn` + `link` when the link must exist before the actor
can fail.

Two link patterns — keep them separate by design:

- **Supervision links** — death should propagate. Use the default
  `on_link_died`; the survivor stops on abnormal peer death and the
  supervisor restarts both per its strategy.
- **Observational links** — death should be observed without
  stopping. Override `on_link_died` to record the event (counter,
  channel send, sema row) and return `Ok(ControlFlow::Continue(()))`
  for all reasons.

One actor may do both — fail-fast on its sibling, observe a
downstream watchdog. Be deliberate per pair.

## Workspace conventions on top of Kameo

These apply `skills/actor-systems.md` to Kameo's surface; they are
not Kameo defaults.

- **Public actor nouns carry data.** Kameo permits ZST actors
  (`struct Pinger;`) but the workspace forbids them as the public
  surface. The actor type IS the state; no fields means no actor —
  you have a verb. Find the noun.
- **One actor per file when durable enough to name**
  (`src/actors/store_supervisor.rs`). Co-locate the `Actor` impl, the
  `Message<T>` impls, and the message/reply types in one file.
- **No raw `Spawn::spawn` outside the runtime root.** Spawn at the
  supervision tree's root; child spawns go through
  `supervise(&parent, …).spawn().await`.
- **No blocking inside a normal handler.** A handler that sleeps,
  polls, or runs sync IO has recreated a hidden lock. Move the wait
  into a dedicated supervised actor and send it a typed message (see
  "Blocking-plane templates").
- **Tests live in `tests/`, not `#[cfg(test)] mod tests`** (per
  `skills/rust-discipline.md`).
- **Don't reach for `remote` until cross-process actors are
  designed.** Document the decision in the consumer's
  `ARCHITECTURE.md` if you enable it.
- **Wait on the terminal outcome, not `is_alive()` or mailbox
  closure** (see "Lifecycle contract").

## Lifecycle contract

Implements `skills/actor-systems.md` §"Release before notify": a
watcher must learn an actor terminated only *after* the actor's owned
resources released.

```rust
pub struct ActorTerminalOutcome {
    pub state:  ActorStateAbsence,        // Dropped | NeverAllocated | Ejected
    pub reason: ActorTerminalReason,
}

impl<A: Actor> ActorRef<A> {
    pub async fn wait_for_shutdown(&self) -> ActorTerminalOutcome;
    pub fn is_accepting_messages(&self) -> bool;
    pub fn is_terminated(&self) -> bool;
    // is_alive() — deprecated alias for is_accepting_messages().
}
```

Watchers receive `Signal::LinkDied { id, outcome }` exactly once per
terminated peer, on a control channel physically separate from the
user mailbox. Application rules:

- Supervisors branch on `outcome.state`; `Dropped` is the only signal
  that owned resources released.
- Resource-owning actors need component-specific falsifiable tests
  (rebind socket, reopen redb, etc.).
- Never `tokio::spawn(...)` death dispatch fire-and-forget. Await the
  control-channel accept.

Use the `kameo-push-only-lifecycle` fork; pre-fork versions expose an
ordinal `ActorLifecyclePhase` that is not this contract.

## Blocking-plane templates

The no-blocking-handler rule says *move the wait into a dedicated
supervised actor*. Three templates land it, each for a different
shape of blocking work. Pick by shape:

| Shape of work | Template |
|---|---|
| occasional short blocking call, no async equivalent | 1 — `spawn_blocking` + `DelegatedReply` |
| frequent sync DB / store / watcher | 2 — dedicated OS thread |
| process-exec with async API (`tokio::process`) | 3 — `tokio::process` + timeout |

**Anti-template (the violation):** doing the blocking work inline in
an `async fn handle()` with no detach. The mailbox stalls and the
Tokio worker thread starves any sibling actors scheduled there.

### Template 1 — `spawn_blocking` + `DelegatedReply` detach

For short-to-medium, occasional blocking work (subprocess calls,
blocking IO leaves, bounded CPU bursts). The handler returns
immediately; the work runs on Tokio's blocking pool; the reply ships
when it completes.

```rust
impl Message<DeliverToHarness> for HarnessDelivery {
    type Reply = DelegatedReply<DeliveryResult>;

    async fn handle(
        &mut self,
        message: DeliverToHarness,
        context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let (delegated, sender) = context.reply_sender();
        context.spawn(async move {
            let outcome = tokio::task::spawn_blocking(move || {
                HarnessDelivery::deliver(message)  // sync work
            }).await;
            if let Some(sender) = sender {
                sender.send(outcome.into());
            }
        });
        delegated
    }
}
```

Name the actor as the dedicated blocking plane for its backend in
ARCH — the detach is invisible without that.

### Template 2 — dedicated OS thread (`spawn_in_thread`)

For a state-bearing actor with *frequent* sync work that would burn
through per-call `spawn_blocking` — a redb-backed store, file watcher,
anything where every message touches the same sync backend. The actor
runs on its own OS thread, off the Tokio worker pool. One mailbox,
one writer, one thread.

```rust
fn spawn_in_thread(store: StateStore) -> ActorRef<StateStore> {
    let (actor_ref, mailbox) = kameo::actor::Mailbox::bounded(64);
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("dedicated store runtime");
        runtime.block_on(store.run_loop(mailbox));
    });
    actor_ref
}
```

Pair with a typed schema and the sema-family pattern from
`skills/rust/storage-and-wire.md`.

**Do not use `spawn_in_thread` on a supervised state-bearing actor in
Kameo 0.20.** Kameo signals "child closed" the moment `notify_links`
drops `mailbox_rx`, **before** the actor's `Self` value (and any
durable resource it owns — redb `Database`, file lock, open socket)
is dropped. The parent's `wait_for_shutdown` returns while the OS
thread is still in `block_on(...)` and the resource is still held;
the next process opening the same redb path races the still-locked
file and fails with `Io(UnexpectedEof)` or hangs on the second
`bind()`. Until upstream grows a hook that fires after `Self` is
dropped (or the actor owns a close-then-confirm protocol the
supervisor awaits before propagating shutdown), supervised state-
bearing actors stay on `.spawn()` even when Template 2 is the right
destination shape. The non-supervised `Self::spawn_in_thread(self)`
shape (no parent supervisor, process exits on its own clock) is fine;
the trap is specifically `supervise(&parent, …).spawn_in_thread().await`.
Document the deferral in ARCH and the actor's `on_start` comment.

### Template 3 — `tokio::process` + bounded `timeout` + `kill_on_drop`

For process-exec work where an async equivalent exists
(`tokio::process::Command`). Often cleaner than Template 1 because the
whole handler stays async — no detach machinery.

```rust
async fn run_dconf_write(key: &str, value: &str) -> Result<(), ApplyError> {
    tokio::time::timeout(Duration::from_secs(1), async {
        let mut child = tokio::process::Command::new("dconf")
            .args(["write", key, value])
            .kill_on_drop(true)
            .spawn()
            .map_err(ApplyError::spawn)?;
        let status = child.wait().await.map_err(ApplyError::wait)?;
        if !status.success() {
            return Err(ApplyError::exit(status));
        }
        Ok(())
    })
    .await
    .map_err(|_| ApplyError::timeout())?
}
```

Bounded by `timeout`; child killed on drop or timeout; no
`spawn_blocking`. When `tokio::process` is available prefer this over
`std::process::Command::output()` wrapped in detach machinery.

## Anti-patterns and gotchas

- **Unbounded `on_stop`.** An `on_stop` that awaits forever holds the
  supervisor's restart sequence forever — and supervisors now
  correctly wait for terminal, so it's more visible. Bound async
  cleanup with `tokio::time::timeout`; keep `Drop` impls on actor
  state non-blocking.
- **`tell`-ing a fallible handler.** A `Result::Err` from a `tell`'d
  handler crashes the actor by default. `ask` instead, or override
  `on_panic` to recover from `PanicReason::OnMessage`.
- **Self-`ask` from within a handler.** Deadlocks — the handler is
  busy and can't reply to itself. Debug+tracing builds warn at the
  call site. Split the work into a separate method or actor.
- **`spawn_in_thread` under `#[tokio::test]`.** The default test
  flavor is `current_thread`; `spawn_in_thread` panics with
  *"threaded actors are not supported in a single threaded tokio
  runtime"*. Use `#[tokio::test(flavor = "multi_thread")]`.
- **Supervised `spawn_in_thread` releases `wait_for_shutdown` before
  `Self::drop()` runs.** A supervised state-bearing actor owning a
  durable resource sees it outlive the parent's "children closed"
  signal; restart on the same path races the still-held lock. Use
  `.spawn()` until the upstream `pre_notify_links` hook lands (see
  Template 2).
- **`multi_thread` + parallel restart tests hang.** Even with
  `.spawn()`, multi-thread runtime per test plus `cargo test`'s
  parallel runner triggers a kameo/tokio interaction that hangs
  daemon-restart tests indefinitely. Single-thread `#[tokio::test]`
  (the default) passes the same restart tests in parallel. Prefer it
  for daemon-restart witnesses unless the test specifically needs
  `spawn_in_thread`.
- **`#[derive(Actor)] #[actor(mailbox = bounded(64))]` doesn't
  work.** Documented but unparsed; only `#[actor(name = "...")]` is
  implemented. Use `spawn_with_mailbox`.
- **`PendingReply` (from `ask().enqueue()`) blocks the caller.** The
  actor still runs; the reply sits in the oneshot until you await it.
  Forget to await/drop and the caller hangs.
- **Pipelined `tell(panic_trigger) + ask(other)` races on_panic
  recovery.** Even with `on_panic` returning `Continue(())`, the
  second message's reply oneshot can be set up before recovery
  finishes — caller observes `ActorStopped`. Use `ask(panic_trigger)`
  (which awaits past the panic AND recovery), then `ask(other)` on a
  known-recovered actor.
- **`DelegatedReply`'s spawned task is not supervised actor work.**
  Errors from the detached future don't call `on_panic`; they route
  to the global error hook (or the original ask caller). Use it for
  short reply deferrals; for real long work, supervise a dedicated
  actor.
- **`Args = Self` requires `Clone + Sync` for supervision.**
  `MyActor::supervise(&parent, args)` clones `Args` per restart. If
  `Self` isn't `Clone + Sync`, use `supervise_with(|| MyActor { … })`.
- **`RpcReply` does not exist** (likely confusion with ractor's
  `RpcReplyPort`). Use `DelegatedReply<R>`, `ForwardedReply<M, R>`,
  or `ReplySender<R>`.

## See also

- `skills/actor-systems.md` — the architectural discipline this skill
  serves.
- `skills/rust-discipline.md` — the Rust style Kameo code follows.
- `skills/naming.md` — the type-naming rule applied above.
