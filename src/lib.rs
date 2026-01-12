/*!
This crate provides a simple, easy to setup opentelemetry configuration and reexports the KeyValue and global object
for ease if use.

The Otelmanager object is here to allow graceful shutdown
*/
pub use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use std::sync::OnceLock;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

mod errors;
mod logs;
mod metrics;
mod traces;

pub use errors::ObservlibError;

///Singleton object to have one place to call shutdown on the complete telemetry apparatus
pub struct OtelManager {
    logger: SdkLoggerProvider,
    meter: SdkMeterProvider,
    tracer: SdkTracerProvider,
}

impl OtelManager {
    ///Blocking function to shutdown telemetry gracefully
    pub fn shutdown(&self) -> Result<(), ObservlibError> {
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
            return Err(ObservlibError::MultipleShutdownFailures(
                shutdown_errors.join("\n")
            ));
        }
        Ok(())
    }

    ///Async function to shutdown telemetry gracefully with timeout support
    ///
    /// This is useful when shutting down in async contexts (e.g., tokio runtime)
    /// or when you need to enforce a timeout to prevent hanging on shutdown.
    ///
    /// # Arguments
    /// * `timeout` - Maximum duration to wait for shutdown. If None, waits indefinitely.
    ///
    /// # Example
    /// ```no_run
    /// use std::time::Duration;
    /// # use observlib::{KeyValue, initialize_telemetry};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let otel = initialize_telemetry("service", "127.0.0.1:4318", vec![]);
    ///
    /// // Shutdown with 5 second timeout
    /// otel.async_shutdown(Some(Duration::from_secs(5))).await.unwrap();
    /// # }
    /// ```
    #[cfg(feature = "async")]
    pub async fn async_shutdown(
        &self,
        timeout: Option<std::time::Duration>,
    ) -> Result<(), ObservlibError> {
        let shutdown_future = async {
            tokio::task::spawn_blocking({
                let tracer = self.tracer.clone();
                let meter = self.meter.clone();
                let logger = self.logger.clone();
                move || {
                    let mut shutdown_errors = Vec::new();
                    if let Err(e) = tracer.shutdown() {
                        shutdown_errors.push(format!("tracer provider: {e}"));
                    }

                    if let Err(e) = meter.shutdown() {
                        shutdown_errors.push(format!("meter provider: {e}"));
                    }

                    if let Err(e) = logger.shutdown() {
                        shutdown_errors.push(format!("logger provider: {e}"));
                    }
                    if !shutdown_errors.is_empty() {
                        return Err(ObservlibError::MultipleShutdownFailures(
                            shutdown_errors.join("\n")
                        ));
                    }
                    Ok(())
                }
            })
            .await?
        };

        match timeout {
            Some(duration) => {
                tokio::time::timeout(duration, shutdown_future)
                    .await
                    .map_err(|_| ObservlibError::ShutdownTimeout)?
            }
            None => shutdown_future.await,
        }
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

///library entrypoint
///service name used for initialization
///otlp http endpoint (example: 127.0.0.1:4318)
///Resource attributes that will be added to all providers
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
