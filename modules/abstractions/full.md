# Skill — verb belongs to noun

## Rules

Every reusable verb belongs to a noun. Before writing a function, method,
dispatcher, or helper, ask which type owns the behavior.

If the owning type exists, attach the behavior there. If no owning type exists,
the model is incomplete; name the missing type instead of writing a floating
verb.

Free functions are narrow exceptions: binary entry points, tiny private helpers
inside one implementation, and genuinely relational operations between values of
equal status.

The rule's value is the forcing question. It makes the missing noun visible
before behavior spreads across call sites.

Prefer affordances over operations. `value.render()` advertises what that value
can do; `render(value)` hides the owner and lets unrelated values look eligible.

Do not choose a nearby noun just because it can hold the method. The right noun
is the type whose concern matches the verb's concern.

When a parser-like free function appears, look for the implicit state object:
input, cursor, diagnostics, mode, and output policy usually want a parser type.

When schema-generated types are the boundary objects, attach behavior to the
emitted noun that is the primary subject. Change data shape in the schema and
regenerate; do not hand-write mirror types.

A newtype owns its representation. Keep wrapped fields private unless public raw
access is truly the contract.

Typed boundaries are specific. Avoid string tags, generic record bags, and enums
that mix unrelated concerns to avoid naming the real type.

When two structured inputs meet, name their contact point instead of scattering
logic through boolean checks and string predicates.

Distinct roles get distinct types. Two fields with the same primitive
representation but different meanings are different domain values.

Actor nouns carry state and behavior together unless the framework forbids it.
Do not create public empty actor markers as method holders.

During review, sweep both sides: reusable free functions that need nouns, and
empty or wrong nouns that merely host behavior.
