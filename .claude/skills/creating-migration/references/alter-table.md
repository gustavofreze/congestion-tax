<!-- Vendored from payment (.claude/skills/creating-migration/references/alter-table.md) on 2026-11-18. Adapted to this project: the PostgreSQL spelling (no AFTER, a separate CREATE INDEX and COMMENT ON, a closed set as a CHECK) sits beside the MySQL one, and the ON DELETE clause the source split across three lines is written on one. -->

# Alter table

Use this when a migration adds columns or constraints to an existing table, placing each new column inside its group.

## `mysql` spelling

```sql
-- ALTER TABLE shape. Multiple statements representing a single logical change
-- may be combined. Use AFTER <column> (or FIRST) to place each new column inside
-- its group/subgroup, and place business DATETIME(6) columns immediately before
-- the audit columns. Without AFTER, MySQL appends at the end, breaking both the
-- grouping invariant and the audit-columns-last invariant.

ALTER TABLE orders
    ADD COLUMN dispatch_id BINARY(16)   NULL COMMENT '[NONE] The dispatch identifier in Version 7 UUID format (e.g., 01912d42-3d4e-75f6-a7b8-91a2b3c4d5e6).' AFTER customer_id,
    ADD COLUMN dispatched_at DATETIME(6) NULL COMMENT '[NONE] The UTC date and time when the order was dispatched in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).' AFTER dispatch_id,
    ADD KEY idx_orders_dispatch_id (dispatch_id),
    ADD CONSTRAINT fk_orders_dispatch_id FOREIGN KEY (dispatch_id) REFERENCES dispatches (id) ON DELETE SET NULL;
```

## PostgreSQL spelling

PostgreSQL has no `AFTER` and always appends a new column at the end, so the column-ordering invariant cannot be kept
by an `ALTER TABLE` on that engine. Order the columns of the `ALTER` by § Column ordering among themselves and accept
the physical position. The secondary index is its own `CREATE INDEX` statement, and each new column takes its own
`COMMENT ON COLUMN` statement.

```sql
ALTER TABLE tax_periods
    ADD COLUMN daily_minimum_amount INTEGER NULL,
    ADD CONSTRAINT chk_tax_periods_daily_minimum_amount CHECK (daily_minimum_amount >= 0);

COMMENT ON COLUMN tax_periods.daily_minimum_amount IS '[NONE] The least a charged vehicle pays in one day, in whole units of the currency of the city (e.g., 10).';
```

## Evolving a closed set

On `mysql`, when the change adds a value to an `ENUM` column, use `MODIFY COLUMN` restating the full definition
(including the generation expression when the column is generated, and the `COMMENT` with its classification tag), and
append the new value at the END of the list. Never reorder or remove existing values: appending preserves the stored
ordinal of every existing value, and on a non-generated column it keeps the change eligible for the in-place algorithm.

The PostgreSQL spelling of a closed set is a `CHECK (<column> IN (...))` constraint, so adding a value drops the
constraint and adds it again under the same name with the value appended, in one migration. Removing a value is a
destructive change and follows the two-phase sequence in `database-migration` § Migration policy.

```sql
ALTER TABLE exempt_vehicles
    DROP CONSTRAINT chk_exempt_vehicles_vehicle_type,
    ADD CONSTRAINT chk_exempt_vehicles_vehicle_type CHECK (vehicle_type IN ('BUS', 'CAR', 'FOREIGN', 'DIPLOMAT', 'MILITARY', 'EMERGENCY', 'MOTORCYCLE', 'TRACTOR'));
```

On either engine, update the `(e.g., ...)` list in the column comment to include the new value in its lifecycle
position, which in PostgreSQL is a new `COMMENT ON COLUMN` statement in the same migration.
