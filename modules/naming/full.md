# Skill — naming

## Use full English without redundant ancestry

Spell identifiers as full English words, and do not repeat the namespace that already contains them. Inside `Profile`, use `size`, not `profile_size`. Inside a protocol module, use `Request`, not `ProtocolRequest`, unless the shorter name collides with another concept in the same scope.

Names carry the context the namespace does not. Short names are acceptable only when their scope is tiny and their meaning is immediate.

## Common offenders

Replace abbreviated code dialect with words:

| Avoid | Use |
|---|---|
| `id`, `ident` | `identifier` |
| `ctx` | `context` |
| `cfg` | `configuration` or `config` when that is the domain word |
| `addr` | `address` |
| `buf` | `buffer` |
| `tmp` | name the value's role |
| `args` | `arguments` |
| `params` | `parameters` |
| `vars` | `variables` |
| `tok` | `token` |
| `lex` | `lexer` |
| `repr` | `representation` |
| `gen` | `generate` or `generator` |
| `ser`, `deser` | `serialize`, `deserialize` |
| `calc` | `calculate` |
| `init` | `initialize` |
| `proc` | `process` or `procedure` |

## Narrow exceptions

- Loop counters in tiny scopes may be `i`, `j`, or `k`.
- Standard mathematical symbols may be used inside clear mathematical code.
- Generic parameters may use conventional single letters when they carry no domain meaning.
- Widely lexicalized technical words such as `json`, `http`, `cpu`, or `url` may stay short.
- Names inherited from a library API may match that API at the boundary.
- Wire formats may preserve externally specified field names at the serialization boundary.

## Length follows scope

A variable used for three lines may be short. A public type, field, trait, module, or error variant needs enough words to be understood without reading its implementation.

## Minted identifiers stay context-cheap

Identifiers minted for agents to read are short codes, never long hashes; long hashes are context-expensive for the LLMs that consume them. Mint an agent-facing identifier as a randomly generated base36 code starting at four characters, checked for uniqueness against the live store when minted and grown one character only when a length saturates. Prefer that scheme over a content hash. The Spirit record-identifier mint (`spirit` repo, `src/store/record_identifier.rs`) is the reference. This governs the identifier's value; the field and type that hold it still spell `identifier` in full.

## Avoid category suffixes

Do not add `Manager`, `Handler`, `Helper`, `Util`, `Info`, `Data`, or `Details` unless the suffix names a real domain role. Rename to the thing owned, the event handled, or the operation represented.

## Siblings get distinguishing words

Sibling names should expose the axis that differs: `PendingMessage`, `DeliveredMessage`, `RejectedMessage`; not `MessageData`, `MessageInfo`, `MessageDetails`.

## Generated names preserve the schema vocabulary

When schema emits code, fix naming at the schema source. Do not compensate with local aliases or wrapper names that hide a bad schema term.
