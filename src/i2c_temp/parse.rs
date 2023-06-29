use super::consumer::UDCO2SExporter;
use chrono::Local;
use regex::Regex;
use serde_derive::Serialize;
use std::io;
use tokio::time::{sleep, Duration};
use rppal::i2c::I2c;

#[derive(Debug, PartialEq, Serialize)]
pub struct UDCO2SDATA {
    pub timestamp: String,
    pub co2: f64,
    pub hum: f64,
    pub temp: f64,
}

const BUS: u8 = 1;
const ADDRESS_SHT31DIS : u16 = 0x48;

pub async fn monitor_temp(
    port_name: &str,
    consumers: &mut [Box<dyn UDCO2SExporter + Send>],
    duration_sec: u64,
) {

    let mut i2c = I2c::with_bus(BUS).expect("Couldn't start i2c. Is the interface enabled?");
    i2c.set_slave_address(ADDRESS_SHT31DIS).unwrap();

    let mut buf: Vec<u8> = vec![0; 100];
    loop {
        let temp = read_temperature(&i2c);
        sleep(Duration::from_secs(duration_sec)).await;
    }
}

fn read