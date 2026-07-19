# Module — harness placement

## Placement

Classify each instruction before placing it. General doctrine describes behavior
independent of a runtime API. Harness-specific doctrine depends on one target's
API, wrapper, or interaction.

Place concrete harness API fields only in a module explicitly routed by
`target-module-insertions.nota` to an emitted output surface for that harness.
Keep them out of base modules and role-composition modules that feed multiple
targets.

Confirm the target and its insertion exist in the active manifests before writing.
When no matching target-specific surface exists, omit the rule and return the
missing support as a placement gap; do not invent an overlay or generalize it.

After generation, verify the target module appears only in its scoped output and
that the general outputs contain none of its API fields.
