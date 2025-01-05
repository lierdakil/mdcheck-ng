use croner::Cron;

#[derive(Debug, Clone)]
pub struct ParsedCron(Cron);

impl std::ops::Deref for ParsedCron {
    type Target = Cron;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for ParsedCron {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(ParsedCron(
            Cron::new(&s)
                .with_seconds_required()
                .with_dom_and_dow()
                .parse()
                .map_err(|err| serde::de::Error::custom(err.to_string()))?,
        ))
    }
}
