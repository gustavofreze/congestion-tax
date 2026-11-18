use crate::application::domain::errors::CityCannotBeEmpty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct City(String);

impl City {
    pub fn new(name: &str) -> Result<Self, CityCannotBeEmpty> {
        let normalized = name.trim().to_lowercase();

        if normalized.is_empty() {
            return Err(CityCannotBeEmpty);
        }

        Ok(Self(normalized))
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}
