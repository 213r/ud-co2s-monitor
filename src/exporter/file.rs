use super::UDCO2SDATA;
use chrono::{Local, TimeZone};
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub struct FileExporter {
    path: PathBuf,
}

impl FileExporter {
    pub fn new(path: impl AsRef<Path>) -> Self {
        FileExporter {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub async fn set(&mut self, data: &UDCO2SDATA) -> Result<(), std::io::Error> {
        let localtime = Local.timestamp_opt(data.timestamp, 0).unwrap().to_rfc3339();
        let line = format!(
            "[{}] temp={}, hum={}, co2={}\n",
            localtime, data.temp, data.hum, data.co2
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        file.write_all(line.as_bytes()).await?;

        Ok(())
    }
}
