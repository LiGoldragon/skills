# Skill — spirit query

## Query Rules

Use `spirit` for read-only intent queries before judgment. Query relevant public intent early when orchestrating, auditing, scouting, translating, designing, editing doctrine, or deciding how a brief should map to durable guidance. Purely mechanical workers may skip this when the brief already supplies the needed intent context.

Read-only operations are `PublicRecords`, `Lookup`, `Count`, and `Observe`. Do not use `Record`, `Propose`, `Clarify`, `Supersede`, `Retire`, `ResolveClarification`, `ChangeRecord`, certainty or importance changes, stash mutation, subscriptions, or maintenance operations from this module.

Use public reads by default. Use private reads only when the task explicitly authorizes that privacy scope, and keep private content out of public chat, reports, commits, and generated doctrine.

## Query Shapes

The CLI takes exactly one argument: inline NOTA when the argument starts with `(`, or a NOTA file otherwise. It replies on stdout with typed NOTA and returns nonzero on transport, parse, or daemon errors.

List public records in the narrowest relevant domain first:

```sh
spirit "(PublicRecords ((Full [(Technology (Software (Intelligence AgentSystems)))]) None))"
```

Lookup a known record identifier:

```sh
spirit "(Lookup <record-id>)"
```

Treat `(Error [record not found])` and `(Error [no matching record])` as negative evidence, not tool failure. Treat validation rejection, parse failure, daemon failure, or unexpected wire shape as a blocker for intent-grounded judgment.

`PublicTextSearch` is an expert/debug interface. Do not present it as normal orchestration guidance unless a specialist source explicitly needs text search over domain-shaped listing.

## Evidence

Report only the query class, relevant record identifiers, and the conclusion needed for the task. Explain a Spirit identifier on first mention when it matters. Do not paste long record lists or irrelevant hashes.

## Source Maintenance Notes

Keep the public domain-list example synchronized with Spirit's current domain schema. Prefer a narrow domain that demonstrates the normal orchestration path. Keep `PublicTextSearch` documented only as expert/debug guidance unless accepted direction makes text search normal again.
