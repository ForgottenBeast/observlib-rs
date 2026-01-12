use observlib::{global, KeyValue};
use std::time::Duration;

/// Comprehensive async shutdown test that covers:
/// - Initialization of telemetry
/// - Creating and using metrics
/// - Async shutdown with timeout
/// - Simulated graceful shutdown scenario
///
/// Note: Only one telemetry initialization can happen per process due to
/// global subscriber limitations. This test covers all async shutdown scenarios
/// in a single test.
#[cfg(feature = "async")]
#[tokio::test]
async fn async_shutdown_comprehensive() {
    use tokio::sync::oneshot;

    // Initialize telemetry
    let attrs = vec![KeyValue::new("env", "test-async")];
    let otel_manager = observlib::initialize_telemetry("async-test", "127.0.0.1:4318", attrs);

    // Create and use some metrics
    let counter = global::meter("async-meter")
        .u64_counter("async_counter")
        .build();
    counter.add(10, &[]);

    // Simulate some work
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate graceful shutdown scenario with oneshot channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        // Simulate shutdown signal received after some delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        shutdown_tx.send(()).unwrap();
    });

    // Wait for shutdown signal
    shutdown_rx.await.unwrap();

    // Test async shutdown with timeout
    let result = otel_manager
        .async_shutdown(Some(Duration::from_secs(5)))
        .await;

    assert!(result.is_ok(), "Async shutdown should succeed");
}
