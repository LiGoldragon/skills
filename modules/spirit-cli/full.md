# Skill — spirit CLI

How to call the deployed `spirit` binary to capture and observe psyche intent.

## What it is

`spirit` captures psyche statements as typed records and serves
observation/subscription queries. The active production binary is the
schema-derived `spirit` component at version `0.16.0`, installed in the
user profile as `~/.nix-profile/bin/spirit`. The user service is
`spirit-daemon.service`, listening under `~/.local/state/spirit/`.

`spirit` is the sole substrate for intent capture. There is no
file fallback; the old `intent/*.nota` substrate is retired. If the
daemon is unavailable, surface that as a blocker.

## How to invoke

The binary takes exactly one argument (the one-argument rule —
`skills/component-triad.md`). Two accepted shapes:

- **Inline NOTA** — argument starts with `(`. The default. Wrap the
  whole expression in shell double quotes. Valid NOTA never contains
  `"` (strings are bare when canonical, or bracket forms `[text]` /
  `[|text|]` when delimiters are needed), so the
  shell double quote is a clean boundary and apostrophes inside the
  description survive. Single-quoting is wrong — it loses apostrophes.
  ```sh
  spirit "(Record (([(Information Documentation)] Decision summary Medium Minimum Zero []) ([([verbatim psyche words] None)] [reasoning supporting the capture])))"
  ```
- **Path to a NOTA file** — argument does not start with `(`; the CLI
  reads the file as the NOTA argument. For records with embedded shell
  metacharacters or too large to keep the bash line readable.
  ```sh
  spirit ./record.nota
  ```

The CLI replies on stdout with the daemon's typed `Output` as NOTA text
— `(RecordAccepted ...)`, `(RecordsObserved [...])`, etc. Exit code is
nonzero on transport, parse, or daemon errors.

The wrapper sets `SPIRIT_SOCKET`; the daemon configuration carries the
ordinary and meta socket paths. There are no CLI flags for sockets or
configuration. Inspect the active wrapper with
`readlink -f $(command -v spirit)` and the user service with
`systemctl --user status spirit-daemon.service`.

## Read the wire shape from the pinned source

The active implementation is `/git/github.com/LiGoldragon/spirit`, with
generated Signal/Nexus/SEMA types under `src/schema/`. `signal-spirit`
provides the working signal contract, including `RecordRequest`, `Query`,
maintenance inputs, `Version`, and `Marker`. Do not infer the wire shape
from old `persona-spirit` documents — read the deployed `spirit` and
`signal-spirit` sources pinned by `CriomOS-home/flake.lock`.

```sh
rg -n '"spirit"' /git/github.com/LiGoldragon/CriomOS-home/flake.lock
cd /git/github.com/LiGoldragon/spirit
rg -n "Entry \\{|RecordRequest \\{|Justification \\{|Query \\{|RecordSelection \\{|PublicTextSearch|pub enum Input|pub struct VersionReport|CollectRemovalCandidates|Marker" schema src/schema
cd /git/github.com/LiGoldragon/signal-spirit
rg -n "RecordRequest \\{|Query \\{|RecordSelection \\{|PublicTextSearch|RemovalCandidateCollection \\{|pub enum Input|pub enum Output|VersionReport|DatabaseMarker" schema src/schema
```

## Encoding rules

Records are **untagged** (`NotaRecord`): enum variants carry a head,
record bodies do not. `Option` is `Some`-wrapping — bare `None` or
`(Some <value>)`. `Description`, `Referent`, `Keyword`, `SearchText`,
and `StatementText` are transparent strings — bare tokens when possible,
bracket strings when they need delimiters. Redundant brackets around a
bare-eligible string are rejected; write `abcd`, not `[abcd]`, and
`schema`, not `[schema]`.

## Recording intent

The deployed `Record` operation carries a two-field `RecordRequest`:
`Entry`, then `Justification`. The deployed `Entry` has exactly seven
positional fields: a vector of
closed-taxonomy `Domains`, a `Kind`, one agent-clarified `Description`,
a certainty `Magnitude`, an importance `Magnitude`, a privacy
`Magnitude`, and a vector of `Referents` — in that order. No verbatim
field on `Entry` and **no time field at all**.

`Justification` is the court model:
`Justification { Testimony * Reasoning * }` where `Testimony` is a
vector of `VerbatimQuote { QuoteText * antecedent (Optional Antecedent) }`
and `Reasoning` is the agent's argued case. `QuoteText` must be the
psyche's **verbatim words** — a paraphrase draws a `MissingTestimony`
guardian rejection. The `antecedent` carries the question or context the
quote answers, as `(Some [text])`; it is load-bearing for short
confirmations ("Sure.") that mean nothing without the question. NOTA
positional records never omit fields, so every `Record` spells the seven
`Entry` fields and the two `Justification` fields.

The agent clarifies the psyche's wording into the description before
recording — that keeps the log dense and searchable rather than verbose
and lossy; the testimony preserves the raw words.

```sh
spirit "(Record (([<Domain> ...] <Kind> [description] <Certainty> <Importance> <Privacy> [<referent> ...]) ([([verbatim quote] (Some [antecedent question or context])) ...] [reasoning])))"
# Kind       ∈ { Decision Principle Correction Clarification Constraint }
# Certainty  ∈ { Zero Minimum VeryLow Low Medium High VeryHigh Maximum }
# Importance uses the same Magnitude ladder; Minimum is the ordinary default.
# Privacy    uses the same Magnitude ladder; Zero is open/public.
```

**Guardian discipline.** The agent guardian judges every working-socket
write and rejects with one typed reason from the deployed
`GuardianRejectionReason` enum. The prompt is stored in
`/git/github.com/LiGoldragon/spirit/src/guardian-prompts/` and enforces
an ordered checklist: testimony/authenticity, warrant, durable-intent
shape, domain/privacy, certainty/importance burden, then duplicate or
contradiction checks. Common rejections: `MissingTestimony` when no
verbatim psyche quote is supplied, `Overstated` when the claimed
certainty outruns the quote's modal strength, `ImportanceUnsupported`
when an elevated importance rung lacks recurrence or blast-radius
evidence, and `NonIntent` when a submission is task state, a
single-component or architectural decision, or an instruction about
operating Spirit itself, rather than a durable universal arrow. Matter
and task state belong in code, `ARCHITECTURE.md`, or skills, not the
intent log — capture is rare (`skills/intent-log.md` §"Intent is rare"). Rejection replies include the supporting record set and
can be very large; pipe through `head` when exploring. The deployed
contract carries `Propose`, `Clarify`, `Supersede`, `Retire`, `Remove`,
`ChangeCertainty`, `BumpImportance`, `ChangeRecord`, and
`CollectRemovalCandidates` maintenance operations. Use them for
maintenance. In particular, a psyche clarification of an existing intent
is a `Clarify` or `Supersede`, not a new `Record` whose description
starts with "clarification".

Domains are closed taxonomy variants such as
`(Information Documentation)`, `(Safety Privacy)`,
`(Technology (Software (Engineering Architecture)))`, or
`(Technology (Software (Quality Testing)))`. Read `signal-spirit`'s
deployed `schema/domain.schema` for the full list when a domain is
unclear.
Narrow free words belong in the description where keyword/text search
can find them. **Named particulars go in `Referents` — populate them.**
Every named thing a record is about — `spirit`, `sema-engine`, `nota`,
`rkyv`, `mirror`, `DeepSeek`, a host, a bead — belongs in the referent
vector. Referents are the record's stable retrieval-and-dedup key: the
guardian pulls existing records that share a referent with the
candidate, so an untagged record hides from that path and forces the
judge to scan far wider. A referent need not be pre-registered — first
use auto-registers it (the implied-referent path), and the
referent-guardian judges the new name, rejecting a verb or vague
concept as `NonReferent` / `TooVague`. Tag the real particulars; leave
the vector empty `[]` only when the record names none. A named instance
is a referent, never a domain — `(... DeepSeek)` is wrong, `DeepSeek`
is a referent (domains are universal subjects). Concrete example — a
universal work principle, which names no single component and so carries
an empty referent vector:

```sh
spirit "(Record (([(Information Documentation)] Principle [ask the psyche when intent is unclear instead of inferring it] High Minimum Zero []) ([([if you don't know what I want, ask me — don't make it up] None)] [universal work-conduct rule; it names no single particular, so the referent vector is empty])))"
```

A single-component architectural constraint — "the sema-engine is the
exclusive interface to the database" — is matter, not intent: it is
scoped to one component, so it is written into that repo's
`ARCHITECTURE.md`, never recorded in Spirit. Populate the referent vector
when a universal rule genuinely concerns named particulars; leave it
empty `[]` when the rule is a workspace-universal subject.

Higher privacy values narrow the audience; `Zero` is the workspace
default. Never put private personal substance in a `Zero` record.

The reply is terse and does not echo content: `(RecordAccepted abcd)`.
Spirit mints random lowercase base36 identifiers and shows the shortest
collision-free code with a four-character minimum. Cite and pass the
short code the daemon returns.

**Shorthand stays typed.** Any future shorthand is a distinct typed NOTA
operation that fills defaults and lowers to the full record. Production
calls use the heads present in the deployed contract.

## Removing and changing records

```sh
spirit "(Remove (abcd ([([psyche authorization quote] None)] [reasoning])))"  # -> (RecordRemoved (abcd))
spirit "(ChangeCertainty (abcd Zero))"    # -> (CertaintyChanged (abcd Zero))
spirit "(BumpImportance abcd)"            # -> (ImportanceBumped (abcd <importance>))
spirit "(ChangeRecord (abcd ([<Domain> ...] <Kind> [replacement description] <Certainty> <Importance> <Privacy> [<referent> ...]) ([([verbatim psyche edit] (Some [target record being edited]))] [reasoning])))"
```

## Clarifying, superseding, and resolving records

Use these when the psyche is editing the meaning of existing intent.
They keep the active store from accumulating "record about another
record" duplicates.

`Clarify` edits one existing record's description while preserving its
identity and core meaning:

```sh
spirit "(Clarify (abcd [corrected description] ([([verbatim psyche clarification] (Some [what was being clarified]))] [reasoning for why this is an edit of abcd])))"
```

`Supersede` retires one or more old identifiers and replaces them with
new `Entry` values in the same operation:

```sh
spirit "(Supersede ([abcd] [([<Domain> ...] <Kind> [replacement description] <Certainty> <Importance> <Privacy> [<referent> ...])] ([([verbatim psyche override] (Some [prior record being replaced]))] [reasoning for replacement])))"
```

`Retire` deactivates one record without a replacement:

```sh
spirit "(Retire (abcd ([([verbatim psyche retirement] None)] [reasoning for retirement])))"
```

`ResolveClarification` atomically folds a mistaken standalone
`Kind::Clarification` record into the records it clarified, then removes
that standalone clarification — in one operation. It **is** deployed:
`signal-spirit` carries
`ResolveClarification(ClarificationResolution)` →
`ClarificationResolved(ClarificationResolutionReceipt)`, where
`ClarificationResolution { ClarificationRecordIdentifier
TargetClarifications Justification }`, each
`TargetClarification { RecordIdentifier Description }` pairs a target
record id with the corrected description to fold into it, and the receipt
reports the resolved clarification id plus the edited target
`RecordIdentifiers`. A live process-boundary test proves the targets are
edited in place and the standalone clarification is removed (a later
`Lookup` of it errors).

```sh
spirit "(ResolveClarification (abcd [(wxyz [corrected target description])] ([([psyche clarification authorization] (Some [what abcd was folding]))] [fold abcd into wxyz as an in-place edit])))"
# -> (ClarificationResolved (abcd [wxyz]))
```

Use it instead of the old manual pass (look up the bad clarification,
`Clarify`/`Supersede`/`ChangeRecord` each target by hand, then `Remove`
the standalone). Do not leave a standalone clarification record active
beside the records it edits. The full guardian rejection vocabulary now
also includes `NegativeGuideline` — a record whose operative guidance is
*primarily* an exclusion / prohibition / forbidden-wording list is
remanded for affirmative rewording (`skills/intent-log.md` §"Affirmative
framing").

`Remove` deletes a record entirely — use it when nothing should remain
in the active store. Setting certainty to `Zero` is the **recoverable**
removal-candidate nomination: the record stays queryable by explicit
zero-certainty lookup and is restored by changing certainty back to a
non-zero `Magnitude`. Ordinary observation hides zero-certainty records.
Use `Clarify`, `Supersede`, or `Retire` when lineage should stay
visible. Use hard `Remove` only after review.

**Collect removal candidates** — archive matching records to the
owner-configured archive database, then remove them from the hot store:

```sh
spirit "(CollectRemovalCandidates (((Full [(Information Documentation)]) Any Any Any (Some Correction) (Exact Zero) (ExactCertainty Zero) Any) ([([psyche authorization quote] None)] [remove zero-certainty documentation corrections])))"
```

Collection's schema field is `RecordQuery`, a generated newtype around
the same eight-field `Query` used by `Observe` and `Count`; the CLI
accepts the direct query shorthand shown above. The second field is the
required `Justification`. For the removal-candidate path, select records
with `(ExactCertainty Zero)`. The reply
`(RemovalCandidatesCollected (...))` carries archived
`RemovalArchiveRecord` values, removed identifiers, skipped candidates,
and no database marker. Archive location is not a working-signal
argument; the owner configures it through the meta socket.

## Observing records

For ordinary public intent lookup, prefer the low-level shorthand
`PublicTextSearch` before spelling a full `Observe` query:

```sh
spirit "(PublicTextSearch [routing protocol])"
spirit "(PublicTextSearch payload-blind)"
spirit "(PublicTextSearch .criome)"
```

`PublicTextSearch` takes exactly one `SearchText`. It searches active
public records by description text and referent text, tolerates
unregistered words as search terms, ranks likely matches, caps the
result set, and returns `RecordsObserved` directly. It avoids the common
agent failure modes: no eight-field positional query, no
`AnyReferent` hard error for an unregistered referent, no follow-up
`LookupStash`, and no bracketed-bare-string canonicalization trap for
single atoms like `.criome` or `payload-blind`.

Use full `Observe` when you need exact domain / keyword / referent /
kind / privacy / certainty / importance predicates or exhaustive stashed
results. `Observe` replies with `RecordsStashed` for non-empty result sets:
the reply carries `(StashHandle RecordCount ObservedRecords)`, so agents can
read the records immediately while still retaining a recovery handle. A
follow-up `LookupStash` against the handle returns the same records as
`RecordsObserved`; it is for recovery, repeated inspection, or passing a
stable snapshot handle between agents, not the normal first-read path.

`Observe`, `Count`, and `SubscribeIntent` carry the generated eight-field
`Query` directly:

```text
(Observe (<DomainMatch> <KeywordMatch> <TextMatch> <ReferentSelection> <Kind?> <PrivacySelection> <CertaintySelection> <ImportanceSelection>))
```

- **DomainMatch**: bare `Any` (no filter), `(Partial [<Domain> ...])`
  matches any requested domain, `(Full [<Domain> ...])` matches every
  requested domain.
- **KeywordMatch**: `Any`, `(AnyKeyword [word ...])`, or
  `(AllKeywords [word ...])`. Keywords are extracted from descriptions.
- **TextMatch**: `Any` or `(ContainsText [text to search])`.
- **ReferentSelection**: `Any`, `(AnyReferent [name ...])`, or
  `(AllReferents [name ...])`.
- **Kind?**: `None` or `(Some Decision)`.
- **PrivacySelection**: `Any`, `(Exact Zero)`, `(AtMost Low)`,
  `(AtLeast High)`.
- **CertaintySelection**: `Any`, `(ExactCertainty Zero)`,
  `(AtMostCertainty Low)`, `(AtLeastCertainty Minimum)`.
- **ImportanceSelection**: `Any`, `(ExactImportance Medium)`,
  `(AtMostImportance Low)`, `(AtLeastImportance High)`.

`Observe` currently stashes non-empty result sets and returns a
`RecordsStashed` handle. Use `LookupStash` with that handle to retrieve
the full `RecordsObserved` payload. Each observed row is
`ObservedRecord { RecordIdentifier * Entry * }`, so observed records carry
their short IDs. `Lookup` retrieves by identifier and bypasses
observation filters, so it can still read a zero-certainty record when
you already know its identifier.

Use `PublicRecords` and `PrivateRecords` for the ergonomic
privacy-scoped shortcuts. They take a two-field `RecordSelection`:

```text
(PublicRecords (<DomainMatch> <Kind?>))
(PrivateRecords (<DomainMatch> <Kind?>))
```

```sh
spirit Version
spirit "(Observe ((Full [(Information Documentation)]) Any Any Any (Some Constraint) (Exact Zero) (AtLeastCertainty Minimum) Any))"
spirit "(Observe (Any (AnyKeyword [spirit schema]) Any Any None (Exact Zero) (AtLeastCertainty Minimum) Any))"
spirit "(Observe (Any Any (ContainsText [schema-derived Spirit]) Any None (Exact Zero) (AtLeastCertainty Minimum) Any))"
spirit "(PublicRecords ((Full [(Information Documentation)]) None))"
spirit "(PrivateRecords ((Partial [(Safety Privacy)]) None))"
spirit "(Lookup abcd)"
spirit "(LookupStash 12)"
spirit "(Count (Any Any Any Any None (Exact Zero) (AtLeastCertainty Minimum) Any))"
```

Use the production heads shown above: `Observe` / `Count` with the
eight-field `Query`, and `PublicRecords` / `PrivateRecords` with the
two-field `RecordSelection`.

Database markers are not part of ordinary replies. Use the explicit
`Marker` operation only when you need the durable database marker:

```sh
spirit Marker      # -> (MarkerReported (<commit-sequence> <state-digest>))
```

## Rendering public intent snapshots

Spirit main contains a companion binary, `spirit-render`, that renders a
public intent snapshot for one or more referents. It is a source-backed
client, not guaranteed to be installed in every user profile yet; if
`command -v spirit-render` fails, run it from the Spirit checkout with
the repo's normal Cargo/Nix surface.

The request is a single NOTA record:

```sh
spirit-render "(Render ([spirit schema] None))"
spirit-render "(Render ([spirit] (Some [/tmp/spirit-intent])))"
```

It queries `Privacy Zero`, `Certainty >= Minimum`, any domain/kind, and
`AnyReferent` over the named referents; it writes `spirit.nota` in the
current directory or requested output directory and prints the output
path. The generated file carries the Spirit marker, generator version,
timestamp, privacy/certainty filters, and the observed records. Treat it
as a generated snapshot, not a replacement for the live Spirit store.

## Certainty and importance

Production Spirit stores both certainty and importance. Certainty means
confidence/currentness, with `Zero` reserved for removal-candidate nomination.
Importance means how much attention has accumulated around this topic or
composite record. Higher importance does not mean higher certainty; it affects
retrieval order and can be filtered with `ImportanceSelection`.

## Other operations

```sh
spirit "(State [free-form psyche statement text])"   # classified, then persisted as a Record
```

`State` carries raw psyche text; the daemon classifies it (fallback
`unclassified` / `Clarification` / `Minimum` / `Zero`) and persists the
resulting `Entry` through the same `Record` write path. The canonical
shape is `(State [text])`; the CLI also accepts the deployed shorthand
`(State ([text]))`.

`Version` is a bare NOTA atom:

```sh
spirit Version
```

`SubscribeIntent` opens a long-lived intent event stream. `Tap` and
`Untap` expose the observer surface over operation/effect observations.

## Daemon startup is binary-only

The CLI accepts NOTA because it is the human/agent text edge; the
daemon does not. Daemon startup is exactly one pre-generated
signal-encoded/rkyv message — inline NOTA and `.nota` paths are
rejected before daemon decoding. A deploy helper (CriomOS-home) may
author configuration from typed NOTA source, but it encodes the binary
startup signal before launching. A virgin daemon can receive an initial
`Configure` as binary signal; after configuration, restarts self-resume
from persisted SEMA state. New configuration fields land as typed
fields in the startup schema or as authenticated meta-signal messages —
never flags, never daemon NOTA parsing.

## Substrate migration discipline

Applies to any migration where a permissive substrate (file with free
PascalCase tokens, untyped store) is replaced by a strict one
(rkyv-archived enum, typed sema-engine store). Four rules:

1. **Enumerate every closed-world enum on both sides before
   relogging.** Compare variant sets; where they differ, design an
   explicit mapping. Don't assume parallel evolution kept the
   vocabularies aligned.
2. **The strict substrate is ground truth.** When the daemon rejects a
   token the file accepted, the target shape wins — the permissive
   substrate was permissive by accident. Migration normalises; it does
   not bridge backward.
3. **Surface mismatches before bulk relog.** A dumb migration tool
   needs the mapping table baked in; even a no-import daemon does not
   absolve the migration step of vocabulary auditing.
4. **Older vocabulary may not round-trip without explicit mapping.**
   Permissive parsers accept tokens the strict decoder later rejects;
   the gap surfaces only at the strict-substrate boundary.

**Canonical pattern — two-submodule migration module** (one per
component-version step):

- `mod historical` — private rkyv reproduction of the deployed old
  types. Every leaf and branch the source bytes need is redefined
  locally, with no dependency on the old crate version, so the
  migration crate reads source bytes deterministically.
- `mod current_shape` — same-name types binding the current crate's
  unchanged leaves, overriding only the fields that changed.
- **A `From`-chain composes the conversion** — `StoredRecord ->
  StampedEntry -> Entry`, plus enum-to-enum maps for the changed leaves
  (e.g. `historical::Certainty -> Magnitude`). One direction of typed
  flow; no per-field handwiring at the call site.

## No manual dual-writing or in-CLI migration

Do not log the same intent by hand to multiple Spirit databases —
version cutover and dual-write are implemented in code. Importing
legacy nota files stays out of Spirit; a separate migration tool may
translate legacy records, but Spirit itself remains the intent daemon
and CLI. Pass the same broad topic strings (`workspace`, `spirit`,
`signal`, `component-shape`, `persona`, …) the deployed store already
uses; the sema-engine `.sema` database carries the canonical record set.

## See also

- `skills/intent-log.md` — what gets logged, the five-kind taxonomy,
  the gold-mining discipline.
- `skills/intent-maintenance.md` — sweep / supersession discipline.
- `skills/nota-design.md` — positional-record encoding rules.
- `/git/github.com/LiGoldragon/spirit` — active component source;
  `tests/process_boundary.rs` and `tests/nix_integration.rs` show the
  live wire shape.
- `/git/github.com/LiGoldragon/signal-spirit` — daemon startup
  configuration contract consumed by the active component.
