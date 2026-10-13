//! Output artifact list routing for Belvedere artifact manifests.

mod diff;
mod normal;

pub(super) use diff::diff_run_outputs;
pub(super) use normal::normal_run_outputs;
