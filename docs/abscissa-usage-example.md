# Abscissa Integration Usage Example

This document shows how to integrate the observlib observability component into an Abscissa application.

## 1. Add Dependencies

```toml
[dependencies]
observlib = { version = "0.1", features = ["abscissa"] }
abscissa_core = "0.9"
```

## 2. Configure Your Application Config

```rust
use abscissa_core::Config;
use observlib::abscissa::{HasObservabilityConfig, ObservabilityConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Config, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MyAppConfig {
    /// Observability configuration
    pub observability: ObservabilityConfig,

    // ... other config fields
}

// Implement the trait to provide access to observability config
impl HasObservabilityConfig for MyAppConfig {
    fn observability_config(&self) -> &ObservabilityConfig {
        &self.observability
    }
}
```

## 3. Register the Component

In your application's `register_components` method:

```rust
use abscissa_core::{Application, Component, FrameworkError};
use observlib::abscissa::ObservabilityComponent;

impl Application for MyApp {
    // ... other methods ...

    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let mut components = self.framework_components(command)?;

        // Register the observability component
        components.register(ObservabilityComponent::new())?;

        self.state.components = components;
        Ok(())
    }
}
```

## 4. Create Configuration File

Create a TOML configuration file (e.g., `config.toml`):

```toml
[observability]
enabled = true
service_name = "my-awesome-service"
endpoint = "localhost:4318"

[observability.attributes]
environment = "production"
version = "1.0.0"
region = "us-west-2"

[observability.protocol]
format = "http-binary"

[observability.filters]
otel = "info,hyper=off,tonic=off,h2=off,reqwest=off"
console = "info,opentelemetry=debug"
```

## 5. Use OpenTelemetry in Your Code

Once the component is registered, you can use OpenTelemetry throughout your application:

```rust
use opentelemetry::global;
use tracing::{info, warn, error};

fn my_function() {
    // Logs are automatically captured and exported
    info!("Processing request");

    // Traces
    let tracer = global::tracer("my-component");
    let span = tracer.start("my-operation");
    // ... do work ...
    drop(span);

    // Metrics
    let meter = global::meter("my-component");
    let counter = meter.u64_counter("requests_total").init();
    counter.add(1, &[]);
}
```

## Configuration Options

### Required Fields

- `service_name`: Identifies your service in telemetry data

### Optional Fields

- `enabled` (default: `true`): Enable/disable observability
- `endpoint` (default: `"localhost:4318"`): OTLP collector endpoint
- `attributes`: Custom key-value pairs attached to all telemetry
- `protocol.format`: `"http-binary"` (default) or `"http-json"`
- `filters.otel`: EnvFilter for OTLP layer (default suppresses HTTP client logs)
- `filters.console`: EnvFilter for console output

### Environment Variable Overrides

The component supports environment variable overrides:

```bash
export OTEL_SERVICE_NAME="my-service-override"
export OTEL_EXPORTER_OTLP_ENDPOINT="prod-collector:4318"
```

Call `config.observability.apply_env_overrides()` before initialization to apply these.

## Component Lifecycle

1. **`after_config`**: Called when configuration is loaded
   - Validates configuration
   - Initializes telemetry providers if enabled
   - Sets up tracing subscriber with configured filters

2. **`before_shutdown`**: Called before application shutdown
   - Flushes pending telemetry data
   - Shuts down all providers (logger, tracer, meter)
   - Ensures clean exit

## Complete Example Application

```rust
use abscissa_core::{
    Application, Command, Component, Config,
    FrameworkError, StandardPaths, Runnable
};
use observlib::abscissa::{HasObservabilityConfig, ObservabilityComponent, ObservabilityConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Config, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub observability: ObservabilityConfig,
}

impl HasObservabilityConfig for AppConfig {
    fn observability_config(&self) -> &ObservabilityConfig {
        &self.observability
    }
}

#[derive(Command, Debug, Default)]
pub struct StartCommand;

impl Runnable for StartCommand {
    fn run(&self) {
        tracing::info!("Application started!");
        // Your app logic here
    }
}

pub struct MyApp {
    config: Option<AppConfig>,
    state: application::State<Self>,
}

impl Application for MyApp {
    type Cmd = StartCommand;
    type Cfg = AppConfig;
    type Paths = StandardPaths;

    fn config(&self) -> &AppConfig {
        self.config.as_ref().expect("config not loaded")
    }

    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let mut components = self.framework_components(command)?;
        components.register(ObservabilityComponent::new())?;
        self.state.components = components;
        Ok(())
    }

    fn after_config(&mut self, config: Self::Cfg) -> Result<(), FrameworkError> {
        self.config = Some(config);
        Ok(())
    }
}
```

## Testing

To test observability in development:

1. Run an OTLP collector locally:
   ```bash
   docker run -p 4318:4318 otel/opentelemetry-collector
   ```

2. Configure your app to point to localhost:
   ```toml
   [observability]
   service_name = "my-app-dev"
   endpoint = "localhost:4318"
   ```

3. Run your application and observe telemetry data in the collector.
