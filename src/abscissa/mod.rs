//! Abscissa framework integration
//!
//! This module provides an Abscissa component for easy integration of
//! observlib telemetry into Abscissa applications.
//!
//! # Features
//!
//! Enable the `abscissa` feature to use this module:
//!
//! ```toml
//! [dependencies]
//! observlib = { version = "0.1", features = ["abscissa"] }
//! ```

mod component;
mod config;

pub use component::{HasObservabilityConfig, ObservabilityComponent};
pub use config::{FilterConfig, ObservabilityConfig, ProtocolConfig};
