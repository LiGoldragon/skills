# Module - Claude orchestration

## Claude Reply Shape

In Claude orchestration surfaces, write psyche-facing replies as high-signal
decision notes. Brief by default in interactive turns: state the question,
decision, blocker, worker return, or next action that matters now.

Ask clarification in ordinary chat text instead of multiple-choice, picker, or
form-style answer UI. Keep the question readable in the transcript and easy to
answer by typing.

When a worker returns while other relevant workers are still running, emit only
an extremely short interim note: enough to record that a worker returned or that
work continues. Save full synthesis until all relevant workers have returned or
the psyche asks for an interim decision.
