use std::time::{Duration, Instant};

use chrono::Local;

use super::cron::ParsedCron;

use ioprio::{BePriorityLevel, Class as IoPrioClass, RtPriorityLevel};

#[derive(serde::Deserialize)]
#[serde(remote = "IoPrioClass")]
enum IoPrioClassDef {
    #[serde(alias = "realtime")]
    Realtime(#[serde(with = "RtPriorityLevelDef")] RtPriorityLevel),
    #[serde(alias = "best_effort")]
    BestEffort(#[serde(with = "BePriorityLevelDef")] BePriorityLevel),
    #[serde(alias = "idle")]
    Idle,
}

#[derive(serde::Deserialize)]
#[serde(remote = "RtPriorityLevel")]
struct RtPriorityLevelDef(#[serde(getter = "RtPriorityLevel::level")] u8);

impl From<RtPriorityLevelDef> for RtPriorityLevel {
    fn from(value: RtPriorityLevelDef) -> Self {
        Self::from_level(value.0).expect("Realtime level out of bounds")
    }
}

#[derive(serde::Deserialize)]
#[serde(remote = "BePriorityLevel")]
struct BePriorityLevelDef(#[serde(getter = "BePriorityLevel::level")] u8);

impl From<BePriorityLevelDef> for BePriorityLevel {
    fn from(value: BePriorityLevelDef) -> Self {
        Self::from_level(value.0).expect("BestEffort level out of bounds")
    }
}

#[derive(serde::Deserialize, Default, Debug, Clone)]
enum MaybeIoPrioClass {
    #[default]
    Nothing,
    #[serde(untagged)]
    Just(#[serde(with = "IoPrioClassDef")] IoPrioClass),
}

impl MaybeIoPrioClass {
    fn or_else(self, other: impl FnOnce() -> Self) -> Self {
        match self {
            MaybeIoPrioClass::Just(_) => self,
            MaybeIoPrioClass::Nothing => other(),
        }
    }
}

impl From<MaybeIoPrioClass> for Option<IoPrioClass> {
    fn from(value: MaybeIoPrioClass) -> Self {
        match value {
            MaybeIoPrioClass::Just(class) => Some(class),
            MaybeIoPrioClass::Nothing => None,
        }
    }
}

#[derive(serde::Deserialize, Default, Debug)]
#[serde(default)]
pub struct DeviceConfig {
    start: Option<ParsedCron>,
    r#continue: Option<ParsedCron>,
    ionice: MaybeIoPrioClass,
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

    pub fn ionice(&self) -> Option<&IoPrioClass> {
        match &self.ionice {
            MaybeIoPrioClass::Just(class) => Some(class),
            MaybeIoPrioClass::Nothing => None,
        }
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
