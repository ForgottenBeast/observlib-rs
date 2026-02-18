//! Configuration structures for the observability component

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the OpenTelemetry observability component
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityConfig {
    /// Whether observability is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Service name for telemetry identification
    pub service_name: String,

    /// OTLP endpoint (host:port)
    /// Example: "localhost:4318" or "otel-collector.example.com:4318"
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Custom resource attributes
    /// These will be attached to all telemetry signals
    #[serde(default)]
    pub attributes: HashMap<String, String>,

    /// Optional protocol configuration
    #[serde(default)]
    pub protocol: ProtocolConfig,

    /// Optional filter configuration for logs
    #[serde(default)]
    pub filters: FilterConfig,
}

/// Protocol configuration for OTLP exporters
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolConfig {
    /// Protocol format: "http-binary" or "http-json"
    #[serde(default = "default_protocol")]
    pub format: String,
}

/// Filter configuration for controlling log output
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterConfig {
    /// Environment filter for OpenTelemetry layer
    /// Example: "info,hyper=off,tonic=off"
    #[serde(default = "default_otel_filter")]
    pub otel: String,

    /// Environment filter for console output
    /// Example: "info,opentelemetry=debug"
    #[serde(default = "default_console_filter")]
    pub console: String,
}

// Default values
fn default_enabled() -> bool {
    true
}

fn default_endpoint() -> String {
    "localhost:4318".to_string()
}

fn default_protocol() -> String {
    "http-binary".to_string()
}

fn default_otel_filter() -> String {
    "info,hyper=off,tonic=off,h2=off,reqwest=off".to_string()
}

fn default_console_filter() -> String {
    "info,opentelemetry=debug".to_string()
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            format: default_protocol(),
        }
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            otel: default_otel_filter(),
            console: default_console_filter(),
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "default-service".to_string(),
            endpoint: default_endpoint(),
            attributes: HashMap::new(),
            protocol: ProtocolConfig::default(),
            filters: FilterConfig::default(),
        }
    }
}

impl ObservabilityConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if self.service_name.is_empty() {
                return Err("service_name cannot be empty".to_string());
            }

            if !self.endpoint.contains(':') {
                return Err("endpoint must be in format 'host:port'".to_string());
            }
        }

        Ok(())
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            self.endpoint = endpoint;
        }
        if let Ok(service_name) = std::env::var("OTEL_SERVICE_NAME") {
            self.service_name = service_name;
        }
    }
}
