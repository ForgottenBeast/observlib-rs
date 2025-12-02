use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{Protocol, SpanExporter};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;

pub fn init_traces(resource: Resource, endpoint: &str) -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) //can be changed to `Protocol::HttpJson` to export in JSON format
        .with_endpoint(format!("http://{}/v1/traces", endpoint))
        .build()
        .expect("Failed to create trace exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build()
}
