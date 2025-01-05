use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::process::Stdio;

use sysinfo::ProcessRefreshKind;
use sysinfo::RefreshKind;

use crate::config::device::DeviceConfig;

pub fn renice(dev: &str, schedule: &DeviceConfig) -> anyhow::Result<()> {
    // renice the sync process
    let sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    if let Some(p) = sys
        .processes_by_exact_name(OsStr::from_bytes(format!("{dev}_resync").as_bytes()))
        .next()
    {
        let pid = p.pid();
        if let Some(ionice) = schedule.ionice() {
            log::debug!("Setting {dev} ionice to {}", ionice);
            let _ = std::process::Command::new("ionice")
                .args(["-p", &pid.to_string()])
                .args(ionice.split(' '))
                .spawn();
        }
        if let Some(nice) = schedule.nice() {
            log::debug!("Setting {dev} nice to {}", nice);
            let _ = std::process::Command::new("renice")
                .args(["-n", &nice.to_string()])
                .args(["-p", &pid.to_string()])
                .stdout(Stdio::null())
                .spawn();
        }
    }
    Ok(())
}
