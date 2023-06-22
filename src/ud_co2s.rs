use rumqtt::MqttClient;

mod consumer;
mod parse;

pub async fn prometheus_export_udco2s(port_name: String, duration_sec: u64) {
    parse::monitor_co2ppm(
        &port_name,
        &mut consumer::UDCO2SPrometheusExporter,
        duration_sec,
    )
    .await;
}

pub async fn awsiot_export_udco2s(
    port_name: String,
    duration_sec: u64,
    mqtt_client: MqttClient,
    topic: String,
) {
    let mut exporter = consumer::UDCO2SAWSIOTExporter::new(mqtt_client, topic);
    parse::monitor_co2ppm(&port_name, &mut exporter, duration_sec).await;
}
