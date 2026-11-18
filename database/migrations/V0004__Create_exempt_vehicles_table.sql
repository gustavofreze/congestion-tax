CREATE TABLE exempt_vehicles
(
    id            UUID           NOT NULL,
    vehicle_type  VARCHAR(16)    NOT NULL,
    tax_period_id UUID           NOT NULL,
    created_at    TIMESTAMPTZ(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT unq_exempt_vehicles_tax_period_id_vehicle_type UNIQUE (tax_period_id, vehicle_type),
    CONSTRAINT fk_exempt_vehicles_tax_period_id FOREIGN KEY (tax_period_id) REFERENCES tax_periods (id) ON DELETE CASCADE,
    CONSTRAINT chk_exempt_vehicles_vehicle_type CHECK (vehicle_type IN ('BUS', 'CAR', 'FOREIGN', 'DIPLOMAT', 'MILITARY', 'EMERGENCY', 'MOTORCYCLE'))
);

CREATE INDEX idx_exempt_vehicles_tax_period_id ON exempt_vehicles (tax_period_id);

COMMENT ON TABLE exempt_vehicles IS 'Table used to persist the vehicle types a tax period never charges.';
COMMENT ON COLUMN exempt_vehicles.id IS '[NONE] The exempt vehicle identifier in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN exempt_vehicles.vehicle_type IS '[NONE] The type of the exempt vehicle (e.g., BUS, CAR, FOREIGN, DIPLOMAT, MILITARY, EMERGENCY, MOTORCYCLE).';
COMMENT ON COLUMN exempt_vehicles.tax_period_id IS '[NONE] The identifier of the tax period the row belongs to in Version 7 UUID format (e.g., 0190d09e-a7a8-7e89-b48c-09fff0f0f0f0).';
COMMENT ON COLUMN exempt_vehicles.created_at IS '[NONE] The UTC date and time when the record was created in ISO 8601 format (e.g., 2026-02-13T08:49:44.931408+00:00).';
