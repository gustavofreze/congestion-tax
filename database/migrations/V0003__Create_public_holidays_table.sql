CREATE TABLE public_holidays
(
    id            UUID           NOT NULL,
    tax_period_id UUID           NOT NULL,
    holiday       DATE           NOT NULL,
    created_at    TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT unq_public_holidays_tax_period_id_holiday UNIQUE (tax_period_id, holiday),
    CONSTRAINT fk_public_holidays_tax_period_id FOREIGN KEY (tax_period_id) REFERENCES tax_periods (id) ON DELETE CASCADE
);

CREATE INDEX idx_public_holidays_tax_period_id ON public_holidays (tax_period_id);

COMMENT ON TABLE public_holidays IS 'Table used to persist the public holidays on which a tax period charges no passage.';
COMMENT ON COLUMN public_holidays.id IS '[NONE] The public holiday identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN public_holidays.tax_period_id IS '[NONE] The identifier of the tax period the row belongs to in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN public_holidays.holiday IS '[NONE] The local date of the public holiday, which may fall on the day after the period ends (e.g., 2014-01-01).';
COMMENT ON COLUMN public_holidays.created_at IS '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
