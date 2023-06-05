//! Values and utilities related to logging configuration

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, Registry};

const LOG_LEVEL_ENV_VAR: &str = "WORMHOLE_LOG_LEVEL";
const LOG_FILE_PREFIX: &str = "log";
const LOG_FILE_DIRECTORY: &str = "./log";

pub fn configure_tracing() -> anyhow::Result<Option<WorkerGuard>> {
    let mut _guard: Option<WorkerGuard> = None;
    let subscriber = Registry::default();

    #[cfg(feature = "stream-logging")]
    let subscriber = {
        let layer = fmt::layer()
            .event_format(tracing_subscriber::fmt::format().compact())
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
        subscriber.with(layer)
    };

    #[cfg(feature = "file-logging")]
    let subscriber = {
        let file_appender = tracing_appender::rolling::hourly(LOG_FILE_PREFIX, LOG_FILE_DIRECTORY);
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender);
        _guard = Some(worker_guard);
        let layer = fmt::layer()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_writer(file_writer);
        subscriber.with(layer)
    };
    let default_level_filter = if cfg!(debug_assertions) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(default_level_filter.into())
        .with_env_var(LOG_LEVEL_ENV_VAR)
        .from_env_lossy();

    let subscriber = subscriber.with(env_filter);

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Set up logging subscriber");
    Ok(_guard)
}
