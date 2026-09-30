use std::{env, process::ExitCode};

use tracing::instrument;

#[instrument(level = "info")]
fn main() -> ExitCode {
    belvedere::init_tracing();
    belvedere::run_cli(env::args().skip(1))
}
