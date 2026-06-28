# Skill — context handover

## Rules

Use when the psyche asks for a handover, clean-session prompt, restart context,
or material to bring forward next turn.

A handover gives a fresh agent the least context needed to understand the matter
accurately. It is not a transcript digest, correction log, or display of agent
reasoning.

Keep psyche intent and matter context distinct. Psyche intent is what the psyche
wants, rejects, values, or explicitly decides. Matter context is terminology,
constraints, source locators, observed facts, and live decisions that still affect
the work.

State the final corrected truth. Do not preserve the mistaken path that led to
it.

Include only facts whose absence would make a fresh agent misunderstand the
matter or repeat a settled question.

Exclude agent guesses, apologies, reasoning trails, tool chronology, and routine
working-copy state.

Exclude restatements of skills, protocols, repository instructions, or command
manuals the next agent will load from their owning surfaces.

Exclude dead acronyms, resolved mistakes, stale branches, and trivia that no
longer affects a decision.

If operational continuation matters, name the exact live blocker, artifact,
command result, or source locator the next agent needs. Otherwise delete
implementation chronology.

Every line must answer: would a fresh agent misunderstand the matter without
this? If not, delete it.
