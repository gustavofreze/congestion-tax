use jiff::civil::Date;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub(super) struct DecodedDate(pub(super) Date);

impl TryFrom<String> for DecodedDate {
    type Error = jiff::Error;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        raw.parse().map(Self)
    }
}
