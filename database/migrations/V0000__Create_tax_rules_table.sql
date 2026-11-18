CREATE TABLE tax_rules
(
    id         UUID           NOT NULL,
    city       VARCHAR(64)    NOT NULL,
    currency   VARCHAR(3)     NOT NULL,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT unq_tax_rules_city UNIQUE (city),
    CONSTRAINT chk_tax_rules_city CHECK (city = lower(btrim(city)) AND city <> ''),
    CONSTRAINT chk_tax_rules_currency CHECK (currency ~ '^[A-Z]{3}$')
);

COMMENT ON TABLE tax_rules IS 'Table used to persist the cities that charge a congestion tax.';
COMMENT ON COLUMN tax_rules.id IS '[NONE] The tax rules identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN tax_rules.city IS '[NONE] The city the tax rules apply to, in lowercase, as callers name it (e.g., gothenburg).';
COMMENT ON COLUMN tax_rules.currency IS '[NONE] The ISO 4217 code of the currency every amount of the city is charged in (e.g., SEK).';
COMMENT ON COLUMN tax_rules.created_at IS '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
