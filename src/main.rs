mod config;
mod md_dev;

use std::{
    os::unix::ffi::OsStrExt,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use figment::providers::Format;
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

    let mut active_md_devs = vec![];

    for i in &all_md_devs {
        let dev = i.name();
        if !matches!(&*i.sync_action()?, "idle") {
            log::info!("{dev} is busy");
            continue;
        }
        let schedule = config.get(dev);
        if let Some(state) = i.state()? {
            // we can resume where we left off
            i.set_sync_min(state)?;
            if schedule.cont() {
                log::info!("Resuming check of {dev} from {state}");
                i.set_sync_action("check")?;
                active_md_devs.push(i);
            }
        } else if schedule.start() {
            i.set_sync_min(0)?;
            log::info!("Starting check of {dev}");
            i.set_sync_action("check")?;
            active_md_devs.push(i);
        }
    }

    while !terminated.load(Ordering::Acquire) {
        let mut cont = false;
        for i in &active_md_devs {
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
                    if let Some(ionice) = schedule.ionice.as_deref() {
                        log::debug!("Setting {dev} ionice to {}", ionice);
                        let _ = std::process::Command::new("ionice")
                            .args(["-p", &pid.to_string()])
                            .args(ionice.split(' '))
                            .spawn();
                    }
                    if let Some(nice) = schedule.nice {
                        log::debug!("Setting {dev} nice to {}", nice);
                        let _ = std::process::Command::new("renice")
                            .args(["-n", &nice.to_string()])
                            .args(["-p", &pid.to_string()])
                            .stdout(Stdio::null())
                            .spawn();
                    }
                }

                if schedule.runs_now() {
                    // if at least one device should still be checking, continue
                    cont |= true;
                }
            } else {
                log::debug!("Clean state for {dev}");
                i.clear_state()?;
            }
        }
        if !cont {
            break;
        }
        std::thread::park_timeout(Duration::from_secs(120));
    }

    log::info!("Running clean-up");

    // stop any checks that are still running
    for i in &active_md_devs {
        let dev = i.name();
        if matches!(&*i.sync_action()?, "check") {
            log::debug!("{dev} is still checking");
            if let Some(completed) = i.sync_completed()? {
                log::debug!("Save state for {dev}");
                i.save_state(completed)?;
            }

            log::debug!("Stop checking {dev}");
            i.set_sync_action("idle")?;
        } else {
            log::debug!("Clean state for {dev}");
            i.clear_state()?;
        }
    }
    Ok(())
}
