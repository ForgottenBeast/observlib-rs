# observlib-rs

A Rust library providing unified OpenTelemetry initialization for traces, metrics, and logs with OTLP export.

## Features

- **Unified Initialization**: Single function call to set up traces, metrics, and logs
- **OTLP Export**: Uses OpenTelemetry Protocol over HTTP with configurable binary or JSON format
- **Proper Lifecycle Management**: Clean shutdown of all telemetry providers
- **Telemetry Loop Prevention**: Built-in filtering to prevent infinite telemetry loops
- **Abscissa Integration**: Optional component for easy integration with Abscissa applications

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
observlib = "0.1"
```

For Abscissa integration:

```toml
[dependencies]
observlib = { version = "0.1", features = ["abscissa"] }
```

## Quick Start

### Standalone Usage

```rust
use observlib::{initialize_telemetry, KeyValue};

fn main() {
    let attrs = vec![
        KeyValue::new("env", "production"),
        KeyValue::new("version", "1.0.0"),
    ];

    let otel_manager = initialize_telemetry(
        "my-service",
        "localhost:4318",
        attrs
    );

    // Your application code here
    tracing::info!("Application started!");

    // Clean shutdown
    otel_manager.shutdown().unwrap();
}
```

### Abscissa Integration

See [Abscissa Usage Guide](docs/abscissa-usage-example.md) for complete integration instructions.

Quick example:

```rust
use abscissa_core::{Application, Component};
use observlib::abscissa::{ObservabilityComponent, ObservabilityConfig, HasObservabilityConfig};

// In your config
#[derive(Config)]
pub struct MyAppConfig {
    pub observability: ObservabilityConfig,
}

impl HasObservabilityConfig for MyAppConfig {
    fn observability_config(&self) -> &ObservabilityConfig {
        &self.observability
    }
}

// Register component
fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
    let mut components = self.framework_components(command)?;
    components.register(ObservabilityComponent::new())?;
    self.state.components = components;
    Ok(())
}
```

Configuration file (`config.toml`):

```toml
[observability]
service_name = "my-service"
endpoint = "localhost:4318"

[observability.attributes]
environment = "production"
```

## Architecture

The library initializes OpenTelemetry in the following order:

1. **Logs First**: Logger provider is initialized first to capture initialization logs from other providers
2. **Tracing Subscriber**: Sets up two layers:
   - OpenTelemetry layer for OTLP export with HTTP client log suppression
   - Console layer for local output
3. **Traces**: Initializes tracer provider and sets global provider
4. **Metrics**: Initializes meter provider and sets global provider

### Telemetry Loop Prevention

The library includes critical filtering to prevent telemetry-induced telemetry loops. HTTP client logs from `hyper`, `tonic`, `h2`, and `reqwest` are suppressed because these components don't propagate OpenTelemetry context.

**Important**: This filtering affects ALL logs from these crates, not just OTLP-related ones.

## Configuration

### OTLP Endpoints

All exporters use HTTP endpoints in the format:
- Logs: `http://{endpoint}/v1/logs`
- Traces: `http://{endpoint}/v1/traces`
- Metrics: `http://{endpoint}/v1/metrics`

### Protocol

Default: HTTP Binary Protocol

To use JSON format, modify the exporter builders in `src/lib/{logs,traces,metrics}.rs`:

```rust
.with_protocol(Protocol::HttpJson)
```

## Development

### Building

```bash
# Standard build
cargo build

# With Abscissa feature
cargo build --features abscissa

# Release build
cargo build --release
```

### Testing

```bash
cargo test
```

### Using Nix

This project includes a Nix flake for reproducible builds:

```bash
# Enter development shell
nix develop

# Build
nix build
```

## Documentation

- [Abscissa Component Research](docs/abscissa-component-research.md)
- [Abscissa Configuration Design](docs/abscissa-config-design.md)
- [Abscissa Usage Example](docs/abscissa-usage-example.md)

## Dependencies

- `opentelemetry` 0.31.0 - Core OTEL API
- `opentelemetry-otlp` 0.31.0 - OTLP exporters with HTTP transport
- `opentelemetry-sdk` 0.31.0 - SDK implementations for traces, metrics, and logs
- `tracing-subscriber` 0.3.22 - Tracing layer composition
- `tracing-opentelemetry` 0.32.0 - Bridge between tracing and OpenTelemetry

Optional:
- `abscissa_core` 0.9 - Abscissa framework integration (when `abscissa` feature is enabled)
- `serde` 1.0 - Configuration serialization (when `abscissa` feature is enabled)

## License

This project follows standard Rust licensing practices.

## Contributing

Contributions are welcome! This project uses [beads](https://github.com/peterprototypes/beads) for issue tracking:

```bash
# List open tasks
br list --status open

# View task details
br show <task-id>

# Find ready tasks
br ready
```

## Roadmap

See the [project epic](https://github.com/yourusername/observlib-rs/issues) for the Abscissa integration roadmap.
