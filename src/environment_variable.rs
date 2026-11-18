use std::env;
use std::error::Error;
use std::str::FromStr;

use anyhow::Context;

pub(crate) struct EnvironmentVariable {
    name: &'static str,
    value: String,
}

impl EnvironmentVariable {
    pub(crate) fn from(name: &'static str) -> anyhow::Result<Self> {
        let value = env::var(name).with_context(|| format!("The environment variable <{name}> must be set."))?;

        Ok(Self { name, value })
    }

    pub(crate) fn text(self) -> String {
        self.value
    }

    pub(crate) fn parsed<T>(self) -> anyhow::Result<T>
    where
        T: FromStr,
        T::Err: Error + Send + Sync + 'static,
    {
        self.value
            .parse()
            .with_context(|| format!("The environment variable <{}> holds an invalid value.", self.name))
    }
}
