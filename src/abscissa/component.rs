//! Abscissa component implementation for observability

use super::config::ObservabilityConfig;
use crate::{KeyValue, OtelManager};
use abscissa_core::{component::Id, Component, FrameworkError, FrameworkErrorKind, Injectable, Shutdown, Version};
use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::Resource;
use std::sync::{Arc, Mutex};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// Abscissa component for OpenTelemetry observability
///
/// This component integrates observlib with Abscissa applications,
/// handling initialization and shutdown of OpenTelemetry telemetry.
///
/// # Example
///
/// ```rust,ignore
/// use abscissa_core::{Application, Component};
/// use observlib::abscissa::ObservabilityComponent;
///
/// // In your application's component registration:
/// let observability = ObservabilityComponent::new();
/// ```
#[derive(Debug)]
pub struct ObservabilityComponent {
    /// The OtelManager, wrapped in Arc<Mutex<>> for thread-safe access
    /// Option because it's only initialized if observability is enabled
    manager: Arc<Mutex<Option<OtelManager>>>,
}

impl ObservabilityComponent {
    /// Create a new observability component
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize telemetry with the given configuration
    fn initialize(&self, config: &ObservabilityConfig) -> Result<(), FrameworkError> {
        // Convert HashMap<String, String> to Vec<KeyValue>
        let attributes: Vec<KeyValue> = config
            .attributes
            .iter()
            .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            .collect();

        // Build resource with service name and attributes
        let resource = Resource::builder()
            .with_service_name(config.service_name.clone())
            .with_attributes(attributes)
            .build();

        // Initialize logs
        let logger_provider = crate::logs::init_logs(resource.clone(), &config.endpoint);
        let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        // Parse and apply filters
        let filter_otel = EnvFilter::try_new(&config.filters.otel).map_err(|e| {
            let ctx = FrameworkErrorKind::ConfigError.context(format!("invalid otel filter: {}", e));
            FrameworkError::from(ctx)
        })?;
        let otel_layer = otel_layer.with_filter(filter_otel);

        let filter_fmt = EnvFilter::try_new(&config.filters.console).map_err(|e| {
            let ctx = FrameworkErrorKind::ConfigError.context(format!("invalid console filter: {}", e));
            FrameworkError::from(ctx)
        })?;
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_filter(filter_fmt);

        // Initialize the tracing subscriber
        tracing_subscriber::registry()
            .with(otel_layer)
            .with(fmt_layer)
            .init();

        // Initialize traces
        let tracer_provider = crate::traces::init_traces(resource.clone(), &config.endpoint);
        global::set_tracer_provider(tracer_provider.clone());

        // Initialize metrics
        let meter_provider = crate::metrics::init_metrics(resource.clone(), &config.endpoint);
        global::set_meter_provider(meter_provider.clone());

        // Store the manager
        let manager = OtelManager {
            logger: logger_provider,
            tracer: tracer_provider,
            meter: meter_provider,
        };
        *self.manager.lock().unwrap() = Some(manager);

        Ok(())
    }
}

impl Default for ObservabilityComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: abscissa_core::Application> Injectable<A> for ObservabilityComponent {
    fn id(&self) -> Id {
        Id::new("observlib::ObservabilityComponent")
    }

    fn version(&self) -> Version {
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
    }
}

/// Trait that application configs must implement to provide observability configuration
pub trait HasObservabilityConfig {
    /// Get a reference to the observability configuration
    fn observability_config(&self) -> &ObservabilityConfig;
}

impl<A> Component<A> for ObservabilityComponent
where
    A: abscissa_core::Application,
    A::Cfg: HasObservabilityConfig,
{
    fn after_config(&mut self, config: &A::Cfg) -> Result<(), FrameworkError> {
        let obs_config = config.observability_config();

        // Validate configuration
        obs_config.validate().map_err(|e| {
            let ctx = FrameworkErrorKind::ConfigError.context(format!("observability: {}", e));
            FrameworkError::from(ctx)
        })?;

        // Only initialize if enabled
        if obs_config.enabled {
            self.initialize(obs_config)?;
        }

        Ok(())
    }

    fn before_shutdown(&self, _kind: Shutdown) -> Result<(), FrameworkError> {
        // Shutdown the telemetry manager if it was initialized
        if let Some(manager) = self.manager.lock().unwrap().as_ref() {
            manager.shutdown().map_err(|e| {
                let ctx = FrameworkErrorKind::ComponentError.context(format!("observability shutdown: {}", e));
                FrameworkError::from(ctx)
            })?;
        }

        Ok(())
    }
}
