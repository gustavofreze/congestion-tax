<!-- Vendored from payment (.claude/skills/creating-migration/references/seed-insert.md) on 2026-11-18. Adapted to this project: the PostgreSQL spelling, as database/migrations/V0007__Insert_tariff_bands.sql writes it, sits beside the MySQL one, and the tax-rules content seeds are named with their foreign key order. -->

# Create table with seed

Use this when a newly created table ships with bootstrap rows, or when an existing table receives new content (a city, a
tax period, a year of holidays). The CREATE TABLE and the seed are two migration files, the seed numbered right after
the table it fills, because a data insert always lives in its own file (`database-migration` § Migration policy).

Seed only non-sensitive reference data. Never insert real personal or secret values (emails, passwords, tokens, document
numbers, card data) in a seed or insert migration: the file is committed to version control and replayed in every
environment, so any secret in it leaks. Real per-user and secret data is created at runtime, never seeded.

## `mysql` spelling

```sql
-- V<n>__Create_roles_table.sql. The table alone, with no INSERT in the file.

CREATE TABLE roles
(
    id         BINARY(16)   NOT NULL COMMENT '[NONE] The role identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).',
    name       VARCHAR(50)  NOT NULL COMMENT '[NONE] The role name (e.g., admin, member, viewer).',
    created_at DATETIME(6)  NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).',
    PRIMARY KEY (id),
    CONSTRAINT unq_roles_name UNIQUE (name)
)
    ENGINE = InnoDB
    DEFAULT CHARSET = utf8mb4
    COLLATE = utf8mb4_0900_ai_ci
    COMMENT = 'Table used to persist user roles.';
```

```sql
-- V<n+1>__Seed_roles.sql. Initial data for the table created just before it.
-- When a newly created table ships with bootstrap rows (default roles,
-- canonical currencies, seed configuration), the INSERT statements live in
-- their own migration file, separate from the DDL that makes them valid. The
-- seed file targets exactly one table and contains no schema changes.
--
-- Seed only non-sensitive reference data (roles, currencies, config). Never seed
-- real personal or secret values (emails, passwords, tokens, document numbers,
-- card data), the file is committed and replayed in every environment.
--
-- Seed UUIDs are literal Version 7 values via UUID_TO_BIN('<literal>') without
-- the swap flag, so the migration is deterministic across environments.
--
-- A data-only migration against an already-existing table (backfill, corrective
-- update, late-added seed) has the same shape: its own dedicated file, exactly
-- one table, and no schema changes.

INSERT INTO roles (id, name)
VALUES (UUID_TO_BIN('11111111-1111-7111-8111-111111111111'), 'admin'),
       (UUID_TO_BIN('22222222-2222-7222-8222-222222222222'), 'member'),
       (UUID_TO_BIN('33333333-3333-7333-8333-333333333333'), 'viewer');
```

## PostgreSQL spelling

A `UUID` literal needs no conversion function. Every seed row carries a literal Version 7 `id`, so the migration is
deterministic across environments and a child row references its parent by that literal. A change that seeds several
tables is one file per table, in foreign key order, so a parent row always exists before the file that references it
runs. A new tax period of a city is four or five files, in this order: `tax_rules` (only for a new city),
`tax_periods`, `tariff_bands`, `public_holidays`, `exempt_vehicles`. This is the shape of
`database/migrations/V0007__Insert_tariff_bands.sql`.

```sql
INSERT INTO tariff_bands (id, amount, tax_period_id, starts_at, ends_at)
VALUES ('01a1e6cd-a800-7000-8000-000000000003', 8, '01a1e6cd-a800-7000-8000-000000000002', '06:00', '06:30'),
       ('01a1e6cd-a800-7000-8000-000000000004', 13, '01a1e6cd-a800-7000-8000-000000000002', '06:30', '07:00');
```

A data-only migration against an already-existing table (backfill, corrective update, late-added seed) follows the same
rule: its own dedicated file, exactly one table, and no schema changes.
