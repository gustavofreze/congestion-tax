use crate::environment_variable::EnvironmentVariable;

pub(crate) struct DatabaseSettings {
    pub(crate) host: String,
    pub(crate) name: String,
    pub(crate) port: u16,
    pub(crate) user: String,
    pub(crate) password: String,
}

impl DatabaseSettings {
    pub(crate) fn from_environment() -> anyhow::Result<Self> {
        Ok(Self {
            host: EnvironmentVariable::from("DATABASE_HOST")?.text(),
            name: EnvironmentVariable::from("DATABASE_NAME")?.text(),
            port: EnvironmentVariable::from("DATABASE_PORT")?.parsed()?,
            user: EnvironmentVariable::from("DATABASE_USER")?.text(),
            password: EnvironmentVariable::from("DATABASE_PASSWORD")?.text(),
        })
    }
}
