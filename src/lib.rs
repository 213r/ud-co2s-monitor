pub mod i2c_temp;
pub mod ud_co2s;

use config::{Config, File};
use hyper::{header::CONTENT_TYPE, Body, Request, Response, StatusCode};
use prometheus::{Encoder, TextEncoder};
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use serde_derive::{Deserialize, Serialize};
use std::fs;

pub async fn serve_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.uri().path() != "/metrics" {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".into())
            .unwrap());
    }

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();
    Ok(response)
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
struct AWSIOTconfig {
    endpoint: String,
    ca_path: String,
    cert_path: String,
    key_path: String,
    client_id: String,
}

pub fn prepare_mqtt_client(path_cfg_awsiot: &str, topic_awsiot: &str) -> MqttClient {
    let cfg = Config::builder()
        .add_source(File::with_name(&path_cfg_awsiot))
        .build()
        .unwrap();
    let cfg: AWSIOTconfig = cfg.try_deserialize::<AWSIOTconfig>().unwrap();
    let mqtt_options = MqttOptions::new(cfg.client_id, cfg.endpoint, 8883)
        .set_ca(fs::read(cfg.ca_path).unwrap())
        .set_client_auth(
            fs::read(cfg.cert_path).unwrap(),
            fs::read(cfg.key_path).unwrap(),
        )
        .set_keep_alive(10)
        .set_reconnect_opts(ReconnectOptions::Always(5));

    let (mut mqtt_client, _notifications) = MqttClient::start(mqtt_options).unwrap();
    mqtt_client
        .subscribe(topic_awsiot, QoS::AtLeastOnce)
        .unwrap();
    mqtt_client
}
