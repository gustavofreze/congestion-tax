---
description: Flyway migrations. The structural residue only, the data-classification tag taxonomy and the every-column-comment-starts-with-one-tag invariant, the .sql column-comment prose rule, the index and constraint naming patterns, the statement-shape invariant. The .sql templates and the column-ordering, collation, and ON DELETE how-to live in the creating-migration skill. The migration policy and the column-type and engine conventions are stated defaults here, overridden by the specification where it speaks, with every engine-specific spelling marked as such in the body.
paths:
    - "**/migrations/*.sql"
    - "**/migrations/**/*.sql"
    - "database/**/*.sql"
---
<!-- Vendored from payment (.claude/rules/database-migration.md) on 2026-11-18. Adapted to this project: the PostgreSQL spelling sits beside every MySQL one, the four-digit version width, the one-table insert file, and the natural-key unique constraint are facts of database/migrations/, the whole-unit amounts follow AGENTS.md, and the connection charset points at rust-hex-sql-queries. -->

# Migrations (Flyway)

This rule keeps the migration invariants and the data-classification taxonomy. The canonical `CREATE TABLE`,
`ALTER TABLE`, and seed shapes, the column-ordering algorithm, the collation decision tree, and the `ON DELETE` aid are
in the `creating-migration` skill. The worked SQL templates live in that skill, and the shape they encode is stated as
an invariant in § Statement shape below, because a migration is most often edited with only this rule loaded.

## Pre-output checklist

1. Every column comment starts with exactly one classification tag (`[PII]`, `[PII-SENSITIVE]`, `[PCI]`,
   `[CONFIDENTIAL]`, `[NONE]`). An untagged column is invalid. See § Data classification. With `<database-engine>` =
   `mysql` the comment is the inline `COMMENT '...'`, and the PostgreSQL spelling is one
   `COMMENT ON COLUMN <table>.<column> IS '...';` statement per column.
2. Every column comment is fluent prose with an `(e.g., ...)` example, and never uses a hyphen as a clause separator.
   See § Column comment prose.
3. Index and constraint names follow the strict patterns (`PRIMARY KEY`, `idx_*`, `unq_*`, `fk_*`, `chk_*`). See § Index
   and constraint naming.
4. The closing `)` sits alone on its line, and every index and constraint entry carries exactly one space after its
   leading keyword. See § Statement shape.

## Data classification

Columns and tables that store regulated or restricted data are tagged in their comment. Every column comment starts
with exactly one of the tags below, before the prose, in bracketed form. An untagged column is invalid. `[NONE]` is the
explicit "no category applies" choice, so a missing tag is never ambiguous (forgotten vs. deliberately non-sensitive).

| Tag               | Meaning                                                                                                                                                                 |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `[PII]`           | Personal data under LGPD Art. 5º I, GDPR Art. 4(1), or CCPA § 1798.140(v) (e.g., name, email, phone, document number, IP address, persistent identifier).               |
| `[PII-SENSITIVE]` | Sensitive personal data under LGPD Art. 5º II, GDPR Art. 9(1), or CCPA § 1798.140(ae) (e.g., race, ethnicity, religion, political opinion, health, biometric, genetic). |
| `[PCI]`           | Payment card data in PCI-DSS scope (e.g., PAN, CVV, expiry). Storage SHOULD be tokenized externally, local copies are tagged.                                           |
| `[CONFIDENTIAL]`  | Non-personal restricted data (e.g., API keys, secrets, internal financial figures).                                                                                     |
| `[NONE]`          | No classification rule applies (e.g., order quantity, status, internal foreign key, currency code, business timestamp).                                                 |

The five tags are regime-neutral. They classify the data, not the statute, so the same tag serves whichever regime
applies to the deployment, and the citations above are anchors rather than a choice of jurisdiction.

When at least one column in a table carries a tag other than `[NONE]`, the table-level comment is prefixed with the
union of those tags in the canonical order above, so the table can be filtered by classification without scanning its
columns. A column carrying `[PII-SENSITIVE]`, `[PCI]`, or `[CONFIDENTIAL]` SHOULD compare and sort byte-for-byte when
stored as text (with `<database-engine>` = `mysql`, that is `COLLATE utf8mb4_bin`, and the PostgreSQL spelling is
`COLLATE "C"`), or be stored encrypted or tokenized, per the service data-protection policy.

## Column comment prose

Every column has a comment describing its purpose with an `(e.g., ...)` example, written as fluent prose following
`'<tag> The <subject> <description> (e.g., <example value>).'`. The prose follows the punctuation default in `AGENTS.md`
§ Global defaults, and on top of it a column comment never uses a hyphen (`-`) as a clause separator: join ideas with
`that`, `which`, `when`, `and`, or a comma. A second sentence may follow the `(e.g., ...).` block to state nullability
or behavior. Always write `e.g.,` with the comma. Hyphens and dashes inside literal example values (UUIDs, ISO
timestamps) are part of the value and are unaffected, and the bracketed classification tag prefix is exempt.

The worked column-comment examples are in the creating-migration skill (`references/create-table.md` and
`references/create-table-classified.md`).

## Index and constraint naming

Index and constraint names follow the strict patterns: `PRIMARY KEY (id)`, single-column index `idx_<table>_<column>`,
composite index `idx_<table>_<col1>_<col2>`, unique `unq_<table>_<column>`, foreign key `fk_<table>_<column>`, check
`chk_<table>_<rule>`. A semantic suffix replaces the joined-name form only when it would exceed 64 characters or the
index covers four or more columns. PostgreSQL truncates an identifier at 63 bytes, so on that engine the semantic suffix
takes over past 63 characters. The composite-naming detail, the redundant-index exception for `UNIQUE` columns, and the
`ON DELETE` action decision aid are in the creating-migration skill.

## Statement shape

Three shape invariants hold in every `.sql` file. The worked templates in the creating-migration skill encode them, and
they are stated here because a migration is most often edited with only this rule loaded.

1. **The closing `)` of a `CREATE TABLE` sits alone on its line.** Nothing is ever joined to it, on any engine. Where
   `<database-engine>` = `mysql`, the engine block that follows (`ENGINE`, `DEFAULT CHARSET`, `COLLATE`, `COMMENT`) is
   indented four spaces under it, and writing `) ENGINE = InnoDB` on one line is the drift form, never the target. The
   PostgreSQL spelling has no engine block: the statement terminator follows the paren directly (`);`), then each
   secondary index is its own `CREATE INDEX` statement, then the `COMMENT ON TABLE` statement and one
   `COMMENT ON COLUMN` statement per column, in column order, each on one line.
2. **Every index and constraint entry carries exactly one space after its leading keyword** (`PRIMARY KEY`, `KEY`,
   `UNIQUE`, `CONSTRAINT`, `INDEX`). Alignment padding inside that block is the drift form. The block is read as a list,
   not as a table.
3. **Column definitions are the exception, and they stay vertically aligned**, type against type and `NOT NULL` against
   `NOT NULL`, exactly as the templates show. The alignment rule applies to columns only, never to the index and
   constraint entries below them.

Nothing in the local gate reads `.sql`: `make review` runs rustfmt and Clippy over the Rust sources only. This section
is the only thing standing between the project and the drift, so treat a deviation as a defect, not as formatting taste.

## Migration policy

The five items below are stated defaults, not deferrals. The specifications at `<spec-root>` override any of them where
they speak, so read the spec before deviating from one. With `<spec-root>` unset these defaults are the whole policy,
and where neither states a rule, ask rather than invent one.

1. **Forward-only.** An already-applied migration is never edited, renumbered, or deleted. A mistake is corrected by a
   new migration stacked on top of it, never by rewriting applied history.
2. **Versioned file naming.** Every change is one versioned Flyway file named `V<number>__<description>.sql` under
   `database/migrations/`. The number is the highest already applied plus one, with no gap and no conflict, and the
   description reads as a summary of that single change. This project pads the number to four digits
   (`V0000__Create_tax_rules_table.sql`), and a new file keeps that width.
3. **One logical change per file.** A file carries one table creation, one alteration, one backfill, or one seed, and
   never a combination of them. A data insert always lives in its own file, separate from the DDL that makes it valid,
   and one insert file targets exactly one table.
4. **Two-phase destructive change.** Every migration is backwards-compatible with the code already running. A
   destructive change (dropping a column or a table, tightening a column to `NOT NULL`, narrowing a type) is split into
   two phases, (1) the code stops using the structure and ships, (2) a later migration removes or tightens it. One risky
   file doing both at once is never the target.
5. **Single-schema isolation.** A service owns exactly one schema and its database user reaches no other. There is no
   cross-schema foreign key, and a reference into another bounded context is a soft reference carrying the identifier
   only.

Table names are `snake_case` and plural. The outbox and idempotency table semantics are owned by the spec at
`<spec-root>` and are not restated here. With `<spec-root>` unset, ask for those semantics instead of inventing them.

## Column types and engine

The intents below are portable and hold on any engine. The spellings beside them are the `<database-engine>` binding,
given for `mysql` and for `postgresql`, and on another engine the same intent takes that engine's nearest equivalent.
The specifications at `<spec-root>` override a spelling where they speak, and with `<spec-root>` unset the table below
is the whole convention.

| Portable intent                                              | `mysql` spelling                                         | PostgreSQL spelling                                       |
|--------------------------------------------------------------|----------------------------------------------------------|-----------------------------------------------------------|
| Fixed-width binary identifier holding a UUIDv7               | `BINARY(16)`                                             | `UUID`                                                    |
| Microsecond-precision UTC instant                            | `DATETIME(6)`                                            | `TIMESTAMPTZ(6)`                                          |
| Non-negative integer money in the minor unit of its currency | `INT UNSIGNED`                                           | `INTEGER` with `CHECK (<column> >= 0)`                    |
| Closed set of values constrained by the schema itself        | `ENUM ('PLACED', 'CONFIRMED')`                           | `VARCHAR(<n>)` with `CHECK (<column> IN ('PLACED', ...))` |
| Append-only surrogate on a high-volume log table             | `BIGINT AUTO_INCREMENT`                                  | `BIGINT GENERATED ALWAYS AS IDENTITY`                     |
| Full Unicode encoding, four-byte characters included         | `DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci` | `UTF8`, the database encoding, no per-table clause        |
| Transactional storage with enforced foreign keys             | `ENGINE = InnoDB`                                        | Every table, no clause                                    |
| Table and column comment                                     | Inline `COMMENT '...'` and `COMMENT = '...'`             | `COMMENT ON TABLE` and `COMMENT ON COLUMN` statements     |

This project charges whole units of the currency of the city, not the minor unit (`AGENTS.md` § Project-specific
rules), so its amount columns hold whole units under the same non-negative `INTEGER` spelling.

Full Unicode is the invariant and it is never optional. An encoding that cannot carry four-byte characters is prohibited
whatever its name on the engine (on `mysql`, `utf8` is an alias of `utf8mb3` and is never the declared charset). The
same encoding is negotiated on the connection, owned by `rust-hex-sql-queries` § Connection encoding (sqlx sends
`client_encoding` `UTF8` at startup).

Every date and time column holds a UTC instant. With `<database-engine>` = `mysql` its type is `DATETIME(6)` and never
`TIMESTAMP(6)`, because `TIMESTAMP` stops at 2038-01-19 and converts every value through the session time zone, while
`DATETIME(6)` stores the value as written from 1000-01-01 to 9999-12-31 for one more byte per column. The type carries
no zone, so every connection writes and reads in UTC, and the service pins its session with `SET time_zone = "+00:00"`.
`DEFAULT CURRENT_TIMESTAMP(6)` and `ON UPDATE CURRENT_TIMESTAMP(6)` remain valid on a `DATETIME(6)` column and take the
session time zone, which is one more reason the session is always pinned to UTC. The PostgreSQL spelling is
`TIMESTAMPTZ(6)` and never `TIMESTAMP(6)`: `TIMESTAMP` without a time zone drops any offset it is given, while
`TIMESTAMPTZ` normalizes every value to UTC on write and has no 2038 limit. A calendar date or a time of day that the
domain states in local terms (a tax period's `starts_on`, a tariff band's `starts_at`) is not an instant, and takes
`DATE` or `TIME` on that engine.

The identifier is generated by the application and never by the database, so an aggregate carries its identity before it
is ever persisted. The append-only surrogate is the exception and not the default: a table whose rows have a semantic
identity takes the fixed-width binary UUIDv7 identifier, and only an append-only, high-volume log table without one
takes the integer surrogate, for cache locality and to keep the clustered index unfragmented. A natural key the domain
already names (the `city` of `tax_rules`, the `(tax_rule_id, starts_on)` of `tax_periods`) is a `unq_*` unique key
beside the `id` primary key, never the primary key itself.

## Canonical shapes

The shapes and how-to are in the creating-migration skill. This covers the `CREATE TABLE`, classified columns,
`ALTER TABLE`, and seed-insert templates, the two-level column-ordering algorithm, the per-column collation decision
tree, and the `ON DELETE` action aid.
