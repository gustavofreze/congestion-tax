<!-- Vendored from payment (.claude/skills/php-creating-driven/references/write-sql.md) on 2026-11-18. Adapted to this project: the MySQL spelling (UUID_TO_BIN, named binds, a qualified SET) became the PostgreSQL one (native UUID, positional binds, an unqualified SET), and the shapes stay placeholders because this project's adapter writes nothing. -->

# Write-side SQL shapes

Use for the INSERT, UPDATE, and DELETE shapes on a tenant-scoped table, each carrying that table's tenant discriminator,
with UUID values bound as native `UUID` values.

Two invariants live in these shapes and they are not the same claim. The mandatory WHERE on UPDATE and DELETE always
holds, on every table, in every service. The tenant scope holds only where there is a tenant: in a single-tenant
service, or on a table with no tenant discriminator, the scope column is dropped from the shapes below and the scoping
gate is skipped rather than failed. This project is single-tenant and its adapter only reads, because the tax rules are
content seeded by migrations, so the shapes below use placeholder names (`organization_id` stands for a tenant
discriminator).

```sql
-- Write-side SQL shapes. Pure SQL without the surrounding Rust constant. When placed in a query
-- constant, the SQL body sits flush left inside the raw string literal (see query-constants.md).
-- INSERT lists columns explicitly, ordered by name length ascending with id first. UPDATE aligns
-- the = signs vertically inside SET. DELETE always carries a WHERE. A UUID column takes the bound
-- value as is, with no conversion function. The table's tenant discriminator is present in every
-- INSERT column list and in the WHERE of every UPDATE and DELETE on a tenant-scoped table. A
-- single-tenant service, or a table without one, has no such column, and the mandatory WHERE on
-- UPDATE and DELETE still holds without it.

-- ---------------------------------------------------------------------------------------------
-- INSERT. Columns listed explicitly, id first then ascending by name length. Positional binds.
-- A start and end pair (starts_at, ends_at) stays adjacent, start first, whatever its length.

INSERT INTO payments (id, amount, status, currency, provider, charge_id, organization_id)
VALUES ($1, $2, $3, $4, $5, $6, $7)

-- ---------------------------------------------------------------------------------------------
-- UPDATE. The = signs align vertically inside SET. PostgreSQL refuses a table alias on a SET
-- column, so the SET columns stay unqualified while WHERE uses the alias. WHERE is mandatory and
-- carries organization_id.

UPDATE payments AS pay
SET status     = $1,
    provider   = $2,
    updated_at = $3
WHERE pay.id = $4
  AND pay.organization_id = $5

-- ---------------------------------------------------------------------------------------------
-- DELETE. Always a WHERE clause (unconditional deletes are prohibited), scoped by organization_id.

DELETE
FROM payments AS pay
WHERE pay.id = $1
  AND pay.organization_id = $2
```
