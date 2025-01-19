use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

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
            log::debug!("Setting {dev} ionice to {:?}", ionice);
            let tgt = ioprio::Target::Process(ioprio::Pid::from_raw(pid.as_u32() as i32));
            let prio = crate::e!(ioprio::get_priority(tgt));
            let new_prio = ioprio::Priority::new(*ionice);
            if prio != new_prio {
                crate::e!(ioprio::set_priority(tgt, new_prio));
            }
        }
        if let Some(nice) = schedule.nice() {
            log::debug!("Setting {dev} nice to {}", nice);
            let nice = i32::from(nice);
            if let Some(rpid) = rustix::process::Pid::from_raw(pid.as_u32() as i32) {
                let cur_nice = crate::e!(rustix::process::getpriority_process(Some(rpid)));
                if nice != cur_nice {
                    crate::e!(rustix::process::setpriority_process(Some(rpid), nice));
                }
            }
        }
    }
    Ok(())
}
