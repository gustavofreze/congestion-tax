<!-- Vendored from payment (.claude/skills/php-creating-driven/references/queries-class.md) on 2026-11-18. Adapted to this project: renamed from queries-class.md, the final readonly Queries class became const raw string literals in the adapter file, the named binds became PostgreSQL positional binds, and the worked example is FIND_TAX_RULES of src/driven/postgres/postgres_tax_rules_catalog.rs. -->

# Query constants shape

Use for the SQL an adapter runs, on the write side (repository) and the read side alike: `const` raw string literals
at the top of the adapter file, ordered by name length ascending.

A constant is not a type, so it lives in the adapter file that runs it and needs no file of its own. Rust has no class
to group the constants in, and an adapter with several of them keeps them together above the struct.

```rust
// The query-constant shape, shared by the write side and the read side. Each statement is a
// const &str in SCREAMING_SNAKE_CASE named after what it does (FIND_TAX_RULES, INSERT_PAYMENT),
// declared at the top of the adapter file and ordered by name length ascending. The SQL body is a raw
// string literal (r"...") so quotes need no escaping, starting on the line after the opening quote
// and written flush left. Dynamic values flow through positional bind parameters ($1, $2) bound with
// .bind(), never format! or interpolation.

const FIND_TAX_RULES: &str = r"
SELECT json_build_object(
    'currency', rules.currency,
    'periods', COALESCE((
        SELECT json_agg(json_build_object(
            'starts_on', to_char(periods.starts_on, 'YYYY-MM-DD'),
            'ends_on', to_char(periods.ends_on, 'YYYY-MM-DD'),
            'daily_maximum_amount', periods.daily_maximum_amount
        ) ORDER BY periods.starts_on)
        FROM tax_periods AS periods
        WHERE periods.tax_rule_id = rules.id
    ), '[]'::json)
) AS tax_rules_document
FROM tax_rules AS rules
WHERE rules.city = $1
";

impl TaxRulesCatalog for PostgresTaxRulesCatalog {
    async fn find(&self, city: &City) -> Result<Option<Arc<TaxRules>>, TaxRulesUnavailable> {
        let decoded = sqlx::query_scalar::<_, Json<DecodedTaxRules>>(FIND_TAX_RULES)
            .bind(city.name())
            .fetch_optional(&self.pool)
            .await
            .map_err(TaxRulesUnavailable::new)?;

        Ok(decoded.map(|Json(DecodedTaxRules(rules))| Arc::new(rules)))
    }
}
```

The real `FIND_TAX_RULES` projects every column of the tax period, its tariff bands, public holidays, and exempt
vehicles. The excerpt above keeps three of them to show the shape.
