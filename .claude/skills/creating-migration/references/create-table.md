<!-- Vendored from payment (.claude/skills/creating-migration/references/create-table.md) on 2026-11-18. Adapted to this project: the PostgreSQL spelling, as database/migrations/V0001__Create_tax_periods_table.sql writes it, sits beside the MySQL one. -->

# Create table

Use this when a migration creates a single table with foreign keys, explicit indexes, unique and check constraints, and
the engine block or the comment statements.

Portable, and holding on any engine: the column ordering, the classification tag opening every column comment, the
vertical alignment of the column definitions, the one space after each index and constraint keyword, the closing `)`
alone on its line, and the naming patterns (`database-migration` § Statement shape and § Index and constraint naming).
Contextual, and changing with the engine: the type spellings (`BINARY(16)`, `DATETIME(6)`, `INT UNSIGNED`, `ENUM` on
`mysql`) and the table-level clauses, whose portable intents are tabulated in `database-migration` § Column types and
engine.

## `mysql` spelling

```sql
-- Canonical CREATE TABLE shape. One CREATE TABLE per migration file.
-- Column order: id first, non-DATETIME business columns grouped by prefix,
-- business DATETIME(6) columns, then audit columns last. Align definitions
-- vertically. Every column carries a classification tag in its COMMENT.
-- Every foreign key column gets an explicit idx_<table>_<column> immediately
-- above its fk_<table>_<column> constraint.
-- Vertical alignment applies to the column definitions only. The index and
-- constraint entries below them take exactly one space after their keyword,
-- and the closing ) sits alone on its line with the engine block indented
-- four spaces under it (database-migration, § Statement shape).

CREATE TABLE orders
(
    id          BINARY(16)                                          NOT NULL COMMENT '[NONE] The order identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).',
    price       INT UNSIGNED                                        NOT NULL COMMENT '[NONE] The order price in the minor unit of the order currency (e.g., 15000).',
    status      ENUM ('PLACED', 'CONFIRMED', 'SHIPPED', 'CANCELED') NOT NULL COMMENT '[NONE] The order status (e.g., PLACED, CONFIRMED, SHIPPED, CANCELED).',
    currency    VARCHAR(3)                                          NOT NULL COMMENT '[NONE] The order currency in ISO 4217 format (e.g., BRL).',
    quantity    INT                                                 NOT NULL COMMENT '[NONE] The number of units ordered (e.g., 2).',
    product_id  BINARY(16)                                          NOT NULL COMMENT '[NONE] The product identifier in Version 7 UUID format (e.g., 01912d40-1b2c-73d4-85e6-7f8091a2b3c4).',
    customer_id BINARY(16)                                          NOT NULL COMMENT '[NONE] The customer identifier in Version 7 UUID format (e.g., 01912d41-2c3d-74e5-96f7-8091a2b3c4d5).',
    external_id VARCHAR(64)                                         NULL COLLATE utf8mb4_bin COMMENT '[NONE] The provider order identifier used for webhook correlation (e.g., acme_9f2c4d81).',
    created_at  DATETIME(6)                                         NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).',
    updated_at  DATETIME(6)                                         NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6) COMMENT '[NONE] The UTC date and time when the record was last updated in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).',
    PRIMARY KEY (id),
    KEY idx_orders_product_id (product_id),
    CONSTRAINT fk_orders_product_id FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE RESTRICT,
    KEY idx_orders_customer_id (customer_id),
    CONSTRAINT fk_orders_customer_id FOREIGN KEY (customer_id) REFERENCES customers (id) ON DELETE RESTRICT,
    CONSTRAINT unq_orders_external_id UNIQUE (external_id),
    CONSTRAINT chk_orders_quantity CHECK (quantity > 0)
)
    ENGINE = InnoDB
    DEFAULT CHARSET = utf8mb4
    COLLATE = utf8mb4_0900_ai_ci
    COMMENT = 'Table used to persist orders.';
```

## PostgreSQL spelling

The table body carries the columns and the constraints only. The statement terminator follows the closing paren, then
each secondary index is its own `CREATE INDEX` statement, then the `COMMENT ON TABLE` statement and one
`COMMENT ON COLUMN` statement per column in column order. The identifier is `UUID`, a closed set is a `VARCHAR` with a
`CHECK (... IN ...)`, money is an `INTEGER` with a non-negative `CHECK`, a set of small numbers is a `SMALLINT[]` with a
containment `CHECK (<column> <@ '{...}'::SMALLINT[])`, and every instant is `TIMESTAMPTZ(6)`. This is the shape of
`database/migrations/V0001__Create_tax_periods_table.sql`.

```sql
CREATE TABLE tax_periods
(
    id                           UUID           NOT NULL,
    tax_rule_id                  UUID           NOT NULL,
    toll_free_months             SMALLINT[]     NOT NULL,
    toll_free_weekdays           SMALLINT[]     NOT NULL,
    toll_free_day_before_holiday BOOLEAN        NOT NULL,
    daily_maximum_amount         INTEGER        NOT NULL,
    single_charge_window_minutes SMALLINT       NOT NULL,
    starts_on                    DATE           NOT NULL,
    ends_on                      DATE           NOT NULL,
    created_at                   TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT unq_tax_periods_tax_rule_id_starts_on UNIQUE (tax_rule_id, starts_on),
    CONSTRAINT fk_tax_periods_tax_rule_id FOREIGN KEY (tax_rule_id) REFERENCES tax_rules (id) ON DELETE CASCADE,
    CONSTRAINT chk_tax_periods_toll_free_months CHECK (toll_free_months <@ '{1,2,3,4,5,6,7,8,9,10,11,12}'::SMALLINT[]),
    CONSTRAINT chk_tax_periods_toll_free_weekdays CHECK (toll_free_weekdays <@ '{1,2,3,4,5,6,7}'::SMALLINT[]),
    CONSTRAINT chk_tax_periods_daily_maximum_amount CHECK (daily_maximum_amount >= 0),
    CONSTRAINT chk_tax_periods_single_charge_window_minutes CHECK (single_charge_window_minutes >= 0),
    CONSTRAINT chk_tax_periods_ends_on CHECK (ends_on >= starts_on)
);

CREATE INDEX idx_tax_periods_tax_rule_id ON tax_periods (tax_rule_id);

COMMENT ON TABLE tax_periods IS 'Table used to persist the tax rules a city applies between two dates.';
COMMENT ON COLUMN tax_periods.id IS '[NONE] The tax period identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN tax_periods.tax_rule_id IS '[NONE] The identifier of the tax rules the row belongs to in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN tax_periods.toll_free_months IS '[NONE] The months of the year, numbered from 1 for January, during which no passage is charged (e.g., {7}).';
COMMENT ON COLUMN tax_periods.toll_free_weekdays IS '[NONE] The ISO 8601 weekdays, numbered from 1 for Monday, on which no passage is charged (e.g., {6,7}).';
COMMENT ON COLUMN tax_periods.toll_free_day_before_holiday IS '[NONE] The flag that makes the day before a public holiday toll free as well (e.g., true).';
COMMENT ON COLUMN tax_periods.daily_maximum_amount IS '[NONE] The most a vehicle pays in one day, in whole units of the currency of the city (e.g., 60).';
COMMENT ON COLUMN tax_periods.single_charge_window_minutes IS '[NONE] The minutes after its first charged passage during which a vehicle pays only the highest fee once (e.g., 60).';
COMMENT ON COLUMN tax_periods.starts_on IS '[NONE] The first local date on which the period is in force, included in the period (e.g., 2013-01-01).';
COMMENT ON COLUMN tax_periods.ends_on IS '[NONE] The last local date on which the period is in force, included in the period (e.g., 2013-12-31).';
COMMENT ON COLUMN tax_periods.created_at IS '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
```
