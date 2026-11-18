use sqlx::PgPool;
use sqlx::types::Json;

use super::stored_city_tax_rules::StoredCityTaxRules;
use crate::query::tax_rules::{CityTaxRules, TaxRulesFinding, TaxRulesFindingUnavailable};

const FIND_TAX_RULES_BY_CITY: &str = r"
SELECT json_build_object(
    'city', rules.city,
    'currency', rules.currency,
    'periods', COALESCE((
        SELECT json_agg(json_build_object(
            'starts_on', to_char(periods.starts_on, 'YYYY-MM-DD'),
            'ends_on', to_char(periods.ends_on, 'YYYY-MM-DD'),
            'toll_free_months', periods.toll_free_months,
            'toll_free_weekdays', periods.toll_free_weekdays,
            'toll_free_day_before_holiday', periods.toll_free_day_before_holiday,
            'daily_maximum_amount', periods.daily_maximum_amount,
            'single_charge_window_minutes', periods.single_charge_window_minutes,
            'tariff_bands', COALESCE((
                SELECT json_agg(json_build_object(
                    'amount', bands.amount,
                    'starts_at', to_char(bands.starts_at, 'HH24:MI:SS'),
                    'ends_at', to_char(bands.ends_at, 'HH24:MI:SS')
                ) ORDER BY bands.starts_at)
                FROM tariff_bands AS bands
                WHERE bands.tax_period_id = periods.id
            ), '[]'::json),
            'public_holidays', COALESCE((
                SELECT json_agg(to_char(holidays.holiday, 'YYYY-MM-DD') ORDER BY holidays.holiday)
                FROM public_holidays AS holidays
                WHERE holidays.tax_period_id = periods.id
            ), '[]'::json),
            'exempt_vehicles', COALESCE((
                SELECT json_agg(vehicles.vehicle_type ORDER BY vehicles.vehicle_type)
                FROM exempt_vehicles AS vehicles
                WHERE vehicles.tax_period_id = periods.id
            ), '[]'::json)
        ) ORDER BY periods.starts_on)
        FROM tax_periods AS periods
        WHERE periods.tax_rule_id = rules.id
    ), '[]'::json)
) AS city_tax_rules
FROM tax_rules AS rules
WHERE rules.city = lower(btrim($1))
";

pub struct PostgresTaxRulesFinding {
    pool: PgPool,
}

impl PostgresTaxRulesFinding {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TaxRulesFinding for PostgresTaxRulesFinding {
    async fn find(&self, city: &str) -> Result<Option<CityTaxRules>, TaxRulesFindingUnavailable> {
        let stored = sqlx::query_scalar::<_, Json<StoredCityTaxRules>>(FIND_TAX_RULES_BY_CITY)
            .bind(city)
            .fetch_optional(&self.pool)
            .await
            .map_err(TaxRulesFindingUnavailable::new)?;

        Ok(stored.map(|Json(rules)| CityTaxRules::from(rules)))
    }
}
