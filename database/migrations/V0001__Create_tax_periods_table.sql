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
