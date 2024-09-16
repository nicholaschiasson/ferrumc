use crate::utils::constants::DEFAULT_LOG_LEVEL;
use crate::utils::prelude::*;
use tokio::time::Instant;
use tracing::{error, info, Id, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

pub mod binary_utils;
pub mod components;
pub mod config;
pub mod constants;
pub mod encoding;
pub mod error;
pub mod hash;
pub mod impls;
pub mod prelude;
mod profiler;

/// Sets up the logger. Needs to be run before anything else in order for logging to run.
pub fn setup_logger() -> Result<()> {
    let trace_level = std::env::args()
        .find(|arg| arg.starts_with("--log="))
        .map(|arg| arg.replace("--log=", ""));

    let mut trace_level = trace_level.as_deref().unwrap_or("");
    if trace_level.is_empty() {
        eprintln!(
            "No log level specified, using default: {}",
            DEFAULT_LOG_LEVEL
        );
        trace_level = DEFAULT_LOG_LEVEL;
    }

    let trace_level = match trace_level.trim().parse::<tracing::Level>() {
        Ok(level) => level,
        Err(_) => {
            eprintln!("Invalid log level: {}", trace_level);
            eprintln!("Possible values: trace, debug, info, warn, error");
            std::process::exit(1);
        }
    };

    let env_filter =
        tracing_subscriber::EnvFilter::from_default_env().add_directive(trace_level.into());

    let fmt_layer = if trace_level == tracing::Level::INFO {
        // remove path from logs if log level is info
        tracing_subscriber::fmt::Layer::default()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
    } else {
        if cfg!(debug_assertions) {
            tracing_subscriber::fmt::Layer::default().with_line_number(true)
        } else {
            tracing_subscriber::fmt::Layer::default().with_file(false)
        }
    };

    let profile_layer = profiler::ProfilerTracingLayer::default();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(profile_layer)
        .init();

    Ok(())
}
