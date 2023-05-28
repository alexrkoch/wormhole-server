//! Values and utilities related to logging configuration

use tracing_appender::non_blocking::WorkerGuard;
use tracing_core::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, Registry};

const LOG_FILE_PREFIX: &str = "log";
const LOG_FILE_DIRECTORY: &str = "./log";

pub fn configure_tracing() -> anyhow::Result<Option<WorkerGuard>> {
    let mut guard: Option<WorkerGuard> = None;
    let subscriber = Registry::default();

    #[cfg(feature = "stream-logging")]
    let subscriber = {
        let s = subscriber.with(fmt::layer());
        s
    };

    #[cfg(feature = "file-logging")]
    let subscriber = {
        let file_appender = tracing_appender::rolling::hourly(LOG_FILE_PREFIX, LOG_FILE_DIRECTORY);
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender);
        guard = Some(worker_guard);
        let s = subscriber.with(fmt::layer().with_writer(file_writer));
        s
    };

    let targets_filter = Targets::default().with_default(if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    });

    let subscriber = subscriber.with(targets_filter);

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Set up logging subscriber");
    Ok(guard)
}
