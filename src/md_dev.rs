use std::path::Path;

use std::path::PathBuf;
#[derive(Debug)]
pub struct MdDev {
    base_path: PathBuf,
    name: String,
    state_path: PathBuf,
    started_by_us: bool,
}

impl Drop for MdDev {
    fn drop(&mut self) {
        if self.is_ours() {
            let active = match self.checking() {
                Ok(x) => x,
                Err(err) => {
                    log::error!(
                        "Couldn't get sync_action for {dev}: {err}",
                        dev = self.name()
                    );
                    return;
                }
            };
            if active {
                if let Err(err) = self.stop() {
                    log::error!("Failed to stop {dev}: {err}", dev = self.name())
                }
            } else if let Err(err) = self.clear_state() {
                log::error!("Failed to clear state for {dev}: {err}", dev = self.name())
            }
        }
    }
}

impl MdDev {
    fn read(base_path: &Path, relative_path: impl AsRef<Path>) -> anyhow::Result<String> {
        Ok(
            String::from_utf8(std::fs::read(base_path.join(relative_path))?)?
                .trim_end()
                .to_string(),
        )
    }

    fn write(
        base_path: &Path,
        relative_path: impl AsRef<Path>,
        data: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        Ok(std::fs::write(
            base_path.join(relative_path),
            format!("{}\n", data.as_ref()),
        )?)
    }

    pub fn state(&self) -> anyhow::Result<Option<usize>> {
        if std::fs::exists(&self.state_path)? {
            // we can resume where we left off
            let start_at: usize = std::str::from_utf8(&std::fs::read(&self.state_path)?)?
                .trim()
                .parse()?;
            Ok(Some(start_at))
        } else {
            Ok(None)
        }
    }

    pub fn save_state(&self, start: usize) -> anyhow::Result<()> {
        std::fs::create_dir_all(
            self.state_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Couldn't get state dir"))?,
        )?;
        std::fs::write(&self.state_path, start.to_string())?;
        Ok(())
    }

    pub fn clear_state(&self) -> anyhow::Result<()> {
        if std::fs::exists(&self.state_path)? {
            log::debug!("Clean state for {dev}", dev = self.name());
            std::fs::remove_file(&self.state_path)?
        }
        Ok(())
    }

    pub fn sync_completed(&self) -> anyhow::Result<Option<usize>> {
        let completed = Self::read(&self.base_path, "sync_completed")?;
        if let Ok(next_start) = completed
            .split_once('/')
            .map(|x| x.0)
            .unwrap_or(&completed)
            .trim()
            .parse()
        {
            Ok(Some(next_start))
        } else {
            log::warn!(
                "Couldn't read sync_completed for {dev}: {completed}",
                dev = self.name()
            );
            Ok(None)
        }
    }

    pub fn sync_action(&self) -> anyhow::Result<String> {
        Self::read(&self.base_path, "sync_action")
    }

    pub fn set_sync_action(&self, data: impl AsRef<str>) -> anyhow::Result<()> {
        Self::write(&self.base_path, "sync_action", data)
    }

    pub fn resume(&mut self, pos: usize) -> anyhow::Result<()> {
        self.set_sync_min(pos)?;
        log::info!("Resuming check of {dev} from {pos}", dev = self.name());
        self.set_sync_action("check")?;
        self.started_by_us = true;
        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.set_sync_min(0)?;
        log::info!("Starting check of {dev}", dev = self.name());
        self.set_sync_action("check")?;
        self.started_by_us = true;
        Ok(())
    }

    pub fn is_ours(&self) -> bool {
        self.started_by_us
    }

    pub fn checking(&self) -> anyhow::Result<bool> {
        Ok(matches!(&*self.sync_action()?, "check"))
    }

    pub fn idle(&self) -> anyhow::Result<bool> {
        Ok(matches!(&*self.sync_action()?, "idle"))
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        let dev = self.name();
        if let Some(completed) = self.sync_completed()? {
            log::debug!("Save state for {dev}");
            self.save_state(completed)?;
        } else {
            log::warn!("Failed to read completion status for {dev}, will save 0 as state");
            self.save_state(0)?;
        }

        log::debug!("Stop checking {dev}");
        self.set_sync_action("idle")?;
        self.started_by_us = false;
        Ok(())
    }

    pub fn set_sync_min(&self, data: usize) -> anyhow::Result<()> {
        Self::write(&self.base_path, "sync_min", data.to_string())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn find() -> anyhow::Result<Vec<Self>> {
        let mut result = vec![];
        let reader = std::fs::read_dir("/sys/block/")?;
        for entry in reader {
            let entry = entry?;
            let mut md_path = entry.path();
            md_path.push("md");
            if !std::fs::exists(&md_path)? {
                continue;
            }
            let meta = std::fs::metadata(&md_path)?;
            if !meta.is_dir() {
                continue;
            }
            let uuid = MdDev::read(&md_path, "uuid")?;
            result.push(MdDev {
                state_path: PathBuf::from(format!("state_{uuid}")),
                base_path: md_path,
                name: entry
                    .file_name()
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Couldn't read device name"))?
                    .to_string(),
                started_by_us: false,
            });
        }
        Ok(result)
    }
}
