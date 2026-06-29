# Skill — NOTA-as-comments

## Rules

Use a NOTA-shaped comment only when a future editor needs a machine-readable rationale at the code location. Routine comments stay ordinary prose.

A rationale comment starts with `(Why "short summary" ...)` and uses positional sub-records:

```rust
// (Why "preserve retry state on transient failure"
//   (caused-by "the failure path dropped pending state")
//   (alternatives-considered [RetryWithoutState PreservePendingState])
//   (chosen-because "keeps retry semantics explicit"))
fn handle_failure(&mut self) -> Reply {
    // ...
}
```

Use bare PascalCase atoms for variant positions and string forms for prose. Keep field order stable. Do not invent labeled pairs inside the record.

Place the comment immediately above the function, type, module, line, or block it explains. A function rationale goes above the function, not at the top of the file.

Write one only for a substantive choice, a surprising shape, or a decision a later editor is likely to reverse without the rationale. Skip mechanical edits and obvious code.

If the rationale does not fit positional NOTA, write ordinary prose. Do not wrap free text in fake NOTA.
