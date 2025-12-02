use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use std::error::Error;
use std::sync::OnceLock;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

mod logs;
mod metrics;
mod traces;

pub use opentelemetry_api::KeyValue;

pub struct OtelManager {
    logger: SdkLoggerProvider,
    meter: SdkMeterProvider,
    tracer: SdkTracerProvider,
}

impl OtelManager {
    pub fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut shutdown_errors = Vec::new();
        if let Err(e) = self.tracer.shutdown() {
            shutdown_errors.push(format!("tracer provider: {e}"));
        }

        if let Err(e) = self.meter.shutdown() {
            shutdown_errors.push(format!("meter provider: {e}"));
        }

        if let Err(e) = self.logger.shutdown() {
            shutdown_errors.push(format!("logger provider: {e}"));
        }
        if !shutdown_errors.is_empty() {
            return Err(format!(
                "Failed to shutdown providers:{}",
                shutdown_errors.join("\n")
            )
            .into());
        }
        Ok(())
    }
}

fn get_resource<T: IntoIterator<Item = KeyValue>>(
    service_name: &'static str,
    attrs: T,
) -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name(service_name)
                .with_attributes(attrs)
                .build()
        })
        .clone()
}

pub fn initialize_telemetry<T: IntoIterator<Item = KeyValue>>(
    service_name: &'static str,
    endpoint: &str,
    attributes: T,
) -> OtelManager {
    let resource = get_resource(service_name, attributes);
    let logger_provider = logs::init_logs(resource.clone(), endpoint);
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // To prevent a telemetry-induced-telemetry loop, OpenTelemetry's own internal
    // logging is properly suppressed. However, logs emitted by external components
    // (such as reqwest, tonic, etc.) are not suppressed as they do not propagate
    // OpenTelemetry context. Until this issue is addressed
    // (https://github.com/open-telemetry/opentelemetry-rust/issues/2877),
    // filtering like this is the best way to suppress such logs.
    //
    // The filter levels are set as follows:
    // - Allow `info` level and above by default.
    // - Completely restrict logs from `hyper`, `tonic`, `h2`, and `reqwest`.
    //
    // Note: This filtering will also drop logs from these components even when
    // they are used outside of the OTLP Exporter.
    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = otel_layer.with_filter(filter_otel);

    // Create a new tracing::Fmt layer to print the logs to stdout. It has a
    // default filter of `info` level and above, and `debug` and above for logs
    // from OpenTelemetry crates. The filter levels can be customized as needed.
    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    // Initialize the tracing subscriber with the OpenTelemetry layer and the
    // Fmt layer.
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    // At this point Logs (OTel Logs and Fmt Logs) are initialized, which will
    // allow internal-logs from Tracing/Metrics initializer to be captured.

    let tracer_provider = traces::init_traces(resource.clone(), endpoint);
    // Set the global tracer provider using a clone of the tracer_provider.
    // Setting global tracer provider is required if other parts of the application
    // uses global::tracer() or global::tracer_with_version() to get a tracer.
    // Cloning simply creates a new reference to the same tracer provider. It is
    // important to hold on to the tracer_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_tracer_provider(tracer_provider.clone());

    let meter_provider = metrics::init_metrics(resource.clone(), endpoint);
    // Set the global meter provider using a clone of the meter_provider.
    // Setting global meter provider is required if other parts of the application
    // uses global::meter() or global::meter_with_version() to get a meter.
    // Cloning simply creates a new reference to the same meter provider. It is
    // important to hold on to the meter_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_meter_provider(meter_provider.clone());
    OtelManager {
        logger: logger_provider,
        tracer: tracer_provider,
        meter: meter_provider,
    }
}
