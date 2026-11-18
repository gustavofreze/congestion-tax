use serde::Deserialize;

use crate::application::domain::errors::UnsupportedVehicleType;
use crate::application::domain::models::tax_rules::VehicleType;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub(super) struct DecodedVehicleType(pub(super) VehicleType);

impl TryFrom<String> for DecodedVehicleType {
    type Error = UnsupportedVehicleType;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        raw.parse().map(Self)
    }
}
