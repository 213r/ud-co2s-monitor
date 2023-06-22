use clap::{Arg, Command};
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
    StatusCode
};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr; 
use ud_co2s_monitor::ud_co2s::prometheus_export_udco2s; 

async fn serve_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.uri().path() != "/metrics" {
        return  Ok(Response::builder()
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
            Arg::new("addr")
                .long("addr")
                .help("The address on which prometheus server listens")
                .default_value("127.0.0.1:9233"),
        )
        .get_matches();
    let port_name = matches.get_one::<String>("port").unwrap().clone();

    tokio::spawn(prometheus_export_udco2s(port_name, 5));
    
    let addr = matches
        .get_one::<String>("addr")
        .unwrap()
        .parse::<SocketAddr>()
        .map_err(|_| eprintln!("The address is invalid."))
        .unwrap();
    println!("Listening on http://{}", addr);

    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(serve_req))
    }));

    if let Err(err) = serve_future.await {
        eprintln!("server error: {}", err);
    }
}
