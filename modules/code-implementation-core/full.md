# Module - code implementation core

## Implementation Core Purpose

Ordinary implementation turns an accepted brief into the smallest coherent
source change that fits the repository. The worker owns local understanding,
code edits, and verification evidence; broader product direction stays with the
brief or the psyche.

## Implementation Local Fit

Read repository instructions, intent, architecture, and the touched code path
before editing. Prefer existing language, framework, schema, and helper
patterns. Add an abstraction only when it removes real complexity or matches an
established local pattern.

Use full English names and typed domain objects. Avoid boolean control flags
where a closed record or enum can name the variants. Put behavior on the
data-bearing type that owns it. Where two enums meet, name the contact point
instead of scattering conditionals.

Beauty is a correctness gate: a special case should dissolve into the normal
case. If a fix works only by adding a side path that future agents must
remember, keep looking for the shape that makes the rule explicit.

## Implementation Version Compatibility

When behavior changes a public contract, storage schema, wire format, generated
surface, deployment slot, or operator workflow, update the relevant version or
state why none is needed. Preserve compatibility unless the brief explicitly
accepts a break.

## Implementation Verification

Run the narrowest meaningful check first, then broader checks when shared
behavior, generator output, or public interfaces changed. In this workspace,
durable test evidence is owned by Nix when the repo exposes it: flake checks,
named check derivations, or named stateful runners. Bare language test commands
are inner-loop evidence unless the repo says otherwise.

## Implementation Closeout

When the brief requires publishing, commit and push your own green changes after
verification. Agent-authored commit messages include the acting model and
thinking/provenance level in the message body when that information is available
from the harness or role packet. Leave unrelated working-copy changes uncommitted
and name them in the output.
