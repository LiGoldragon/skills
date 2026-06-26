# Skill — main feature integration

Use this when updating a code repo's `main` with a feature that is not already
green. This applies to code repos under `/git`, not primary.

## Flow

1. Start from the current `main` head: `jj new main`.
2. Work on a named operator bookmark while the feature is not green.
3. Keep each affected repo on a main-based branch; do not stack new work on stale
   designer integration branches unless the task is explicitly to integrate that
   branch.
4. Test the branch family together. Temporary local path overrides are allowed
   during testing, but remove them from the merge-ready state unless the branch
   dependency is intentional and documented.
5. When the candidate is green, fetch/recheck main. If main moved, rebase the
   branch onto the new main and rerun the relevant tests.
6. Land main in dependency order. For a compiler stack, land producers before
   consumers, then update consumer lockfiles against the newly landed main.
7. Push main only after the post-rebase checks are green.

## Reporting

State the basis commit, the branch bookmark, the exact temporary overrides used
for testing, the tests run, and whether final main landing happened or remains
blocked on an upstream branch landing.
