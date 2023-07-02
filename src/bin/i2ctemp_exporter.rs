use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use std::net::SocketAddr;
use structopt::StructOpt;
use ud_co2s_monitor::{
    i2c_temp::{consumer::*, monitor_sht31_temp},
    prepare_mqtt_client, serve_req,
};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "SHT31 Temperature and Humidity Monitor",
    about = "SHT31 Temperature and Humidity Monitor"
)]
pub struct Opt {
    // Add your command line arguments as fields of the structure
    // For example:
    #[structopt(
        long,
        about = "The address on which prometheus server listens. Ex. 127.0.0.1:9233"
    )]
    pub addr_prometheus: Option<String>,

    #[structopt(long, about = "The path of awsiot config file. Ex. awsiot.toml")]
    pub path_cfg_awsiot: Option<String>,

    #[structopt(
        long,
        default_value = "raspi/sht31",
        about = "The topic name of awsiot mqtt"
    )]
    pub topic_awsiot: String,
}

fn prepare_exporters(opt: &Opt) -> Vec<Box<dyn SHT31Exporter + Send>> {
    let mut exporters: Vec<Box<dyn SHT31Exporter + Send>> = Vec::new();
    if let Some(path_cfg_awsiot) = &opt.path_cfg_awsiot {
        let mqtt_client = prepare_mqtt_client(&path_cfg_awsiot, &opt.topic_awsiot);
        let exporter = SHT31AWSIOTExporter::new(mqtt_client, opt.topic_awsiot.to_string());
        exporters.push(Box::new(exporter));
    }
    if opt.addr_prometheus.is_some() {
        exporters.push(Box::new(SHT31PrometheusExporter));
    }
    exporters
}

const DURATION_SEC: u64 = 5;
#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let mut exporters = prepare_exporters(&opt);
    if exporters.is_empty() {
        println!("do Nothing. Set args of eigher 'addr_prometheus' or 'path_cfg_awsiot', or both.");
        return;
    }

    let monitor_future = tokio::spawn(async move {
        monitor_sht31_temp(&mut exporters, DURATION_SEC).await;
    });

    // launch server
    if let Some(addr_prometheus) = opt.addr_prometheus {
        let addr = addr_prometheus
            .parse::<SocketAddr>()
            .map_err(|_| eprintln!("The address is invalid."))
            .unwrap();
        println!("Prometheus Server: Listening on http://{}", addr);

        let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(serve_req))
        }));

        let server_future = tokio::spawn(serve_future);

        match tokio::try_join!(monitor_future, server_future) {
            Ok((_, _)) => {
                // both tasks completed successfully.
            }
            Err(err) => {
                eprintln!("One of the tasks encountered an error: {}", err);
            }
        }
    } else {
        if let Err(err) = monitor_future.await {
            eprintln!("monitor_co2ppm task encountered an error: {}", err);
        }
    }
}
