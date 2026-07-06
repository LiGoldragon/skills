# Skill — reporting

## Rules

Default to chat or the required worker return shape. Write a report only when the
file itself is the working surface: fresh-context handoff, cross-role pickup,
durable analysis, requested artifact, or discovery that must survive the current
harness output.

Do not write a report to restate a routine commit, test list, or tool transcript.
Put routine implementation summaries in chat, the worker return, or the commit
description.

When a report exists, chat still carries the user's working surface:

- the report path;
- a one-sentence result;
- blockers, decisions, surprising findings, or recommendations stated inline.

A report is for a fresh agent. Include task, scope, consulted files and
commands, observed facts, interpretations, changed files or proposed changes,
checks run with results, blockers, unknowns, and follow-up requirements.

Keep observations and interpretations separate. A command result, file path, or
visible absence is an observation. A likely cause, risk, or next step is an
interpretation.

Name every meaningful command or step and its result. For checks, state pass,
fail, or not run with the reason. Prefer relevant evidence and major steps over
exhaustive tool-call transcripts.

Use short revision hashes for ordinary human context. Include full revision
identity only when exact machine identity is required.

Do not paste secrets, private personal material, tokens, or host-private details
into public reports, chat, or generated outputs.

Reports live where the active lane or brief says. If no exact path is supplied,
use a stable session directory and a filename that names the role and artifact.
Do not scatter continuation files across unrelated directories.

Use versioned filenames only when a prior report remains useful as a historical
artifact. Otherwise correct the existing report in place.

References are self-contained. When a user decision depends on evidence, paste
the relevant evidence or summarize it completely; do not send the user to hunt
through another file.

Use prose, tables, or diagrams only when they carry the decision. Delete
narration, apology, progress diary, and tool-output padding.

A report may recommend durable guidance, but it does not create authority. Move
accepted rules into the owning source surface instead of treating the report as
doctrine.
