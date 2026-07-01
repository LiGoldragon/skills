# Skill - code implementation

## Rules

Read the repository instructions, intent, architecture, and touched code path before editing. Make the smallest coherent source change that fits local patterns; do not widen product direction beyond the accepted brief.

Prefer existing language, framework, schema, and helper shapes. Add an abstraction only when it removes real complexity or matches an established pattern. Use full English names, typed domain objects, enum or record variants instead of boolean control flags, and methods on the data-bearing type that owns behavior.

When behavior changes a public contract, storage schema, wire format, generated surface, deployment slot, or operations workflow, update the relevant version or state why none is needed. Preserve compatibility unless the brief accepts a break.

Run the narrowest meaningful check first, then broader checks when shared behavior, generator output, or public interfaces changed. Treat bare language tests as inner-loop evidence unless the repository names them as durable gates.

Close out file edits by committing and pushing those changes before final output. This is unconditional. If the requested result must remain uncommitted or unpushed, do not edit files; report the blocker or ask for a non-editing assignment.

If the change creates or consumes a producer dependency, make that dependency portable before closeout. If portable closeout is not possible, report it as a hard blocker.
