mod config;
mod md_dev;
mod ptr_hash;

use std::{
    collections::HashSet,
    os::unix::ffi::OsStrExt,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use figment::providers::Format;
use ptr_hash::PtrHash;
use sysinfo::{ProcessRefreshKind, RefreshKind};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or("mdcheck-ng.toml".to_string());

    let config: config::Config = figment::Figment::new()
        .admerge(figment::providers::Toml::file(config_path))
        .admerge(figment::providers::Env::prefixed("MD_CHECK"))
        .extract()?;

    if !config.runs_now() {
        log::debug!("Not in the allowed run interval, exiting");
        return Ok(());
    }

    let terminated = Arc::new(AtomicBool::new(false));
    let main_thread = std::thread::current();

    ctrlc::set_handler({
        let terminated = terminated.clone();
        move || {
            terminated.store(true, Ordering::Release);
            main_thread.unpark();
        }
    })
    .expect("Error setting Ctrl-C handler");

    let all_md_devs = md_dev::MdDev::find()?;

    let mut active_md_devs = HashSet::new();

    for i in &all_md_devs {
        let dev = i.name();
        if !matches!(&*i.sync_action()?, "idle") {
            log::info!("{dev} is busy");
            continue;
        }
        let schedule = config.get(dev);
        if let Some(state) = i.state()? {
            if schedule.resume() {
                i.resume(state)?;
                active_md_devs.insert(PtrHash::from(i));
            }
        } else if schedule.start() {
            i.start()?;
            active_md_devs.insert(PtrHash::from(i));
        }
    }

    while !terminated.load(Ordering::Acquire) && !active_md_devs.is_empty() {
        for i in active_md_devs.clone() {
            let dev = i.name();
            if matches!(&*i.sync_action()?, "check") {
                log::debug!("{dev} is still checking");
                if let Some(completed) = i.sync_completed()? {
                    log::debug!("Save state for {dev}");
                    i.save_state(completed)?;
                }
                let schedule = config.get(dev);

                // renice the sync process
                let sys = sysinfo::System::new_with_specifics(
                    RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
                );
                if let Some(p) = sys
                    .processes_by_exact_name(std::ffi::OsStr::from_bytes(
                        format!("{dev}_resync").as_bytes(),
                    ))
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

                if !schedule.runs_now() {
                    i.stop()?;
                    active_md_devs.remove(&i);
                }
            } else {
                // either finished or went into resync. Either way not our
                // responsibility any more.
                log::debug!("Clean state for {dev}");
                i.clear_state()?;
                active_md_devs.remove(&i);
            }
        }
        std::thread::park_timeout(Duration::from_secs(120));
    }

    log::info!("Running clean-up");

    // stop any checks that are still running
    for i in active_md_devs {
        let dev = i.name();
        if matches!(&*i.sync_action()?, "check") {
            log::debug!("{dev} is still checking");
            i.stop()?;
        } else {
            // finished or went into resync since we checked last time.
            log::debug!("Clean state for {dev}");
            i.clear_state()?;
        }
    }
    Ok(())
}
