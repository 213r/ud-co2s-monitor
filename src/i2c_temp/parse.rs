use super::consumer::SHT31Exporter;
use chrono::Local;
use rppal::i2c::I2c;
use serde_derive::Serialize;
use tokio::time::{sleep, Duration};

#[derive(Debug, PartialEq, Serialize)]
pub struct SHT31DATA {
    pub timestamp: String,
    pub hum: f64,
    pub temp: f64,
}

const ADDRESS_SHT31DIS: u16 = 0x45;

pub async fn monitor_sht31_temp(
    consumers: &mut [Box<dyn SHT31Exporter + Send>],
    duration_sec: u64,
) {
    let mut i2c = I2c::new().expect("Failed to get I2C bus");
    i2c.set_slave_address(ADDRESS_SHT31DIS)
        .expect("Failed to set slave address"); // SHT31のデフォルトアドレス
    i2c.smbus_write_byte(0x20, 0x32).unwrap();
    sleep(Duration::from_secs(1)).await;

    let mut buf: [u8; 6] = [0; 6];
    loop {
        match i2c.read(&mut buf) {
            Ok(_s) => {
                let data = parse_data(&buf);
                println!("{data:?}");
                for c in consumers.iter_mut() {
                    c.set(&data);
                }
            }
            Err(e) => {
                eprintln!("Parsing is failed. {:?}", e);
            }
        }
        sleep(Duration::from_secs(duration_sec)).await;
    }
}

fn parse_data(buf: &[u8]) -> SHT31DATA {
    let measured_temp = u16::from(buf[0]) << 8 | u16::from(buf[1]);
    let real_temp = f64::from(measured_temp) * 175. / (((1 << 16) - 1) as f64) - 45.;
    let measured_hum = u16::from(buf[3]) << 8 | u16::from(buf[4]);
    let real_hum = f64::from(measured_hum) * 100. / (((1 << 16) - 1) as f64);
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    SHT31DATA {
        timestamp: ts,
        hum: real_hum,
        temp: real_temp,
    }
}
