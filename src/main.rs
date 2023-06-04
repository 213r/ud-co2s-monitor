use clap::{Arg, Command};
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
    StatusCode
};
use prometheus::{Encoder, Gauge, TextEncoder};
use regex::Regex;
use serialport;
use std::{io, net::SocketAddr, time::Duration};
use chrono::Local;
use lazy_static::lazy_static;
use prometheus::{opts, register_gauge};

lazy_static! {
    static ref CO2PPM_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_co2ppm", "CO2 ppm value",)).unwrap();
    static ref HUM_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_hum", "Humidity value",)).unwrap();
    static ref TEMP_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_temp", "Temperature value",)).unwrap();
}

#[derive(Debug, PartialEq)]
struct UDCO2SDATA {
    co2: f64,
    hum: f64,
    temp: f64,
}

fn parse_udco2s_data(input: &str) -> Option<UDCO2SDATA> {
    // The values for hum, tmp is not trustworthy.
    let re = Regex::new(r"CO2=(?P<co2>[\d.]+),HUM=(?P<hum>[\d.]+),TMP=(?P<temp>[\d.]+)").unwrap();
    match re.captures(input) {
        Some(caps) => {
            let data = UDCO2SDATA {
                co2: caps["co2"].parse().unwrap(),
                hum: caps["hum"].parse().unwrap(),
                temp: caps["temp"].parse().unwrap(),
            };
            Some(data)
        }
        None => None,
    }
}

async fn monitor_co2ppm(port_name: String) {
    const BAUD_RATE: u32 = 115200;
    let mut port = serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_secs(10))
        .open()
        .map_err(|_| eprintln!("Can not open serial. THe UD-CO2S device path may be wrong"))
        .unwrap();
    port.write("STA\r\n".as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; 100];
    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(t) => {
                let bytes = &buf[..t];
                let data = String::from_utf8(bytes.to_vec()).unwrap();
                let data = parse_udco2s_data(&data);
                if let Some(data) = data {
                    let generated_at = Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
                    println!("[{generated_at}] {data:?}");
                    CO2PPM_GAUGE.set(data.co2);
                    HUM_GAUGE.set(data.hum);
                    TEMP_GAUGE.set(data.temp);
                } else {
                    eprintln!("Parsing is failed.");
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                eprintln!("{:?}", e);
                break;
            }
            Err(e) => eprintln!("{:?}", e),
        }
        //std::thread::sleep(Duration::from_secs(5));
    }
}

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

    tokio::spawn(monitor_co2ppm(port_name));

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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_udco2s_data() {
        // Valid input
        let input = "CO2=400.0,HUM=50.0,TMP=25.0";
        let expected_output = Some(UDCO2SDATA {
            co2: 400.0,
            hum: 50.0,
            temp: 25.0,
        });
        assert_eq!(parse_udco2s_data(input), expected_output);

        // Invalid input
        let invalid_input = "CO2=invalid,HUM=50.0,TMP=25.0";
        let expected_output = None;
        assert_eq!(parse_udco2s_data(invalid_input), expected_output);
    }
}
