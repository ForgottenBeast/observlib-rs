# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`observlib-rs` is a Rust library that provides a unified OpenTelemetry initialization interface for traces, metrics, and logs. It handles the complete setup of OpenTelemetry exporters using OTLP over HTTP with proper resource management and lifecycle handling.

## Building and Testing

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Run the example application
cargo run

# Check for compilation errors without building
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Nix Development Environment

This project uses Nix flakes for reproducible development:

```bash
# Enter development shell with all dependencies
nix develop

# Build the package
nix build
```

The dev shell automatically:
- Sets `CARGO_HOME` to `.cargo` in the project directory
- Provides complete Rust toolchain (cargo, clippy, rust-src, rustc, rustfmt)

## Task Tracking

This project uses **beads** for git-backed issue tracking. Use the `br` command to interact with tasks:

```bash
# List all open tasks
br list --status open

# List tasks by priority (0=critical, 1=high, 2=medium, 3=low, 4=backlog)
br list --priority 1

# Show details of a specific task
br show <issue-id>

# Create a new task
br create

# Update task status
br update <issue-id> --status in_progress

# List ready-to-work tasks (no blockers)
br ready
```

All tasks are stored in `.beads/` and tracked in git, surviving conversation compaction and providing persistent context across sessions.

## Architecture

### Core Components

The library consists of three main telemetry signal types, each initialized independently:

1. **Logs** (`src/lib/logs.rs`) - Configures `SdkLoggerProvider` with OTLP HTTP binary exporter
2. **Traces** (`src/lib/traces.rs`) - Configures `SdkTracerProvider` with batch span exporter
3. **Metrics** (`src/lib/metrics.rs`) - Configures `SdkMeterProvider` with periodic metric exporter

### Initialization Flow

The `initialize_telemetry()` function in `src/lib/mod.rs` orchestrates:

1. **Resource Creation** - Shared resource with service name and custom attributes (cached in `OnceLock`)
2. **Logger Initialization** - Sets up logs first to capture initialization logs from other providers
3. **Tracing Subscriber Setup** - Configures two layers:
   - `OpenTelemetryTracingBridge` - Bridges tracing logs to OTLP
   - `fmt::layer` - Console output with thread names
4. **Tracer Provider** - Initializes traces and sets global tracer provider
5. **Meter Provider** - Initializes metrics and sets global meter provider

### Telemetry Loop Prevention

The library includes critical filtering to prevent telemetry-induced telemetry loops. Internal HTTP client logs from `hyper`, `tonic`, `h2`, and `reqwest` are completely suppressed via `EnvFilter` directives. This filtering is necessary because these components don't propagate OpenTelemetry context (see upstream issue #2877).

**Important**: This filtering affects ALL logs from these crates, not just OTLP-related ones.

### Lifecycle Management

The `OtelManager` struct holds all three providers and implements graceful shutdown:
- Call `shutdown()` to flush pending data and release resources
- Collects errors from all providers and returns a combined error if any fail
- Should be called at application termination

### OTLP Configuration

All exporters use:
- **Protocol**: HTTP Binary (can be changed to `Protocol::HttpJson`)
- **Endpoints**: `http://{endpoint}/v1/{logs|traces|metrics}`
- **Transport**: Reqwest blocking HTTP client

## Key Dependencies

- `opentelemetry` 0.31.0 - Core OTEL API
- `opentelemetry-otlp` 0.31.0 - OTLP exporters
- `opentelemetry-sdk` 0.31.0 - SDK implementations
- `tracing-subscriber` 0.3.22 - Tracing layer composition
- `tracing-opentelemetry` 0.32.0 - Bridge between tracing and OTEL

## Usage Pattern

```rust
use observlib::{initialize_telemetry, KeyValue};

let attrs = vec![KeyValue::new("env", "production")];
let otel_manager = initialize_telemetry("my-service", "localhost:4318", attrs);

// ... application code ...

otel_manager.shutdown().unwrap();
```

The library automatically sets global tracer and meter providers, allowing application code to use `global::tracer()` and `global::meter()` anywhere.
