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
        let mut s: String = serde::Deserialize::deserialize(deserializer)?;
        let num_fields = s.split_whitespace().count();
        if num_fields < 6 {
            let prefix = "* ".repeat(6 - num_fields);
            s = format!("{prefix}{s}");
        }
        Ok(ParsedCron(
            Cron::new(&s)
                .with_seconds_required()
                .with_dom_and_dow()
                .parse()
                .map_err(|err| serde::de::Error::custom(err.to_string()))?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_roundtrip() {
        use serde::Deserialize;
        assert_eq!(
            ParsedCron::deserialize(toml::de::ValueDeserializer::new(r#""Sat""#))
                .unwrap()
                .as_str(),
            "* * * * * 6"
        );
        assert_eq!(
            ParsedCron::deserialize(toml::de::ValueDeserializer::new(r#""Sun#1""#))
                .unwrap()
                .as_str(),
            "* * * * * 0#1"
        );
        assert_eq!(
            ParsedCron::deserialize(toml::de::ValueDeserializer::new(r#""Sun#L""#))
                .unwrap()
                .as_str(),
            "* * * * * 0#l"
        );
        assert_eq!(
            ParsedCron::deserialize(toml::de::ValueDeserializer::new(r#""1-7 * Sun""#))
                .unwrap()
                .as_str(),
            "* * * 1-7 * 0"
        );
        assert_eq!(
            ParsedCron::deserialize(toml::de::ValueDeserializer::new(r#""* * * 1-7 * Sun""#))
                .unwrap()
                .as_str(),
            "* * * 1-7 * 0"
        );
    }
}
