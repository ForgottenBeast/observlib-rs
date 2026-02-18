#![cfg(feature = "abscissa")]

use observlib::abscissa::{ObservabilityComponent, ObservabilityConfig};
use std::collections::HashMap;

#[test]
fn test_component_creation() {
    let component = ObservabilityComponent::new();
    assert!(format!("{:?}", component).contains("ObservabilityComponent"));
}

#[test]
fn test_component_default() {
    let component = ObservabilityComponent::default();
    assert!(format!("{:?}", component).contains("ObservabilityComponent"));
}

#[test]
fn test_config_validation_empty_service_name() {
    let mut config = ObservabilityConfig::default();
    config.service_name = String::new();
    config.enabled = true;

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_name"));
}

#[test]
fn test_config_validation_invalid_endpoint() {
    let mut config = ObservabilityConfig::default();
    config.service_name = "test-service".to_string();
    config.endpoint = "invalid-endpoint".to_string(); // missing port
    config.enabled = true;

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("host:port"));
}

#[test]
fn test_config_validation_disabled_skips_validation() {
    let mut config = ObservabilityConfig::default();
    config.service_name = String::new(); // Would fail if enabled
    config.enabled = false;

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_config_validation_success() {
    let mut config = ObservabilityConfig::default();
    config.service_name = "test-service".to_string();
    config.endpoint = "localhost:4318".to_string();
    config.enabled = true;

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_config_defaults() {
    let config = ObservabilityConfig::default();

    assert!(config.enabled);
    assert_eq!(config.service_name, "default-service");
    assert_eq!(config.endpoint, "localhost:4318");
    assert_eq!(config.protocol.format, "http-binary");
    assert!(config.filters.otel.contains("hyper=off"));
    assert!(config.filters.console.contains("info"));
}

#[test]
fn test_config_with_attributes() {
    let mut config = ObservabilityConfig::default();
    config.attributes.insert("env".to_string(), "test".to_string());
    config.attributes.insert("version".to_string(), "1.0.0".to_string());

    assert_eq!(config.attributes.len(), 2);
    assert_eq!(config.attributes.get("env").unwrap(), "test");
}

#[test]
fn test_filter_config_defaults() {
    let filter = observlib::abscissa::FilterConfig::default();

    // Should suppress HTTP client logs
    assert!(filter.otel.contains("hyper=off"));
    assert!(filter.otel.contains("tonic=off"));
    assert!(filter.otel.contains("h2=off"));
    assert!(filter.otel.contains("reqwest=off"));

    // Should have reasonable console defaults
    assert!(filter.console.contains("info"));
    assert!(filter.console.contains("opentelemetry=debug"));
}

#[test]
fn test_protocol_config_defaults() {
    let protocol = observlib::abscissa::ProtocolConfig::default();
    assert_eq!(protocol.format, "http-binary");
}

#[test]
fn test_config_serialization() {
    let config = ObservabilityConfig {
        enabled: true,
        service_name: "test-service".to_string(),
        endpoint: "localhost:4318".to_string(),
        attributes: {
            let mut map = HashMap::new();
            map.insert("env".to_string(), "test".to_string());
            map
        },
        protocol: observlib::abscissa::ProtocolConfig {
            format: "http-json".to_string(),
        },
        filters: observlib::abscissa::FilterConfig {
            otel: "debug".to_string(),
            console: "info".to_string(),
        },
    };

    // Test that config can be serialized and deserialized
    let toml_str = toml::to_string(&config).unwrap();
    let deserialized: ObservabilityConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(deserialized.service_name, "test-service");
    assert_eq!(deserialized.endpoint, "localhost:4318");
    assert_eq!(deserialized.attributes.get("env").unwrap(), "test");
    assert_eq!(deserialized.protocol.format, "http-json");
}

#[test]
fn test_config_from_toml() {
    let toml_str = r#"
        enabled = true
        service_name = "test-service"
        endpoint = "collector.example.com:4318"

        [attributes]
        env = "production"
        version = "1.2.3"

        [protocol]
        format = "http-json"

        [filters]
        otel = "debug"
        console = "trace"
    "#;

    let config: ObservabilityConfig = toml::from_str(toml_str).unwrap();

    assert!(config.enabled);
    assert_eq!(config.service_name, "test-service");
    assert_eq!(config.endpoint, "collector.example.com:4318");
    assert_eq!(config.attributes.get("env").unwrap(), "production");
    assert_eq!(config.attributes.get("version").unwrap(), "1.2.3");
    assert_eq!(config.protocol.format, "http-json");
    assert_eq!(config.filters.otel, "debug");
    assert_eq!(config.filters.console, "trace");
}

#[test]
fn test_config_minimal_toml() {
    let toml_str = r#"
        service_name = "minimal-service"
    "#;

    let config: ObservabilityConfig = toml::from_str(toml_str).unwrap();

    // Should use defaults for everything else
    assert!(config.enabled);
    assert_eq!(config.service_name, "minimal-service");
    assert_eq!(config.endpoint, "localhost:4318");
    assert!(config.attributes.is_empty());
}

#[test]
fn test_env_override_simulation() {
    let mut config = ObservabilityConfig::default();
    config.service_name = "original-service".to_string();
    config.endpoint = "localhost:4318".to_string();

    // Simulate what apply_env_overrides would do
    unsafe {
        std::env::set_var("OTEL_SERVICE_NAME", "overridden-service");
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "prod:4318");
    }

    config.apply_env_overrides();

    assert_eq!(config.service_name, "overridden-service");
    assert_eq!(config.endpoint, "prod:4318");

    // Cleanup
    unsafe {
        std::env::remove_var("OTEL_SERVICE_NAME");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    }
}

#[test]
fn test_config_validation_with_valid_ports() {
    let test_cases = vec![
        ("localhost:4318", true),
        ("collector.example.com:4318", true),
        ("127.0.0.1:4318", true),
        ("otel:9090", true),
        ("invalid-no-port", false),
        ("", false),
    ];

    for (endpoint, should_be_valid) in test_cases {
        let mut config = ObservabilityConfig::default();
        config.service_name = "test".to_string();
        config.endpoint = endpoint.to_string();
        config.enabled = true;

        let result = config.validate();
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Expected {} to be {}, got {:?}",
            endpoint,
            should_be_valid,
            result
        );
    }
}
