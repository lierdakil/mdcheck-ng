mod config;
mod md_dev;
mod renice;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use figment::providers::Format;

macro_rules! e {
    ($expr:expr) => {
        match $expr {
            Ok(x) => x,
            Err(e) => {
                log::error!("{} failed: {e}", stringify!($expr));
                Default::default()
            }
        }
    };
}

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

    let mut active_md_devs: Vec<_> = md_dev::MdDev::find()?
        .into_iter()
        .filter_map(|mut md| {
            let dev = md.name();
            if !e!(md.idle()) {
                log::info!("{dev} is busy");
                return None;
            }
            let schedule = config.get(dev);
            let state = match md.state() {
                Ok(x) => x,
                Err(e) => {
                    log::error!("Failed to get state for {dev}, skipping it: {e}");
                    return None;
                }
            };
            if let Some(state) = state {
                if schedule.resume() {
                    if let Err(e) = md.resume(state) {
                        log::error!("Couldn't resume scrub for {}: {e}", md.name());
                    }
                }
            } else if schedule.start() {
                if let Err(e) = md.start() {
                    log::error!("Couldn't start scrub for {}: {e}", md.name());
                }
            }
            md.is_ours().then_some(md)
        })
        .collect();

    while !terminated.load(Ordering::Acquire) && !active_md_devs.is_empty() {
        active_md_devs.retain(|md| {
            let dev = md.name();
            let schedule = config.get(dev);
            if schedule.runs_now() && e!(md.checking()) {
                log::debug!("{dev} is still checking");
                let completed = e!(md.sync_completed()).unwrap_or(0);
                log::debug!("Save state for {dev}");
                e!(md.save_state(completed));

                e!(renice::renice(dev, &schedule));
                true
            } else {
                false
            }
        });
        std::thread::park_timeout(Duration::from_secs(120));
    }

    Ok(())
}
