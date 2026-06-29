# Skill — Rust methods and types

## Methods live on data-bearing types

Every production Rust function is a method or associated function on a non-zero-sized data-bearing type, or a trait implementation. The exceptions are `fn main()` and test-only helpers. Module-level production `fn`, `const fn`, and `async fn` are forbidden.

Private helpers follow the same rule. If a helper reads or writes a value, the value or its owner is the receiver. If several calls share state, name the state-bearing object and put the methods there. Prefer `From`, `TryFrom`, and domain traits over standalone projection functions.

## Schema objects are method surfaces

Schema-authored records and enums are the nouns. Generated Rust types receive inherent methods or trait implementations; handwritten mirror types and free wrapper functions are drift.

When generated types lack the right surface:

1. adjust the schema if the noun is wrong;
2. regenerate;
3. attach behavior to the emitted type or to the runtime owner that carries state.

Signal, mail, operation, reply, codec, and store-record behavior belongs on those generated objects or on runtime owners such as engines, stores, mailboxes, or ledgers.

## Async flow is object flow

Async work does not justify procedural glue. Name each lifecycle object and put behavior on the phase owner: input, sent event, in-flight mail, executor input, state transition, processed reply, or the actor that owns the state. A free `route_*`, `process_*`, `dispatch_*`, or `apply_*` function usually names the missing receiver.

## Zero-sized method holders are forbidden

A unit struct whose impl only groups functions is a namespace, not a model. Use a zero-sized type only when the zero-sized value itself carries a real type-level capability, marker, policy, or witness. Parser, router, formatter, or manager names must carry the data or resources they operate on.

## Types carry domain meaning

Do not pass domain concepts as primitives when a type can name them. Wrap identifiers, ports, paths, names, tokens, sizes, and validated strings in domain types. Parse and validate at the boundary; carry the typed value inside.

Do not use string prefixes, sentinel strings, or regex checks to recover a type that should already exist. Tests assert typed structure and behavior, not naming accidents.

## One concept has one type

Avoid companion types such as `ThingDetails`, `ThingInfo`, or `ThingData` unless they name a distinct concept with different invariants. If the suffix only means "the fields of Thing", fold it into `Thing` or name the real variant.

The system mints durable identifiers. Agents and tests do not invent identifiers by concatenating strings unless the type contract explicitly defines that encoding.

## One object in, one object out

Methods consume one coherent input object and return one coherent output object. Replace long parameter lists and tuple returns with named request and reply types. Options are typed variants, not booleans.

## Constructors are associated functions

Construction belongs on the constructed type: `new`, `from_*`, `try_from_*`, or a domain-specific associated function. Builders are data-bearing when they enforce staged construction; otherwise they are another free-function holder.

## Names encode direction only when it matters

Use directional names only for real protocol direction or conversion direction: `RequestToReply`, `ClientInput`, `ServerOutput`. Do not encode call order, layer ancestry, or implementation history in type names.
