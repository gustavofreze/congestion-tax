<!-- Vendored from payment (.claude/skills/creating-migration/references/create-table-classified.md) on 2026-11-18. Adapted to this project: the PostgreSQL spelling, with COMMENT ON statements and COLLATE "C", sits beside the MySQL one. -->

# Create table with classified columns

Use this when a new table holds classified columns (PII or regulated data) and needs the table-level tag union in its
comment.

## `mysql` spelling

```sql
-- CREATE TABLE with classified columns. The tag-union prefix rule is owned by
-- database-migration § Data classification, not restated here. This template
-- shows the concrete shape (COMMENT = '[PII] ...'). Text columns holding
-- regulated data declare COLLATE utf8mb4_bin (or are stored encrypted/tokenized
-- per data-protection policy).

CREATE TABLE customers
(
    id         BINARY(16)   NOT NULL COMMENT '[NONE] The customer identifier in Version 7 UUID format (e.g., 01912d41-2c3d-74e5-96f7-8091a2b3c4d5).',
    name       VARCHAR(150) NOT NULL COMMENT '[PII] The customer full name (e.g., Maria Silva).',
    email      VARCHAR(255) NOT NULL COLLATE utf8mb4_bin COMMENT '[PII] The customer email address (e.g., maria.silva@example.com).',
    created_at DATETIME(6)  NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).',
    updated_at DATETIME(6)  NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6) COMMENT '[NONE] The UTC date and time when the record was last updated in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).',
    PRIMARY KEY (id),
    CONSTRAINT unq_customers_email UNIQUE (email)
)
    ENGINE = InnoDB
    DEFAULT CHARSET = utf8mb4
    COLLATE = utf8mb4_0900_ai_ci
    COMMENT = '[PII] Table used to persist customers.';
```

## PostgreSQL spelling

PostgreSQL has no `ON UPDATE CURRENT_TIMESTAMP(6)`. An `updated_at` column is set by the statement that updates the row,
or by a trigger the same migration creates. The regulated text takes `COLLATE "C"` inline, and the tag union opens the
`COMMENT ON TABLE` text.

```sql
CREATE TABLE customers
(
    id         UUID           NOT NULL,
    name       VARCHAR(150)   NOT NULL,
    email      VARCHAR(255)   COLLATE "C" NOT NULL,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT unq_customers_email UNIQUE (email)
);

COMMENT ON TABLE customers IS '[PII] Table used to persist customers.';
COMMENT ON COLUMN customers.id IS '[NONE] The customer identifier in Version 7 UUID format (e.g., 01912d41-2c3d-74e5-96f7-8091a2b3c4d5).';
COMMENT ON COLUMN customers.name IS '[PII] The customer full name (e.g., Maria Silva).';
COMMENT ON COLUMN customers.email IS '[PII] The customer email address (e.g., maria.silva@example.com).';
COMMENT ON COLUMN customers.created_at IS '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
COMMENT ON COLUMN customers.updated_at IS '[NONE] The UTC date and time when the record was last updated in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
```
