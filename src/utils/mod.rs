use crate::utils::constants::DEFAULT_LOG_LEVEL;
use crate::utils::prelude::*;
use dashmap::DashMap;
use lazy_static::lazy_static;
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

#[derive(Default)]
struct ProfileLayer;

lazy_static! {
    static ref DURATION_MAP: DashMap<u64, Instant> = DashMap::new();
}

impl<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>>
    tracing_subscriber::Layer<S> for ProfileLayer
{
    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        match ctx.span(id) {
            None => {
                error!("No span found")
            }
            Some(span) => {
                if span.name().starts_with("profiler/") {
                    DURATION_MAP.insert(span.id().into_u64(), Instant::now());
                }
            }
        }
    }
    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        let instant = match ctx.span(id) {
            None => {
                error!("No span found");
                None
            }
            Some(span) => {
                if span.name().starts_with("profiler/") {
                    match DURATION_MAP.get(&span.id().into_u64()) {
                        Some(start) => {
                            let result = start.clone();
                            drop(start);
                            DURATION_MAP.remove(&id.into_u64());
                            Some(result)
                        }
                        None => {
                            error!("No start time found");
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };

        if let Some(start) = instant {
            let elapsed = start.elapsed();
            info!("{} took {:?}", ctx.span(id).unwrap().name(), elapsed);
        }
    }
}

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

    let profile_layer = ProfileLayer::default();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(profile_layer)
        .init();

    Ok(())
}
