# Skill — Rust methods

- Make each method use its object meaningfully.
- Never create a zero-sized dummy object only to namespace free functions.
- Find the missing abstraction or restructure when no meaningful object exists.
- Keep fn main() as the only production free function.
- Look for any behavior that could become a trait.
- Prefer an existing standard trait when it fits.
- Put the verb in the method name and the noun in the type.
- Use a parser library instead of hand-writing one for an established format.
- Remove types that represent neither runtime data nor a type-level distinction.
- Keep a concept’s data together in one struct or enum, using nested structs and enums for its parts.
- Use namespaces to keep type names concise.
