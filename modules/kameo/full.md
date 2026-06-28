# Skill — kameo

## Rules

`Self` is the actor. Put state on the type that implements `Actor`; do not split
behavior marker, state wrapper, and handler object unless the wrapper owns real
lifecycle policy.

Prefer `type Args = Self` for stateful actors. Construct the initial state before
spawn, pass it as args, and return it from `on_start` after validating startup.

Name actor types as domain nouns: `ClaimNormalizer`, `ReadyWorkView`,
`SemaWriter`. Do not add `Actor`, `Message`, `Msg`, or `Handler` suffixes when
the trait implementation already states the framework role.

Define one message type per operation kind. The message name is domain language,
not framework language. Replies are typed domain outcomes; fallible handlers
return typed failure, not strings.

Use `ask` when the caller must observe acceptance, failure, or the resulting
value. Use `tell` only for infallible fire-and-forget notifications where lost
reply information is acceptable.

Never hide a fallible operation behind `tell`. If a handler can fail, expose the
failure through `ask`, a typed reply channel, or an explicit terminal event.

Expose `ActorRef<A>` directly when the actor vocabulary is the consumer API.
Wrap it only when the wrapper adds domain abstraction, combines multiple actors,
or enforces policy the raw ref cannot express.

A `*Handle` wrapper earns its name by carrying domain behavior. `*ActorHandle`
rarely earns its name.

Do not block inside handlers. Move CPU-heavy work to blocking execution,
subprocess work to a bounded process path, and long external waits to a child
actor or delegated reply path.

A delegated reply must preserve backpressure, cancellation, and terminal error
visibility. Detached work that can outlive shutdown needs explicit ownership and
tests.

Supervision is declared topology, not a retry afterthought. State what restarts,
what escalates, what is never restarted, and what state is reconstructed after a
restart.

Do not put a restartable state-bearing actor on a dedicated OS thread without a
clear restart witness. Thread teardown and actor restart can diverge.

Bound mailboxes where unbounded backlog would hide overload. Make rejection,
backpressure, or shed policy visible in message replies and traces.

Tests should spawn real actors, send typed messages, assert replies, and witness
restart or shutdown behavior for supervised actors. Avoid testing only helper
methods when the risk is mailbox, ordering, cancellation, or supervision.

Use Kameo as the actor library. Do not add a wrapper actor trait or second actor
runtime for workspace code unless the task explicitly changes the actor
abstraction.
