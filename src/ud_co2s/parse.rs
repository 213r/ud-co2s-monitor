use super::consumer::UDCO2SExporter;
use chrono::Local;
use regex::Regex;
use serde_derive::Serialize;
use std::io;
use tokio::time::{sleep, Duration};

#[derive(Debug, PartialEq, Serialize)]
pub struct UDCO2SDATA {
    pub timestamp: i64,
    pub co2: f64,
    pub hum: f64,
    pub temp: f64,
}

fn parse_udco2s_data(input: &str) -> Option<UDCO2SDATA> {
    // The values for hum, tmp is not trustworthy.
    let re = Regex::new(r"CO2=(?P<co2>[\d.]+),HUM=(?P<hum>[\d.]+),TMP=(?P<temp>[\d.]+)").unwrap();
    match re.captures(input) {
        Some(caps) => {
            let data = UDCO2SDATA {
                timestamp: Local::now().timestamp(),
                co2: caps["co2"].parse().unwrap(),
                hum: caps["hum"].parse().unwrap(),
                temp: caps["temp"].parse().unwrap(),
            };
            Some(data)
        }
        None => None,
    }
}

pub async fn monitor_co2ppm<T: UDCO2SExporter>(
    port_name: &str,
    consumer: &mut T,
    duration_sec: u64,
) {
    const BAUD_RATE: u32 = 115200;
    let mut port = serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_secs(10))
        .open()
        .expect("Can not open serial. THe UD-CO2S device path may be wrong");

    port.write("STA\r\n".as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; 100];
    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(t) => {
                let bytes = &buf[..t];
                let data = String::from_utf8(bytes.to_vec()).unwrap();
                let data = parse_udco2s_data(&data);
                if let Some(data) = data {
                    println!("{data:?}");
                    consumer.set(&data);
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
        sleep(Duration::from_secs(duration_sec)).await;
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
            timestamp: Local::now().timestamp(),
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
