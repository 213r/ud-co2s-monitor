use lazy_static::lazy_static;
use prometheus::{opts, register_gauge, Gauge};
use super::parse::UDCO2SDATA;

pub trait UDCO2SExporter {
    fn set(&self, data: &UDCO2SDATA); 
}

lazy_static! {
    static ref CO2PPM_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_co2ppm", "CO2 ppm value",)).unwrap();
    static ref HUM_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_hum", "Humidity value",)).unwrap();
    static ref TEMP_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_temp", "Temperature value",)).unwrap();
}

pub struct UDCO2SPrometheusExporter; 

impl UDCO2SExporter for UDCO2SPrometheusExporter{
    fn set(&self, data: &UDCO2SDATA) {
        CO2PPM_GAUGE.set(data.co2);
        HUM_GAUGE.set(data.hum);
        TEMP_GAUGE.set(data.temp);
    }
}
