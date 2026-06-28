# Skill — actor systems

## Rules

Actors mark correctness boundaries. Use an actor when a plane owns state,
authority, IO, ordering, supervision, durable mutation, view maintenance, reply
shaping, or trace responsibility.

A plane is actor-shaped when it has a domain noun, a failure mode callers act on,
and independent typed input that can test it. Pure value transformations without
domain failure stay as methods on the owning actor or data type.

Prefer many small named actors over one vague actor with hidden helper phases.
Smallness is acceptable; triviality is the cutoff.

The actor type carries the state. If a wrapper only forwards to one data type,
put the mailbox on that data type. Keep a wrapper only when it owns lifecycle,
supervision, admission control, backpressure, restart policy, or child topology.

Zero-sized actors are not real actors. A real actor carries state, policy,
resources, child refs, metrics, or witness fields that survive between messages.

Do not share mutable state through locks to avoid actor design. A shared lock
around the real state usually means the actor boundary is in the wrong place.
Move ownership behind one mailbox.

Blocking inside a handler is a design bug. Separate blocking work into a bounded
blocking path, subprocess actor, or child worker whose terminal outcome returns
through a typed message.

Supervision is part of the architecture. State which actors restart together,
which failures escalate, which actors never restart, and how state is rebuilt or
recovered.

Release resources before notifying dependents. Shutdown ordering is: stop
admission, drain or reject in-flight work, commit or abort owned state, release
external resources, then publish terminal outcome.

Control planes are physically separate from data planes. A shutdown, reload, or
policy update must not queue behind unbounded ordinary work if it is needed to
restore safety.

"Dispatched" means enqueued, not processed. Use a distinct terminal event or
reply for processed, committed, failed, cancelled, and timed-out outcomes.

Durable state belongs behind one single-writer owner. Readers consume snapshots,
queries, or subscribed views; they do not mutate the durable store.

Counter-only state in tests is a witness, not decoration. Assert the counter or
delete it.

Runtime roots are actors with real data-bearing engine types. Generated or
schema-root messages should enter actor mailboxes early rather than dissolve into
free helper functions.

Trace every actor boundary that matters: message kind, request identity, caller
or authority class, queue/dispatch result, terminal outcome, and child failure
when supervision fires.

Actor-density tests should fail when a plane is collapsed or bypassed. Test the
topology, not just happy-path return values.
