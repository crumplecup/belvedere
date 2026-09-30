//! Tracing subscriber setup for binaries and tests.

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};

use tracing::instrument;
const DEFAULT_FILTER: &str = "belvedere=info";

/// Install Belvedere's tracing subscriber if one is not already active.
#[instrument(level = "debug")]
pub fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let _result = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init();
}
