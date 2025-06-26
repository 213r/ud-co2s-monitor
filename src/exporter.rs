mod influxdb;
use influxdb::{InfluxDBError, InfluxDBExporter};

mod file;
use file::FileExporter;

#[derive(Debug, PartialEq)]
pub struct UDCO2SDATA {
    pub timestamp: i64,
    pub co2: f64,
    pub hum: f64,
    pub temp: f64,
}
pub enum UDCO2SExporter {
    Influx(InfluxDBExporter),
    File(FileExporter),
}

#[derive(Debug, thiserror::Error)]
pub enum UDCO2SExporterError {
    #[error("InfluxDB error: {0}")]
    Influx(#[from] InfluxDBError),

    #[error("File export error: {0}")]
    File(#[from] std::io::Error),
}

impl UDCO2SExporter {
    pub fn influx(org: &str, bucket: &str, token: &str) -> Self {
        UDCO2SExporter::Influx(InfluxDBExporter::new(org, bucket, token))
    }

    pub fn file(path: impl AsRef<std::path::Path>) -> Self {
        UDCO2SExporter::File(FileExporter::new(path))
    }

    pub async fn set(&mut self, data: &UDCO2SDATA) -> Result<(), UDCO2SExporterError> {
        match self {
            UDCO2SExporter::Influx(influx) => {
                influx.set(data).await.map_err(UDCO2SExporterError::from)
            }
            UDCO2SExporter::File(file) => file.set(data).await.map_err(UDCO2SExporterError::from),
        }
    }
}
