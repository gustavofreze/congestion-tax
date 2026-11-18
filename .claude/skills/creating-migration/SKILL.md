---
name: creating-migration
description: Builds a forward-only, one-change-per-file Flyway migration. Use when creating a migration, adding a table or column, altering schema, running a backfill, or seeding rows such as a new city, tariff, or year of holidays.
---
<!-- Vendored from payment (.claude/skills/creating-migration/SKILL.md) on 2026-11-18. Adapted to this project: the php-creating-driven route became rust-creating-driven, the php-reading-spec and php-creating-skeleton routes were dropped, the tax-rules content seeds join the triggers, and every reference carries the PostgreSQL spelling of database/migrations/ beside the MySQL one. -->

# Add migration

Builds a Flyway migration. Routes to the migration rule, and carries the canonical SQL shapes and the generative how-to
that used to live inside the rule.

Target: the change described in the request (e.g. `create tax periods table`, `add the 2014 tax period of gothenburg`).

## When to use

- Create a migration, add a table or column, alter schema, run a backfill, or seed initial rows.
- Add a city, a tariff, or a year of holidays. Tax rules are content, so each of these is a set of insert migrations,
  one file per table (`AGENTS.md` § Project-specific rules).

## When NOT to use

- Write application code around the schema: use the layer action (e.g. `rust-creating-driven`).
- Run the migration: this skill writes the file only, it does not apply it (`make start` and `make tests` apply it).
- No schema change is involved (a pure read slice or in-memory behavior): stay in the layer skills, a migration that
  touches nothing is not warranted.

## Pragmatic stance

Write the smallest forward-only change the request needs, and split rather than combine. One logical change per file,
the schema only ever moves forward, and a destructive or `NOT NULL` change becomes a multiphase sequence (per
`database-migration`) instead of one risky file. Reach for a backfill or a seed only once the DDL that makes it valid is
already in place.

## Rules applied

Load and follow, in this priority. Where a bullet routes to the spec at `<spec-root>`, that spec overrides the stated
default where it speaks, and with `<spec-root>` unset the stated default is the whole rule.

- Migration policy (file numbering, forward-only, one logical change per file, backfill, the two-phase destructive and
  `NOT NULL` sequence): stated in `database-migration` § Migration policy, and overridden by the spec at `<spec-root>`
  where it speaks. Read that spec directly.
- Column types (the UUIDv7 identifier, the microsecond UTC instant, integer money, closed sets): stated in
  `database-migration` § Column types and engine as a portable intent plus its `<database-engine>` spelling, and
  overridden by the spec at `<spec-root>` where it speaks. Money in this project is whole units of the currency of the
  city (`AGENTS.md` § Project-specific rules).
- `database-migration`: the data-classification tags, the column-comment prose, and the index and constraint naming.
- The DDL `CREATE` and `ALTER` shape: `references/` in this skill.

## Shapes and how-to (read when scaffolding)

The literal shapes and decision aids that the rule used to inline live here, surfaced only when this skill runs. Each
reference gives the `mysql` binding of the neutral shape that `database-migration` states (§ Statement shape and §
Column types and engine) and, beside it, the PostgreSQL spelling that `<database-engine>` resolves to here, as the
project's own migrations in `database/migrations/` write it. The shape and the ordering hold unchanged across engines,
and only the type and table-level spellings become that engine's equivalents.

- `references/create-table.md`: the canonical `CREATE TABLE` (with foreign keys, explicit indexes, unique and check
  constraints, and the engine block or the `COMMENT ON` statements).
- `references/create-table-classified.md`: the classified-columns variant (PII tags and the table-level tag union).
- `references/alter-table.md`: the combined `ALTER TABLE` shape with column placement.
- `references/seed-insert.md`: the seed shape for initial data, in its own file after the create, one table per file.
- `references/column-ordering.md`: the two-level prefix grouping algorithm, representative selection, the
  prefix-collision worked example, and placement in `ALTER TABLE`.
- `references/on-delete-actions.md`: the `CASCADE` vs. `RESTRICT` vs. `SET NULL` vs. `NO ACTION` decision aid, with the
  default-to-`RESTRICT` fallback.
- `references/collation.md`: the per-column collation decision tree (binary vs. human-readable vs. technical vs.
  regulated text) and worked examples.

## Assembly order

1. The directory the project's migration runner actually reads. `database/migrations/` is the canonical home, but
   confirm it against the runner's configured location before writing anything (the `Makefile` target that runs the
   migration, the runner's locations variable, the compose mount). Where they disagree, write the file beside the
   migrations the project already applies and report the divergence as a finding. Opening a second directory splits the
   history and produces a file the runner never applies, which surfaces much later as a mystery.
2. Next version number (highest applied plus one, with no gap or conflict).
3. One logical change per file. A change that seeds several tables (a new tax period with its tariff bands, public
   holidays, and exempt vehicles) is one file per table, in foreign key order.
4. SQL per `database-migration` and this skill's `references/`. Start the table or alter from `references/`.
5. Backfill or seed in its own migration, separate from the DDL.

## Completeness gate

- [ ] Forward-only: no already-applied migration was altered.
- [ ] One logical change per file, named per `database-migration` § Migration policy, and per the spec at `<spec-root>`
      where it overrides that default.
- [ ] SQL per `database-migration` and this skill's `references/`.
- [ ] Statement shape per `database-migration` § Statement shape: the closing `)` alone on its line, one space after
      each index and constraint keyword, column definitions vertically aligned.
- [ ] Every column carries its comment with one classification tag (a `COMMENT ON COLUMN` statement in PostgreSQL).
- [ ] Destructive or NOT NULL changes follow the multiphase sequence in `database-migration` § Migration policy, and the
      spec at `<spec-root>` where it overrides that default.

## Does not do

- Does not apply or run the migration: it writes the file only.
- Does not generate the application code that reads or writes the new schema.
