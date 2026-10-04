//! Command-line entrypoints for Belvedere.

use std::{
    io::{self, Write},
    process::ExitCode,
};

use tracing::{error, info, instrument};

use crate::{BelvedereError, BelvedereLandscapeDiffRunReport, BelvedereRunReport};

mod command;
mod errors;
mod output;

pub use command::BelvedereCli;
pub use errors::{BelvedereCliOutputError, BelvedereCliUsageError};
pub use output::{BELVEDERE_CLI_HELP, BelvedereCliOutput};

/// Report emitted by a successful CLI action.
#[derive(Debug, Clone)]
pub enum BelvedereCliReport {
    /// A normal Belvedere pipeline report.
    Run(Box<BelvedereRunReport>),
    /// A certified landscape diff report.
    LandscapeDiff(Box<BelvedereLandscapeDiffRunReport>),
}

/// Run the Belvedere CLI and return a process exit code.
#[instrument(level = "info", skip(args))]
pub fn run_cli(args: impl IntoIterator<Item = String>) -> ExitCode {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    run_cli_with_writer(args, &mut writer)
}

/// Run the Belvedere CLI with an explicit output writer.
///
/// This is the testable form of the CLI API. The process entrypoint uses
/// stdout, while tests and embedders may provide any writer they control.
#[instrument(level = "info", skip(args, writer))]
pub fn run_cli_with_writer(args: impl IntoIterator<Item = String>, writer: impl Write) -> ExitCode {
    match run_cli_report(args, writer) {
        Ok(Some(BelvedereCliReport::Run(report))) => {
            info!(
                claims = report.landscape_receipt().claim_count(),
                proofs = report.landscape_receipt().proof_count(),
                gaps = report.landscape_receipt().gap_count(),
                sections = report.ui_ir_receipt().counts().section_count(),
                trace_widgets = report.ui_ir_receipt().counts().widget_count(),
                tree_nodes = report.ui_ir_receipt().counts().node_count(),
                registry_records = report.registry_receipt().source_record_ids().len(),
                assessment_records = report.assessment_receipt().source_record_ids().len(),
                "belvedere pipeline complete"
            );
            ExitCode::SUCCESS
        }
        Ok(Some(BelvedereCliReport::LandscapeDiff(report))) => {
            info!(
                added = report.diff_receipt().counts().added_items(),
                removed = report.diff_receipt().counts().removed_items(),
                changed = report.diff_receipt().counts().changed_items(),
                total = report.diff_receipt().counts().total_items(),
                source_records = report.diff_receipt().source_record_ids().len(),
                "belvedere landscape diff complete"
            );
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(error) => {
            error!(%error, "belvedere pipeline failed");
            ExitCode::FAILURE
        }
    }
}

#[instrument(level = "debug", skip(args, writer))]
fn run_cli_report(
    args: impl IntoIterator<Item = String>,
    writer: impl Write,
) -> Result<Option<BelvedereCliReport>, BelvedereError> {
    let command = BelvedereCli::from_args(args)?;
    let output = command.output();
    if output == BelvedereCliOutput::Help {
        output::write_cli_help(writer)?;
        return Ok(None);
    }

    let Some(report) = command.act()? else {
        return Err(BelvedereCliUsageError::new().into());
    };
    output::write_cli_output(output, &report, writer)?;
    Ok(Some(report))
}
