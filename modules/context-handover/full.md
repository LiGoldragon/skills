# Skill — context handover

## Purpose

Use when the psyche asks for a handover, clean-session prompt, restart context,
or "bring this back next turn."

A context handover gives a fresh agent the least context needed to understand
the matter accurately. It is not a transcript summary, status report, work log,
or replay of agent reasoning.

## Include

Include only information that furthers the fresh agent's understanding of the
intent being discussed:

- psyche intent: what the psyche wants, rejects, values, or has explicitly
  decided;
- useful matter-context: meanings of terms, relevant research results,
  constraints, source locators, and open decisions that still affect the work;
- current facts only when they are necessary for the intent or next decision.

State resolved corrections as the final truth. Do not preserve the correction
history. Write "psyche wants Y", not "the agent thought X but psyche meant Y".

## Exclude

Exclude anything that does not improve the next agent's understanding:

- agent guesses, confessions, apologies, and reasoning trails;
- report/file/tool results unless they are useful matter-context;
- implementation chronology, commit lists, and working-copy state unless the
  psyche asked for operational continuation and the detail affects the next
  move;
- restatements of skills or protocols the next agent will read itself;
- resolved mistakes, dead acronyms, and trivia that no longer affect a decision;
- generic next-step advice not explicitly requested by the psyche.

## Shape

Prefer a compact prompt with two sections:

```text
Intent:
- ...

Useful context:
- ...
```

Add `Open decisions:` only when the psyche has left live choices for the next
agent to resolve. Do not add a plan unless the psyche asked for one.

## Standard

Every line must answer: "Would a fresh agent misunderstand the matter without
this?" If not, delete it.
