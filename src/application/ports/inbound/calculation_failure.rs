use thiserror::Error;

use super::TaxRulesNotFound;
use crate::application::domain::errors::{
    CityCannotBeEmpty, PassageFormatNotValid, PassageOutsideTaxPeriods, UnsupportedVehicleType,
};
use crate::application::ports::outbound::TaxRulesUnavailable;

#[derive(Debug, Error)]
pub enum CalculationFailure {
    #[error(transparent)]
    TaxRulesNotFound(#[from] TaxRulesNotFound),
    #[error(transparent)]
    CityCannotBeEmpty(#[from] CityCannotBeEmpty),
    #[error(transparent)]
    TaxRulesUnavailable(#[from] TaxRulesUnavailable),
    #[error(transparent)]
    PassageFormatNotValid(#[from] PassageFormatNotValid),
    #[error(transparent)]
    UnsupportedVehicleType(#[from] UnsupportedVehicleType),
    #[error(transparent)]
    PassageOutsideTaxPeriods(#[from] PassageOutsideTaxPeriods),
}
