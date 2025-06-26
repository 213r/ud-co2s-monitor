mod exporter;
mod parse;
use config::Config;
use exporter::UDCO2SExporter;
use parse::monitor_co2ppm;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InfluxDBConfig {
    org: String,
    bucket: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    path: String,
}

#[derive(Debug, Deserialize)]
struct UDCO2MonitorConfig {
    influxdb: Option<InfluxDBConfig>,
    textfile: Option<FileConfig>,
    port_name: String,
    duration_sec: u64,
}

#[tokio::main]
async fn main() {
    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .set_default("duration_sec", 10)
        .expect("Failed to set default duration")
        .build()
        .expect("Failed to load configuration");
    let config: UDCO2MonitorConfig = config
        .try_deserialize()
        .expect("Failed to parse configuration");
    let mut exporters = Vec::new();
    if let Some(influxdb) = config.influxdb {
        exporters.push(UDCO2SExporter::influx(
            &influxdb.org,
            &influxdb.bucket,
            &influxdb.token,
        ));
    }
    if let Some(file) = config.textfile {
        exporters.push(UDCO2SExporter::file(file.path));
    }
    if exporters.is_empty() {
        eprintln!("No exporters configured. Please check your config file.");
        return;
    }
    monitor_co2ppm(&config.port_name, &mut exporters, config.duration_sec).await
}
