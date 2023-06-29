use config::{Config, File};
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use prometheus::{Encoder, TextEncoder};
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use serde_derive::{Deserialize, Serialize};
use std::{fs, net::SocketAddr};
use structopt::StructOpt;
use ud_co2s_monitor::ud_co2s::{consumer::*, monitor_co2ppm};

async fn serve_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
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

#[derive(StructOpt, Debug)]
#[structopt(name = "UD-CO2S Monitor", about = "UD-CO2S Monitor")]
pub struct Opt {
    // Add your command line arguments as fields of the structure
    // For example:
    #[structopt(
        long,
        default_value = "/dev/ttyACM0",
        about = "The UD-CO2S device path for serial port. For Mac /dev/cu.usbmodem***"
    )]
    pub port_udco2s: String,

    #[structopt(
        long,
        about = "The address on which prometheus server listens. Ex. 127.0.0.1:9233"
    )]
    pub addr_prometheus: Option<String>,

    #[structopt(long, about = "The path of awsiot config file. Ex. awsiot.toml")]
    pub path_cfg_awsiot: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
struct AWSIOTconfig {
    endpoint: String,
    ca_path: String,
    cert_path: String,
    key_path: String,
    client_id: String,
    topic: String,
}

fn prepare_awsiot_exporter(path_cfg_awsiot: &str) -> UDCO2SAWSIOTExporter {
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
    mqtt_client.subscribe(&cfg.topic, QoS::AtLeastOnce).unwrap();
    let exporter = UDCO2SAWSIOTExporter::new(mqtt_client, cfg.topic);
    exporter
}

const DURATION_SEC: u64 = 5;
#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    if opt.addr_prometheus.is_none() && opt.path_cfg_awsiot.is_none() {
        println!(
            "do Nothing. Set args of eigher 'addr_prometheus' or 'apath_cfg_awsiot', or both."
        );
        return;
    }
    if let Some(path_cfg_awsiot) = opt.path_cfg_awsiot {
        let exporter = prepare_awsiot_exporter(&path_cfg_awsiot);
        exporters.push(Box::new(exporter))
    }
    let addr = addr_prometheus
        .parse::<SocketAddr>()
        .map_err(|_| eprintln!("The address is invalid."))
        .unwrap();
    println!("Prometheus Server: Listening on http://{}", addr);
    let mut exporters: Vec<Box<dyn UDCO2SExporter + Send>> =
        vec![Box::new(UDCO2SPrometheusExporter)];

    tokio::spawn(async move {
        monitor_co2ppm(&opt.port_udco2s.clone(), &mut exporters, DURATION_SEC).await;
    });

    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(serve_req))
    }));

    if let Err(err) = serve_future.await {
        eprintln!("server error: {}", err);
    }
}
