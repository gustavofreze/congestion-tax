use super::{CalculationFailure, CongestionTax};
use crate::application::commands::CalculateCongestionTax;

/// Calculating of the congestion tax a vehicle owes for its passages through the tolling stations of a city.
pub trait CalculatingCongestionTax: Send + Sync {
    /// Calculates the tax the vehicle owes on each day of its passages and the total across those days, in the
    /// currency of the city.
    ///
    /// Fails when the city, the vehicle, or a passage is not valid, when the city has no tax rules, when no tax period
    /// of the city covers a day of the passages, or when the tax rules cannot be read.
    fn calculate(
        &self,
        command: CalculateCongestionTax,
    ) -> impl Future<Output = Result<CongestionTax, CalculationFailure>> + Send;
}
