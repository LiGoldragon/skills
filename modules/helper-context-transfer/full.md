# Skill — helper context transfer

## Rules

Use a helper when bounded reading or mechanical exploration would waste the lead lane's high-value context. The helper receives the reading envelope; the lead receives distilled evidence.

Brief helpers with:

- task and success question;
- exact source locators or commands to inspect;
- authority: read-only, write scope, or verification-only;
- privacy and safety boundaries;
- output path or return shape;
- required evidence and known blockers, including forced special cases or stale integration facts the helper must surface;
- for editing helpers, session name, lane name, and Fresh/Recovery mode, with instructions to register that lane, claim write paths under it, release its claims, and unregister it at closeout.

Request an output artifact only when the helper's result is a pickup surface for another worker or a fresh context. Otherwise use chat or harness output as the return shape. When an artifact is needed, give the exact path or the session and artifact names.

Do not make the lead read broad source first merely to brief the helper. Put the necessary context in the helper prompt and let the helper inspect.

Read helper outputs before acting. Treat them as evidence, not authority. If tool-call syntax, scaffolding residue, or unsupported claims appear, verify before using the result.

Dispatch the smallest helper that answers the question. Do not outsource decisions the lead owns. A handoff result is concise: verified facts with locators, implications, validation/risk, and a ready next action; omit work chronology.
