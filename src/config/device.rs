use std::time::{Duration, Instant};

use chrono::Local;

use super::cron::ParsedCron;

#[derive(serde::Deserialize, Default, Debug)]
#[serde(default)]
pub struct DeviceConfig {
    start: Option<ParsedCron>,
    r#continue: Option<ParsedCron>,
    ionice: Option<String>,
    nice: Option<i8>,
    force_run: bool,
    #[serde(default, with = "humantime_serde")]
    max_run_duration: Option<Duration>,
}

impl DeviceConfig {
    pub fn runs_now(&self) -> bool {
        self.start() || self.resume()
    }

    pub fn start(&self) -> bool {
        if self.force_run {
            return true;
        }
        if let Some(start) = &self.start {
            let time = Local::now();
            start.is_time_matching(&time).unwrap()
        } else {
            false
        }
    }
    pub fn resume(&self) -> bool {
        if self.force_run {
            return true;
        }
        if let Some(cont) = &self.r#continue {
            let time = Local::now();
            cont.is_time_matching(&time).unwrap()
        } else {
            self.start()
        }
    }

    pub fn admerge(&self, other: &Self) -> Self {
        Self {
            start: self.start.clone().or_else(|| other.start.clone()),
            r#continue: self.r#continue.clone().or_else(|| other.r#continue.clone()),
            ionice: self.ionice.clone().or_else(|| other.ionice.clone()),
            nice: self.nice.or(other.nice),
            force_run: self.force_run || other.force_run,
            max_run_duration: self.max_run_duration.or(other.max_run_duration),
        }
    }

    pub fn ionice(&self) -> Option<&str> {
        self.ionice.as_deref()
    }

    pub fn nice(&self) -> Option<i8> {
        self.nice
    }

    pub fn below_max_duration(&self, started: Instant) -> bool {
        if let Some(dur) = self.max_run_duration {
            started.elapsed() <= dur
        } else {
            true
        }
    }
}
