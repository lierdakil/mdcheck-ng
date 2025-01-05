pub mod cron;
mod device;

use std::collections::HashMap;

use device::DeviceConfig;

#[derive(serde::Deserialize, Debug)]
pub(crate) struct Config {
    #[serde(flatten)]
    pub(crate) schedule: DeviceConfig,
    #[serde(default, flatten)]
    pub(crate) devices: HashMap<String, DeviceConfig>,
}

impl Config {
    pub fn get(&self, name: &str) -> DeviceConfig {
        let main = &self.schedule;
        let dev = self.devices.get(name).unwrap_or(main);
        dev.admerge(main)
    }

    pub fn runs_now(&self) -> bool {
        std::iter::once(&self.schedule)
            .chain(self.devices.values())
            .any(|x| x.runs_now())
    }
}
