use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{LogExporter, Protocol};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;

pub fn init_logs(resource: Resource, endpoint: &str) -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(format!("http://{}/v1/logs", endpoint))
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build()
}
