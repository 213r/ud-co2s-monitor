use clap::{Arg, Command};
use config::{Config, File};
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use serde_derive::{Deserialize, Serialize};
use std::fs::read;
use tokio;
use ud_co2s_monitor::ud_co2s::awsiot_export_udco2s;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
struct AWSIOTconfig {
    endpoint: String,
    ca_path: String,
    cert_path: String,
    key_path: String,
    client_id: String,
    topic: String,
}

#[tokio::main]
async fn main() {
    let matches = Command::new("UD-CO2S Monitor")
        .about("UD-CO2S Monitor")
        .disable_version_flag(true)
        .arg(
            Arg::new("port")
                .long("port")
                .help("The UD-CO2S device path for serial port. For Mac /dev/cu.usbmodem***. ")
                .default_value("/dev/ttyACM0"),
        )
        .arg(
            Arg::new("awsiot_config")
                .long("awsiot_config")
                .help("The path of awsiot config file")
                .default_value("awsiot.toml"),
        )
        .get_matches();
    let config_path = matches.get_one::<String>("awsiot_config").unwrap();
    let cfg = Config::builder()
        .add_source(File::with_name(config_path))
        .build()
        .unwrap();
    let cfg: AWSIOTconfig = cfg.try_deserialize::<AWSIOTconfig>().unwrap();
    dbg!(&cfg);
    let mqtt_options = MqttOptions::new(cfg.client_id, cfg.endpoint, 8883)
        .set_ca(read(cfg.ca_path).unwrap())
        .set_client_auth(read(cfg.cert_path).unwrap(), read(cfg.key_path).unwrap())
        .set_keep_alive(10)
        .set_reconnect_opts(ReconnectOptions::Always(5));

    let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
    mqtt_client.subscribe(&cfg.topic, QoS::AtLeastOnce).unwrap();
    let port_name = matches.get_one::<String>("port").unwrap().clone();
    let duration_sec = 5;
    let _job = tokio::spawn(awsiot_export_udco2s(
        port_name,
        duration_sec,
        mqtt_client,
        cfg.topic,
    ));
    for notification in notifications {
        println!("nf: {:?}", notification)
    }
}
