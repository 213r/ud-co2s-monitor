use super::parse::UDCO2SDATA;
use lazy_static::lazy_static;
use prometheus::{opts, register_gauge, Gauge};
use rumqtt::{MqttClient, QoS};

pub trait UDCO2SExporter {
    fn set(&mut self, data: &UDCO2SDATA);
}

lazy_static! {
    static ref CO2PPM_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_co2ppm", "CO2 ppm value",)).unwrap();
    static ref HUM_GAUGE: Gauge = register_gauge!(opts!("udco2s_hum", "Humidity value",)).unwrap();
    static ref TEMP_GAUGE: Gauge =
        register_gauge!(opts!("udco2s_temp", "Temperature value",)).unwrap();
}

pub struct UDCO2SPrometheusExporter;

impl UDCO2SExporter for UDCO2SPrometheusExporter {
    fn set(&mut self, data: &UDCO2SDATA) {
        CO2PPM_GAUGE.set(data.co2);
        HUM_GAUGE.set(data.hum);
        TEMP_GAUGE.set(data.temp);
    }
}

pub struct UDCO2SAWSIOTExporter {
    mqtt_client: MqttClient,
    topic: String,
}

impl UDCO2SAWSIOTExporter {
    pub fn new(mqtt_client: MqttClient, topic: String) -> Self {
        Self { mqtt_client, topic }
    }
}

impl UDCO2SExporter for UDCO2SAWSIOTExporter {
    fn set(&mut self, data: &UDCO2SDATA) {
        let payload_str = serde_json::to_string(data).unwrap();
        self.mqtt_client
            .publish(&self.topic, QoS::AtLeastOnce, false, payload_str)
            .unwrap();
    }
}
