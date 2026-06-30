# Skill — push, not pull

The principle lives in `ARCHITECTURE.md` §"Push, not poll": **producers push, consumers subscribe; no poll loops.** This skill assumes that rule and describes how to apply it. If you reach for a polling loop, stop and apply the steps below.

## Designing a producer-consumer interaction

1. **Find the producer** — the component that owns the state the consumer cares about.
2. **Find or build its subscription primitive** — a callback registration, event stream, long-lived RPC, Unix-socket subscriber, `inotify` file watch, or `timerfd` deadline. The shape varies; the contract is fixed: the producer pushes, the consumer registers once.
3. **Write the consumer as a subscriber.** No `sleep(N)` in its main loop, no interval timers, no "check every K seconds."
4. **If the producer can't push, escalate** (below). Never write a poll loop "for now" — a poll once written is rarely removed, and its cost is paid forever.

Most substrates already give you a push channel: an actor's mailbox, a database change feed, a backing store's change events. An actor handler that *blocks* also violates push discipline — while it waits, its mailbox can't accept the next pushed fact. Split the wait out: the domain actor sends a typed request and returns to its mailbox; an IO/clock/worker actor replies when the event arrives.

## Subscription contract

Every subscription emits the producer's **current state on connect**, then deltas after that. The consumer must not run a separate "what is it now?" query to seed itself. Without the initial event, a consumer that subscribes after a state already exists waits forever for a change that never comes. "Subscribe, receive current state, then receive changes" is the standard shape for focus state, input buffers, message tails, and any stateful stream.

## When the producer can't push — escalate

If the subscription primitive doesn't exist yet, the path is one of:

1. **Build the primitive in the producer** — usually right when the producer is in scope.
2. **Replace the producer** with one that can push, if it can't be modified.
3. **Defer the dependent feature** until push ships — stated explicitly; don't pretend it's shipping.
4. **Escalate** — when none of the above resolve it, the question goes up, ultimately to the human, who decides whether a carve-out is justified, the producer rebuilt, or the feature deferred.

Escalation is the correct outcome when no push answer is found — it is the discipline working, not a failure. Falling back to a poll is never the answer.

## The three carve-outs

Three patterns look polling-shaped but aren't, and they are exhaustive:

- **Reachability probes** — "is service X alive?"
- **Backpressure-aware pacing** — consumer drains its own buffer; the producer still pushes.
- **Deadline-driven OS timers** — `timerfd` and equivalents; the kernel pushes the wake.

When a design seems to need polling and none of these apply, it needs an escalation, not a fourth de-facto carve-out.

## Pull-shaped traps

Patterns that smell fine but are polling — each replaced by the push event you actually care about:

- **Re-reading a file every N ms** → `inotify`/`kqueue`, or a daemon emitting events on a socket.
- **`sleep_ms(50); observe_again` for stable-state detection** → a producer event for the transition.
- **A retry timer for "unknown" state** → the event that resolves the unknown; if none exists, escalate.
- **A consumer "ticker" driving reconciliation** → subscription + event-triggered reconciliation.
- **An actor handler that sleeps until something changes** → a subscription or dedicated actor receiving a pushed completion event.
- **"Check every interval, debounce flickers"** → the debounce is hiding the poll; use the push source.
- **Asking an LLM agent to "check inbox every few turns"** → a router pushes the inbox into the harness terminal stream; the model doesn't pull.

## Recognising the symptom

Polling shows up as **wake-when-nothing-changed**. A process that shows steady idle syscall traffic on `strace -c`, holds a near-constant context-switch rate in `/proc/<pid>/status`, or emits log lines on a clock independent of input is polling. Push-correct systems go quiet when they have nothing to do.
