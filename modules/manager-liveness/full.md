# Module — manager liveness

## Worker Liveness

The manager never spawns a blocking agent. Every manager-dispatched agent runs
in the background. Never use a foreground agent call or wait synchronously for
a result. If later work depends on a return, defer its dispatch until completion
notification arrives while keeping psyche chat available for redirection.

Never dispatch an agent whose only job is to wait or poll. A wait lives in
durable state — a tracked work item, coordination record, or sequenced
condition — executed by a short-lived check-and-act dispatch when its signal
arrives, so a dead waiter cannot silently take its task with it.

Do not interrupt or terminate a worker for turn count or silence during a
long-running command. Inspect concrete evidence of blockage first. The same
evidence standard binds the opposite claim: absence of completion news is not
liveness. Report a worker as running only on fresh positive evidence — a live
coordination record or a recent run artifact; otherwise its state is unknown,
verified before the manager depends on it or reports it. Match acceptance
criteria to the task shape; do not fail a read-only Scout for lacking
changed-file evidence.
