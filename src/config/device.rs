use chrono::Local;

use super::cron::ParsedCron;

#[derive(serde::Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct DeviceConfig {
    pub(crate) start: Option<ParsedCron>,
    pub(crate) r#continue: Option<ParsedCron>,
    pub(crate) ionice: Option<String>,
    pub(crate) nice: Option<i8>,
}

impl DeviceConfig {
    pub fn runs_now(&self) -> bool {
        self.start() || self.cont()
    }

    pub fn start(&self) -> bool {
        if let Some(start) = &self.start {
            let time = Local::now();
            start.is_time_matching(&time).unwrap()
        } else {
            false
        }
    }
    pub fn cont(&self) -> bool {
        if let Some(cont) = &self.r#continue {
            let time = Local::now();
            cont.is_time_matching(&time).unwrap()
        } else {
            false
        }
    }

    pub fn admerge(&self, other: &Self) -> Self {
        Self {
            start: self.start.clone().or_else(|| other.start.clone()),
            r#continue: self.r#continue.clone().or_else(|| other.r#continue.clone()),
            ionice: self.ionice.clone().or_else(|| other.ionice.clone()),
            nice: self.nice.or(other.nice),
        }
    }
}
