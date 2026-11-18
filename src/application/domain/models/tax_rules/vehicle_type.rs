use std::str::FromStr;

use crate::application::domain::errors::UnsupportedVehicleType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VehicleType {
    Bus,
    Car,
    Foreign,
    Diplomat,
    Military,
    Emergency,
    Motorcycle,
}

impl FromStr for VehicleType {
    type Err = UnsupportedVehicleType;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "BUS" => Ok(Self::Bus),
            "CAR" => Ok(Self::Car),
            "FOREIGN" => Ok(Self::Foreign),
            "DIPLOMAT" => Ok(Self::Diplomat),
            "MILITARY" => Ok(Self::Military),
            "EMERGENCY" => Ok(Self::Emergency),
            "MOTORCYCLE" => Ok(Self::Motorcycle),
            _ => Err(UnsupportedVehicleType::new(raw)),
        }
    }
}
