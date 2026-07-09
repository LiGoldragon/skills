# Skill — session archive garbage collection

## Rules

Use this when cleaning old agent sessions, transcripts, or session-output piles after the psyche asks for session garbage collection.

Privacy is closed by default. Start metadata-first and read private transcript or output content only through aggregator operations within the authorized scope.

Use aggregator as the evidence path. Begin with `InventorySessions(SessionInventoryRequest)` and require a complete, minimal inventory before proposing deletion. If inventory is truncated, malformed, unreadable, missing, or not scoped to the requested roots, stop and report the missing capability unless the only target is an explicitly known already-missing or temporary cleanup path.

Follow list/suggest/review/archive/verify/delete:

1. List sessions with `InventorySessions`.
2. Suggest candidates from metadata such as age, state, duplicate names, existing summaries, and missing files. Do not read transcripts to discover candidates.
3. Use `LookupSession(SessionLookupRequest)` for each candidate. Use bounded aggregator transcript, output read, and search operations only after inventory and lookup, only to harvest evidence for the proposed short summary.
4. Present the candidate list and short summaries for psyche review before archive acceptance.
5. Write accepted summaries to the typed rkyv archive with `WriteSessionArchive(SessionArchiveWriteRequest)` using an explicit archive path under aggregator's local archive root.
6. Verify with `QuerySessionArchive(SessionArchiveQueryRequest)` or `ReadSessionArchive(SessionArchiveReadRequest)` against the same explicit archive path.
7. Delete only after verification succeeds and only as an agent-executed filesystem action on exact reviewed files.

Aggregator archives summaries; it does not physically delete transcripts. Never treat an aggregator reply as deletion authority. Judge the tool results, name the exact files, then delete them yourself only when every gate passes.

Do not delete when archive verification fails, when lookup or bounded evidence is unavailable, or when the evidence would require direct transcript scraping outside aggregator. Stop and report the missing aggregator feature instead of falling back.

Do not perform directory-wide deletion, glob deletion, or inferred sibling cleanup. Delete exact reviewed file targets only. Preserve files not named in review, files outside the request scope, and anything with active lane, tracker, handoff, manifest, or prompt references.

Report rollback limits before deletion. After deletion, report archived path, verification method, exact deleted files, skipped candidates, and any files that must be restored from backups rather than from aggregator.
