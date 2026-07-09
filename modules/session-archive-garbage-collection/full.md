# Skill — session archive garbage collection

## Rules

Use this when cleaning old agent sessions, transcripts, or session-output piles after the psyche asks for session garbage collection.

Privacy is closed by default. Start metadata-first and read private transcript or output content only within the authorized scope.

Use aggregator as the inventory and archive evidence path. Begin with `InventorySessions(SessionInventoryRequest)` over paths supplied by request data or derived configuration, never hardcoded local paths. Treat configured producer source shape as data: if a producer represents sessions through symlinks, inventory and review that shape through the protocol instead of labeling it unsafe by default.

Require a complete, minimal inventory before proposing deletion. If inventory is truncated, failed, malformed, unreadable, missing, or not scoped to the requested roots, do not treat it as complete deletion evidence. Fix or configure the sources, or explicitly exclude the incomplete source from review, before any deletion review continues.

Follow list/suggest/review/archive/verify/delete:

1. List sessions with `InventorySessions`.
2. Suggest candidates from metadata such as age, modified-time sort order, state, duplicate names, existing summaries, and missing files. Deleting sessions older than a threshold is only a reviewed agent action; aggregator may help surface age-based candidates but never deletes automatically.
3. Use `LookupSession(SessionLookupRequest)` for each candidate. Use bounded aggregator transcript, output read, and search operations only after inventory and lookup, only to harvest evidence for the proposed short summary.
4. Present the candidate list and short summaries for psyche review before archive acceptance.
5. Write accepted summaries to the typed rkyv archive with `WriteSessionArchive(SessionArchiveWriteRequest)` using an explicit archive path from request data or derived configuration under aggregator's archive root.
6. Verify with `QuerySessionArchive(SessionArchiveQueryRequest)` or `ReadSessionArchive(SessionArchiveReadRequest)` against the same explicit archive path.
7. Delete only after verification succeeds and only as an agent-executed filesystem action on exact reviewed files.

Aggregator archives summaries; it does not physically delete transcripts. Never treat an aggregator reply as deletion authority. Judge the configured source results, name the exact files, then delete them yourself only when every gate passes.

Do not delete when archive verification fails, when lookup or bounded evidence is unavailable, or when source configuration cannot produce review-complete evidence. Stop and report the missing configuration or aggregator capability instead of falling back to unscoped scans.

Do not perform directory-wide deletion, glob deletion, or inferred sibling cleanup. Delete exact reviewed file targets only. Preserve files not named in review, files outside the request scope, and anything with active lane, tracker, handoff, manifest, or prompt references.

Report rollback limits before deletion. After deletion, report archived path, verification method, exact deleted files, skipped candidates, and any files that must be restored from backups rather than from aggregator.
