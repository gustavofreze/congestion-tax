<!-- Vendored from payment (.claude/skills/php-creating-query/references/select.md) on 2026-11-18. Adapted to this project: the MySQL spelling (BIN_TO_UUID, named binds, JSON path extraction) became the PostgreSQL one (native UUID, positional binds, json_build_object), and the worked projection is FIND_TAX_RULES of src/driven/postgres/postgres_tax_rules_catalog.rs. -->

# Read-side SQL shapes

Use for the single-table SELECT, the SELECT with JOIN, the multi-column ORDER BY, and the JSON document projection on a
read over a tenant-scoped table.

Every read over a tenant-scoped table is filtered by that table's tenant discriminator, and that scope is the
multi-tenant isolation invariant, never an optional filter. The table you are querying tells you whether it has one. In
a single-tenant service, or on a table with no tenant discriminator, the scope predicate is dropped from the shapes
below and the scoping gate is skipped rather than failed. This project is single-tenant. Everything else in the shapes
(the explicit `AS` aliasing, the vertical alignment, the uppercase keywords, the table alias on every column) holds
either way.

UUIDs are stored in the engine's UUID form and exposed textual, and that rule holds on any engine. On
`<database-engine>` (`postgresql`) the column type is `UUID`, it binds and decodes with no conversion function, and a
text projection is `<alias>.id::text`. On `mysql` the same split is `BINARY(16)` with `BIN_TO_UUID` and `UUID_TO_BIN`,
and the storage-versus-exposure split is what must survive.

## Contents

- SELECT (single table)
- SELECT (with JOIN)
- Multi-column ORDER BY
- Worked projection (one JSON document per aggregate)

```sql
-- Read-side SQL shapes. Pure SQL without the surrounding Rust constant. When placed in a query
-- constant, the SQL body sits flush left inside the raw string literal (see the query-constant
-- shape in rust-creating-driven/references/query-constants.md). Every column explicitly aliased
-- with AS, AS aligned vertically, keywords UPPERCASE, every table aliased and every column prefixed,
-- binds positional ($1, $2), and every read over a tenant-scoped table filtered by that table's
-- tenant discriminator (organization_id in the placeholder examples here).

-- ---------------------------------------------------------------------------------------------
-- SELECT (single table). Alias name length ascending, id first. A start and end pair (starts_on,
-- ends_on) stays adjacent, start first, whatever its length.

SELECT periods.id                           AS id,
       periods.starts_on                    AS starts_on,
       periods.ends_on                      AS ends_on,
       periods.tax_rule_id                  AS tax_rule_id,
       periods.daily_maximum_amount         AS daily_maximum_amount,
       periods.single_charge_window_minutes AS single_charge_window_minutes
FROM tax_periods AS periods
WHERE periods.id = $1

-- ---------------------------------------------------------------------------------------------
-- SELECT (with JOIN). INNER JOIN and LEFT JOIN sit flush-left at the same indent as FROM (no
-- continuation indent). The join type is always qualified (INNER or LEFT, never a bare JOIN). The
-- table-name AS aliases align vertically across FROM and every JOIN line. This projection joins
-- three tables and uses semantic column grouping: id first, then identifiers, then names, then amounts.
-- <tenant>_id is the hole for the table's tenant discriminator, whatever that table happens to call
-- it. A table that carries none drops that projected column and that WHERE line.

SELECT <agg>.id                AS id,
       <agg>.<identifier>      AS <identifier>,
       <agg>.<status>          AS <status>,
       <rel>.name              AS <relation>_name,
       <ref>.code              AS <reference>_code,
       <agg>.<tenant>_id       AS <tenant>_id,
       <agg>.<amount>_value    AS <amount>_value,
       <agg>.<amount>_currency AS <amount>_currency
FROM <collection>                  AS <agg>
INNER JOIN <relation_collection>   AS <rel> ON <rel>.id = <agg>.<relation>_id
LEFT JOIN <reference_collection>   AS <ref> ON <ref>.id = <agg>.<reference>_id
WHERE <agg>.id = $1
  AND <agg>.<tenant>_id = $2

-- ---------------------------------------------------------------------------------------------
-- Multi-column ORDER BY. Continuation columns indent to align under the first column. Order columns
-- by name length ascending when it does not alter query behavior (ORDER BY on indexed columns must
-- respect the index order).

SELECT bands.id        AS id,
       bands.amount    AS amount,
       bands.starts_at AS starts_at
FROM tariff_bands AS bands
WHERE bands.tax_period_id = $1
ORDER BY bands.amount,
         bands.starts_at DESC

-- ---------------------------------------------------------------------------------------------
-- Worked projection (FIND_TAX_RULES, one JSON document per aggregate). The column names and the
-- joined tables below are this project's schema, so they are illustrative, never a canonical shape
-- to copy. What generalizes is the composition: one row per aggregate, built with
-- json_build_object, each child collection a correlated json_agg subquery ordered inside the
-- aggregate and defaulted to an empty array with COALESCE, dates and times rendered with to_char
-- in the exact text the decoder parses. The decoded shapes in the adapter turn the document into
-- the typed domain, so the SQL never reaches past the raw columns.

SELECT json_build_object(
    'currency', rules.currency,
    'periods', COALESCE((
        SELECT json_agg(json_build_object(
            'starts_on', to_char(periods.starts_on, 'YYYY-MM-DD'),
            'ends_on', to_char(periods.ends_on, 'YYYY-MM-DD'),
            'daily_maximum_amount', periods.daily_maximum_amount,
            'tariff_bands', COALESCE((
                SELECT json_agg(json_build_object(
                    'amount', bands.amount,
                    'starts_at', to_char(bands.starts_at, 'HH24:MI:SS'),
                    'ends_at', to_char(bands.ends_at, 'HH24:MI:SS')
                ) ORDER BY bands.starts_at)
                FROM tariff_bands AS bands
                WHERE bands.tax_period_id = periods.id
            ), '[]'::json)
        ) ORDER BY periods.starts_on)
        FROM tax_periods AS periods
        WHERE periods.tax_rule_id = rules.id
    ), '[]'::json)
) AS tax_rules_document
FROM tax_rules AS rules
WHERE rules.city = $1
```

The real `FIND_TAX_RULES` also projects the toll free calendar, the public holidays, and the exempt vehicles of each
period. The excerpt above keeps the shape.
