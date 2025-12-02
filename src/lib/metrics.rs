use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{MetricExporter, Protocol};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub fn init_metrics(resource: Resource, endpoint: &str) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) //can be changed to `Protocol::HttpJson` to export in JSON format
        .with_endpoint(format!("http://{}/v1/metrics", endpoint))
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build()
}
