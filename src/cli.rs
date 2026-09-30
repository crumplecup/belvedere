//! Command-line entrypoints for Belvedere.

use std::{panic::Location, process::ExitCode};

use derive_getters::Getters;
use thiserror::Error;
use tracing::{error, info, instrument};

use crate::{BelvedereError, BelvedereRunInput, BelvedereRunReport, run_belvedere_pipeline};

/// Parsed Belvedere command-line invocation.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct BelvedereCli {
    input: BelvedereRunInput,
}

impl BelvedereCli {
    /// Parse a Belvedere pipeline run from positional artifact paths.
    ///
    /// # Errors
    ///
    /// Returns a usage error unless exactly two artifact paths are supplied.
    #[instrument(level = "debug", skip(args))]
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self, BelvedereError> {
        let mut args = args.into_iter();
        let Some(registry_path) = args.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        let Some(assessments_path) = args.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        if args.next().is_some() {
            return Err(BelvedereCliUsageError::new().into());
        }
        Ok(Self {
            input: BelvedereRunInput::new(registry_path, assessments_path),
        })
    }

    /// Execute the parsed command.
    ///
    /// # Errors
    ///
    /// Returns an error when artifact collection or any pipeline exchange
    /// rejects its input evidence.
    #[instrument(level = "debug", skip(self))]
    pub fn act(self) -> Result<BelvedereRunReport, BelvedereError> {
        run_belvedere_pipeline(self.input)
    }
}

/// Error emitted when the Belvedere CLI receives invalid arguments.
#[derive(Debug, Error)]
#[error("usage: belvedere <registry-dump.json> <proof-assessments.jsonl> at {file}:{line}")]
pub struct BelvedereCliUsageError {
    file: String,
    line: u32,
}

impl BelvedereCliUsageError {
    #[instrument(level = "debug")]
    #[track_caller]
    pub(crate) fn new() -> Self {
        let location = Location::caller();
        Self {
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

/// Run the Belvedere CLI and return a process exit code.
#[instrument(level = "info", skip(args))]
pub fn run_cli(args: impl IntoIterator<Item = String>) -> ExitCode {
    match BelvedereCli::from_args(args).and_then(BelvedereCli::act) {
        Ok(report) => {
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
        Err(error) => {
            error!(%error, "belvedere pipeline failed");
            ExitCode::FAILURE
        }
    }
}
