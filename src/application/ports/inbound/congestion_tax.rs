use super::DailyCongestionTax;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CongestionTax {
    pub days: Vec<DailyCongestionTax>,
    pub amount: u32,
    pub currency: String,
}
