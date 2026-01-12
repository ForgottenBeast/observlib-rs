use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObservlibError {
    #[error("Failed to shutdown tracer provider: {0}")]
    TracerShutdown(String),

    #[error("Failed to shutdown meter provider: {0}")]
    MeterShutdown(String),

    #[error("Failed to shutdown logger provider: {0}")]
    LoggerShutdown(String),

    #[error("Multiple shutdown failures: {0}")]
    MultipleShutdownFailures(String),

    #[error("Shutdown timeout exceeded")]
    ShutdownTimeout,

    #[cfg(feature = "async")]
    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}
