# Skill - feature development

## Rules

Use a named feature bookmark in an isolated checkout for features, experiments, rewrites, and shippable prototypes. Do not work directly on a shared integration line unless repository guidance explicitly requires it.

The task owner names the branch and scope. Base the work on `main`, claim the isolated path before editing, and return exact validation and revision evidence. If the deployed coordination client cannot create an isolated checkout, obtain explicit fallback authority for the path, creation method, publication target, and cleanup disposition; do not invent an unavailable operation or share a busy checkout.

Keep experiments in the existing repository even when a branch replaces the whole tree. Separate checkouts are separate claim scopes; one checkout never has concurrent editors.

Publish the feature bookmark without integrating it unless the task grants integration authority. Conclude or remove the isolated checkout through the supported client operation or the explicitly authorized fallback disposition.
