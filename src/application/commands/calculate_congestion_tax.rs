#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculateCongestionTax {
    pub city: String,
    pub vehicle: String,
    pub passages: Vec<String>,
}
