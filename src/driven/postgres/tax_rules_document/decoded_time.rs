use jiff::civil::Time;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub(super) struct DecodedTime(pub(super) Time);

impl TryFrom<String> for DecodedTime {
    type Error = jiff::Error;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        raw.parse().map(Self)
    }
}
