//! Values and utilities related to logging configuration

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, Layer, Registry};

const LOG_LEVEL_ENV_VAR: &str = "WORMHOLE_LOG_LEVEL";
const LOG_FILE_PREFIX: &str = "log";
const LOG_FILE_DIRECTORY: &str = "./log";

fn get_stdout_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fmt::layer()
        .event_format(tracing_subscriber::fmt::format().compact())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
}

fn get_file_layer<S>() -> (impl Layer<S>, WorkerGuard)
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let file_appender = tracing_appender::rolling::hourly(LOG_FILE_PREFIX, LOG_FILE_DIRECTORY);
    let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender);

    let layer = fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(file_writer);

    (layer, worker_guard)
}

fn get_env_filter_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let default_level_filter = if cfg!(debug_assertions) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    EnvFilter::builder()
        .with_default_directive(default_level_filter.into())
        .with_env_var(LOG_LEVEL_ENV_VAR)
        .from_env_lossy()
}

pub fn configure_tracing() -> anyhow::Result<WorkerGuard> {
    let (file_layer, worker_guard) = get_file_layer();
    let subscriber = Registry::default()
        .with(get_stdout_layer())
        .with(file_layer)
        .with(get_env_filter_layer());

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Set up logging subscriber");
    Ok(worker_guard)
}
