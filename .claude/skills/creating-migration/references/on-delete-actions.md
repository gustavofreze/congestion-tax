<!-- Vendored from payment (.claude/skills/creating-migration/references/on-delete-actions.md) on 2026-11-18. Adapted to this project: the PostgreSQL behavior of NO ACTION sits beside the MySQL one, and the CASCADE example is the tariff_bands of a tax period. -->

# ON DELETE action decision aid

Every foreign key declares an `ON DELETE <action>` chosen to match the real semantic of the relationship. Pick
deliberately from the table below. When no row clearly fits, default to `RESTRICT` (the safe non-destructive fallback),
flag the choice, and never pause to ask.

| Action      | When to use                                                                                                                                                                                                                                       |
|-------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `CASCADE`   | The child row has no meaning without the parent and must vanish with it (e.g., `tariff_bands` rows when their `tax_periods` row is deleted).                                                                                                      |
| `RESTRICT`  | The parent must not be deletable while children exist and the application enforces the lifecycle (e.g., a `client` referenced by `appointments`, or any reference table with an `is_active` flag where rows are deactivated rather than deleted). |
| `SET NULL`  | The relationship is optional and losing the link is a valid state. The FK column must be nullable (e.g., `dispatch_id` when the dispatch is removed).                                                                                             |
| `NO ACTION` | Avoid. In MySQL it behaves like `RESTRICT`, and in PostgreSQL it differs only by deferring the check to the end of the statement. Either way it signals nothing extra and confuses readers.                                                       |
