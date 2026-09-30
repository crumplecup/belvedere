use tracing::info;
use tracing::instrument;

#[instrument(level = "info")]
fn main() {
    belvedere::init_tracing();
    info!("belvedere cli initialized");
}
