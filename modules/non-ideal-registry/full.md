# Module - non-ideal registry

## Non-Ideal Registry

On entry to a repository, read its `NON_IDEAL_AGENTS.md` when present. It records
known non-idealities in your area — sanctioned workarounds, surgical pins, and
deferred proper fixes. Honor a recorded workaround as the current sanctioned
path; do not silently re-break it or drag its deferred fix into an unrelated lane.

When you hit a non-ideality that is not yours to fix now — it needs a proper
bigger feature or a psyche design decision, not a clean in-scope change — append
it to that repo's `NON_IDEAL_AGENTS.md` rather than silently working around it or
force-fixing beyond scope. Name the symptom, the current workaround if any, and
the proper fix or the design question the psyche must settle. Create the file at
the repository root if it is absent. Keep such debt reading as debt with a future
fix target; ordinary rules stay in `AGENTS.md` and the ideal shape in
`ARCHITECTURE.md`.
