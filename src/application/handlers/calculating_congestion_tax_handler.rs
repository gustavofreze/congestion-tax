use crate::application::commands::CalculateCongestionTax;
use crate::application::domain::models::tax_rules::{City, Passage, Passages, VehicleType};
use crate::application::ports::inbound::{
    CalculatingCongestionTax, CalculationFailure, CongestionTax, DailyCongestionTax, TaxRulesNotFound,
};
use crate::application::ports::outbound::TaxRulesCatalog;

pub struct CalculatingCongestionTaxHandler<C> {
    catalog: C,
}

impl<C: TaxRulesCatalog> CalculatingCongestionTaxHandler<C> {
    #[must_use]
    pub const fn new(catalog: C) -> Self {
        Self { catalog }
    }
}

impl<C: TaxRulesCatalog> CalculatingCongestionTax for CalculatingCongestionTaxHandler<C> {
    async fn calculate(&self, command: CalculateCongestionTax) -> Result<CongestionTax, CalculationFailure> {
        let city = City::new(&command.city)?;
        let vehicle = command.vehicle.parse::<VehicleType>()?;
        let passages = command
            .passages
            .iter()
            .map(|passage| passage.parse::<Passage>())
            .collect::<Result<Passages, _>>()?;

        let Some(rules) = self.catalog.find(&city).await? else {
            return Err(TaxRulesNotFound.into());
        };

        let assessment = rules.tax_for(vehicle, &passages)?;
        let total = assessment.total();

        Ok(CongestionTax {
            days: assessment
                .days()
                .iter()
                .map(|day| DailyCongestionTax {
                    date: day.date().to_string(),
                    amount: day.amount().value(),
                })
                .collect(),
            amount: total.amount().value(),
            currency: total.currency().code().to_owned(),
        })
    }
}
