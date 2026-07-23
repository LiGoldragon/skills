# Skill — component architecture

- Keep daemon state, thin CLI, and typed public contract separate.
- Put wire types and codecs in the contract, not the runtime.
- Route requests through admission, effect, and state owners.
- Test the public path and each ownership boundary.
