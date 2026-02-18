# Abscissa Component Configuration Design

## Configuration Structure

The observability component will integrate with Abscissa's TOML-based configuration system using serde for deserialization.

## Core Configuration Struct

```rust
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
```

## TOML Configuration Example

### Minimal Configuration

```toml
[observability]
service_name = "my-application"
```

### Full Configuration

```toml
[observability]
enabled = true
service_name = "my-application"
endpoint = "otel-collector.prod.example.com:4318"

[observability.attributes]
environment = "production"
region = "us-west-2"
version = "1.2.3"

[observability.protocol]
format = "http-binary"  # or "http-json"

[observability.filters]
otel = "info,hyper=off,tonic=off,h2=off,reqwest=off"
console = "info,opentelemetry=debug"
```

### Development Configuration Example

```toml
[observability]
enabled = true
service_name = "my-app-dev"
endpoint = "localhost:4318"

[observability.attributes]
environment = "development"
developer = "alice"

[observability.filters]
# More verbose logging in development
console = "debug"
```

## Integration with Abscissa Application Config

The application's main config struct should include the observability config:

```rust
use abscissa_core::Config;
use serde::{Deserialize, Serialize};

#[derive(Clone, Config, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MyAppConfig {
    /// Observability configuration
    pub observability: ObservabilityConfig,

    // ... other app config fields
}
```

## Configuration Validation

The component should validate configuration in `after_config`:

```rust
impl<A: Application> Component<A> for ObservabilityComponent {
    fn after_config(&mut self, config: &A::Cfg) -> Result<(), FrameworkError> {
        let obs_config = // extract ObservabilityConfig from app config

        // Validate service name is not empty
        if obs_config.service_name.is_empty() {
            return Err(FrameworkError::config_error(
                "observability.service_name cannot be empty"
            ));
        }

        // Validate endpoint format
        if !obs_config.endpoint.contains(':') {
            return Err(FrameworkError::config_error(
                "observability.endpoint must be in format 'host:port'"
            ));
        }

        // Initialize observability if enabled
        if obs_config.enabled {
            self.initialize(obs_config)?;
        }

        Ok(())
    }
}
```

## Design Rationale

1. **`enabled` flag**: Allows disabling observability without removing config
2. **Required `service_name`**: Essential for identifying telemetry sources
3. **Default endpoint**: Localhost for easy local development
4. **HashMap for attributes**: Flexible key-value pairs for custom metadata
5. **Protocol configuration**: Future-proof for different OTLP formats
6. **Filter configuration**: Prevents telemetry loops while maintaining control
7. **Serde defaults**: Reduces boilerplate in config files
8. **deny_unknown_fields**: Catches typos and invalid config early

## Environment Variable Override

Optionally support environment variables for containerized deployments:

```rust
impl ObservabilityConfig {
    /// Override config with environment variables
    pub fn apply_env_overrides(&mut self) {
        if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            self.endpoint = endpoint;
        }
        if let Ok(service_name) = std::env::var("OTEL_SERVICE_NAME") {
            self.service_name = service_name;
        }
    }
}
```

This allows runtime configuration in Kubernetes/Docker without config file changes.
