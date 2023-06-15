mod consumer;
mod parse;

pub async fn prometheus_export_udco2s(port_name: String) {
    parse::monitor_co2ppm(&port_name, &consumer::UDCO2SPrometheusExporter) 
}


