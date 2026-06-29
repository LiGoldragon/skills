# Skill — enum contact points

## Make closed state combinations explicit

When two closed domains meet, name the contact point instead of scattering branch logic. Use a nested match, a small trait, or a method on the owning enum. The reader should see the full cross-product in one place.

## Pick the contact shape

Use `Reaches<Right>` when the left value decides how it reaches a right value. Use `Contact<Other>` when neither side owns the meeting. Use `Dispatch<Token>` when an input variant chooses which operation runs.

The contact point owns the match. Call sites pass typed values and receive typed results; they do not repeat conditionals.

## Keep derived values honest

If a derived value is cheap and has no identity, compute it in the contact method. If it names a concept, stores state, or appears in multiple APIs, promote it to a type with its own invariants.

## Replace flags with variants

Boolean fields that steer behavior across a closed set usually hide an enum. Model modes, phases, permissions, and outcomes as variants; then match on the enum at the contact point.

## Avoid sentinel and string dispatch

Sentinel values are unnamed variants. Strings are data, not type state. Parse at the boundary into enums or typed records, then dispatch on the type.

## Avoid predicate soup

A pile of `is_*` methods forces every caller to rebuild the cross-product. Prefer one match that returns the domain result. Keep predicates only for genuinely local assertions.

## Do not over-trait simple local logic

If a contact is used once and both enums live nearby, a private method with a nested match is enough. Use traits when the contact point is reused, crosses module boundaries, or expresses a stable protocol relation.

## Return typed results

The contact point returns a domain type, not an unstructured string or generic boolean. If the result has more than one outcome, model it as an enum and keep the caller's logic typed.

## Test the matrix

Tests cover the meaningful variant pairs. They should fail when a new variant enters without an explicit decision for its contacts.

## Engine rule

Engine branches are architecture. Centralize them, name the operation, and test the variant matrix. Missing arms should fail loudly during review or compilation.
