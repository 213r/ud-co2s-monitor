use super::parse::SHT31DATA;
use lazy_static::lazy_static;
use prometheus::{opts, register_gauge, Gauge};
use rumqtt::{MqttClient, QoS};

pub trait SHT31Exporter {
    fn set(&mut self, data: &SHT31DATA);
}

lazy_static! {
    static ref HUM_GAUGE: Gauge = register_gauge!(opts!("sht31_hum", "Humidity value",)).unwrap();
    static ref TEMP_GAUGE: Gauge =
        register_gauge!(opts!("sht31_temp", "Temperature value",)).unwrap();
}

pub struct SHT31PrometheusExporter;

impl SHT31Exporter for SHT31PrometheusExporter {
    fn set(&mut self, data: &SHT31DATA) {
        HUM_GAUGE.set(data.hum);
        TEMP_GAUGE.set(data.temp);
    }
}

pub struct SHT31AWSIOTExporter {
    mqtt_client: MqttClient,
    topic: String,
}

impl SHT31AWSIOTExporter {
    pub fn new(mqtt_client: MqttClient, topic: String) -> Self {
        Self { mqtt_client, topic }
    }
}

impl SHT31Exporter for SHT31AWSIOTExporter {
    fn set(&mut self, data: &SHT31DATA) {
        let payload_str = serde_json::to_string(data).unwrap();
        self.mqtt_client
            .publish(&self.topic, QoS::AtLeastOnce, false, payload_str)
            .unwrap();
    }
}
