CREATE TABLE tariff_bands
(
    id            UUID           NOT NULL,
    amount        INTEGER        NOT NULL,
    tax_period_id UUID           NOT NULL,
    starts_at     TIME           NOT NULL,
    ends_at       TIME           NOT NULL,
    created_at    TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT unq_tariff_bands_tax_period_id_starts_at UNIQUE (tax_period_id, starts_at),
    CONSTRAINT fk_tariff_bands_tax_period_id FOREIGN KEY (tax_period_id) REFERENCES tax_periods (id) ON DELETE CASCADE,
    CONSTRAINT chk_tariff_bands_amount CHECK (amount >= 0),
    CONSTRAINT chk_tariff_bands_ends_at CHECK (ends_at > starts_at)
);

CREATE INDEX idx_tariff_bands_tax_period_id ON tariff_bands (tax_period_id);

COMMENT ON TABLE tariff_bands IS 'Table used to persist the fee a passage costs in each band of the day, per tax period.';
COMMENT ON COLUMN tariff_bands.id IS '[NONE] The tariff band identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN tariff_bands.amount IS '[NONE] The fee a passage within the band costs, in whole units of the currency of the city (e.g., 18).';
COMMENT ON COLUMN tariff_bands.tax_period_id IS '[NONE] The identifier of the tax period the row belongs to in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN tariff_bands.starts_at IS '[NONE] The local time at which the band starts, included in the band (e.g., 07:00:00).';
COMMENT ON COLUMN tariff_bands.ends_at IS '[NONE] The local time at which the band ends, excluded from the band (e.g., 08:00:00).';
COMMENT ON COLUMN tariff_bands.created_at IS '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
