<!-- Vendored from payment (.claude/skills/creating-migration/references/collation.md) on 2026-11-18. Adapted to this project: the PostgreSQL spelling (the database collation and COLLATE "C") sits beside the MySQL one, with the tax_rules city check as the normalized-value example. -->

# Collation decision tree

The portable intent: every text column inherits the table-level (or database-level) collation, and a column whose
values must compare and sort byte-for-byte overrides it with a binary collation on the column itself. That holds on any
engine, and only the collation names change.

On `mysql` the table-level collation is `utf8mb4_0900_ai_ci` (part of the charset and engine block,
`database-migration` § Column types and engine, which the spec at `<spec-root>` overrides where it speaks), and the
binary override is `COLLATE utf8mb4_bin` declared inline. The PostgreSQL spelling has no table-level collation: every
column inherits the database collation, and the binary override is `COLLATE "C"` declared inline on the column. The
decision tree below fixes when a column storing a technical identifier, token, or hash declares the override. The
regulated-column byte-for-byte mandate (`[PII-SENSITIVE]`, `[PCI]`, `[CONFIDENTIAL]`) is owned by database-migration §
Data classification. This file walks the full decision for a given column.

## Decide per column

1. **Binary-stored value** (`BINARY`, `VARBINARY`, such as a `BINARY(16)` UUID on `mysql`, or a `UUID` or `BYTEA` in
   PostgreSQL). It carries no collation. Declare nothing.
2. **Human-readable text** (names, descriptions, free-form labels). Inherit the default collation. Declare no override.
   On `mysql` the default compares accent-insensitive and case-insensitive, which is what you want here.
3. **Technical text where case and accent must be preserved bit-for-bit** (opaque tokens, slugs, external provider keys,
   hashes, API identifiers used for correlation). Declare the binary collation inline on the column. A value the schema
   already normalizes with a `CHECK` (like `city = lower(btrim(city))` in `tax_rules`) needs no override, because the
   constraint fixes its form.
4. **Regulated text** carrying `[PII-SENSITIVE]`, `[PCI]`, or `[CONFIDENTIAL]`. It SHOULD declare the binary collation
   when stored as text, or be stored encrypted or tokenized, depending on the service's data-protection policy.

## Examples

- `name VARCHAR(150) NOT NULL COMMENT '[PII] The customer full name (e.g., Maria Silva).'` keeps the default. It is
  human-readable text.
- `email VARCHAR(255) NOT NULL COLLATE utf8mb4_bin` on `mysql`, and `email VARCHAR(255) COLLATE "C" NOT NULL` in
  PostgreSQL, preserve the address exactly.
- `external_id VARCHAR(64) NULL COLLATE utf8mb4_bin` on `mysql`, and `external_id VARCHAR(64) COLLATE "C" NULL` in
  PostgreSQL, correlate a provider key bit-for-bit.
- `id BINARY(16) NOT NULL` on `mysql`, and `id UUID NOT NULL` in PostgreSQL, carry no collation. They are binary-stored.
