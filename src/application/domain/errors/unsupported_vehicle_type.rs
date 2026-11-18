use thiserror::Error;

#[derive(Debug, Error)]
#[error("Vehicle type <{raw}> is not supported.")]
pub struct UnsupportedVehicleType {
    raw: String,
}

impl UnsupportedVehicleType {
    pub(crate) fn new(raw: &str) -> Self {
        Self { raw: raw.to_owned() }
    }
}
