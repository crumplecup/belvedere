//! CLI output contracts and writers.

mod help;
mod landscape_diff_output;
mod run_output;

use std::{fs::File, io::Write, path::PathBuf};

use serde::Serialize;
use tracing::instrument;

use crate::BelvedereError;

use super::{BelvedereCliOutputError, BelvedereCliReport};
pub use help::BELVEDERE_CLI_HELP;
use landscape_diff_output::write_landscape_diff_output;
use run_output::write_run_output;

/// Public CLI output contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BelvedereCliOutput {
    /// Write CLI help text.
    Help,
    /// Log the run summary through the configured tracing subscriber.
    LogSummary,
    /// Write a selected artifact to a selected output sink.
    Artifact {
        /// Artifact requested by the command line.
        artifact: BelvedereCliArtifact,
        /// Output sink requested by the command line.
        sink: BelvedereCliOutputSink,
    },
}

/// Artifact selected by the CLI output flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BelvedereCliArtifact {
    /// Complete normal run certificate.
    RunCertificate,
    /// Normal run artifact manifest.
    RunManifest,
    /// Normal run registry ingestion receipt.
    RunRegistryReceipt,
    /// Normal run assessment ingestion receipt.
    RunAssessmentReceipt,
    /// Normal run landscape graph.
    RunLandscape,
    /// Normal run landscape receipt.
    RunLandscapeReceipt,
    /// Normal run auditor navigation view model.
    RunViewModel,
    /// Normal run backend coverage items.
    RunBackendCoverage,
    /// Normal run claim explorer items.
    RunClaimExplorer,
    /// Normal run claim detail deep-dive items.
    RunClaimDetails,
    /// Normal run filter facets.
    RunFilterFacets,
    /// Normal run source navigation map.
    RunSourceNavigation,
    /// Normal run proof-chain deep-dive items.
    RunProofChains,
    /// Normal run auditor review workflow queue.
    RunReviewWorkflow,
    /// Normal run assessment history items.
    RunAssessmentHistory,
    /// Normal run assessment queue items.
    RunAssessmentQueue,
    /// Normal run assessment work items.
    RunAssessmentWork,
    /// Normal run witness surface items.
    RunWitnessSurface,
    /// Normal run provenance root items.
    RunProvenanceRoots,
    /// Normal run conservative gap items.
    RunGaps,
    /// Normal run view-model receipt.
    RunViewModelReceipt,
    /// Normal run UI IR projection.
    RunUiIr,
    /// Normal run UI IR receipt.
    RunUiIrReceipt,
    /// Certified landscape diff.
    LandscapeDiff,
    /// Landscape diff receipt.
    LandscapeDiffReceipt,
    /// Previous registry ingestion receipt from a diff run.
    LandscapeDiffPreviousRegistryReceipt,
    /// Previous assessment ingestion receipt from a diff run.
    LandscapeDiffPreviousAssessmentReceipt,
    /// Previous landscape graph from a diff run.
    LandscapeDiffPreviousLandscape,
    /// Previous landscape receipt from a diff run.
    LandscapeDiffPreviousLandscapeReceipt,
    /// Previous view model from a diff run.
    LandscapeDiffPreviousViewModel,
    /// Previous backend coverage items from a diff run.
    LandscapeDiffPreviousBackendCoverage,
    /// Previous claim explorer items from a diff run.
    LandscapeDiffPreviousClaimExplorer,
    /// Previous claim detail deep-dive items from a diff run.
    LandscapeDiffPreviousClaimDetails,
    /// Previous filter facets from a diff run.
    LandscapeDiffPreviousFilterFacets,
    /// Previous source navigation map from a diff run.
    LandscapeDiffPreviousSourceNavigation,
    /// Previous proof-chain deep-dive items from a diff run.
    LandscapeDiffPreviousProofChains,
    /// Previous review workflow queue from a diff run.
    LandscapeDiffPreviousReviewWorkflow,
    /// Previous assessment history items from a diff run.
    LandscapeDiffPreviousAssessmentHistory,
    /// Previous assessment queue items from a diff run.
    LandscapeDiffPreviousAssessmentQueue,
    /// Previous assessment work items from a diff run.
    LandscapeDiffPreviousAssessmentWork,
    /// Previous witness surface items from a diff run.
    LandscapeDiffPreviousWitnessSurface,
    /// Previous provenance root items from a diff run.
    LandscapeDiffPreviousProvenanceRoots,
    /// Previous conservative gap items from a diff run.
    LandscapeDiffPreviousGaps,
    /// Previous view-model receipt from a diff run.
    LandscapeDiffPreviousViewModelReceipt,
    /// Previous UI IR projection from a diff run.
    LandscapeDiffPreviousUiIr,
    /// Previous UI IR receipt from a diff run.
    LandscapeDiffPreviousUiIrReceipt,
    /// Current registry ingestion receipt from a diff run.
    LandscapeDiffCurrentRegistryReceipt,
    /// Current assessment ingestion receipt from a diff run.
    LandscapeDiffCurrentAssessmentReceipt,
    /// Current landscape graph from a diff run.
    LandscapeDiffCurrentLandscape,
    /// Current landscape receipt from a diff run.
    LandscapeDiffCurrentLandscapeReceipt,
    /// Current view model from a diff run.
    LandscapeDiffCurrentViewModel,
    /// Current backend coverage items from a diff run.
    LandscapeDiffCurrentBackendCoverage,
    /// Current claim explorer items from a diff run.
    LandscapeDiffCurrentClaimExplorer,
    /// Current claim detail deep-dive items from a diff run.
    LandscapeDiffCurrentClaimDetails,
    /// Current filter facets from a diff run.
    LandscapeDiffCurrentFilterFacets,
    /// Current source navigation map from a diff run.
    LandscapeDiffCurrentSourceNavigation,
    /// Current proof-chain deep-dive items from a diff run.
    LandscapeDiffCurrentProofChains,
    /// Current review workflow queue from a diff run.
    LandscapeDiffCurrentReviewWorkflow,
    /// Current assessment history items from a diff run.
    LandscapeDiffCurrentAssessmentHistory,
    /// Current assessment queue items from a diff run.
    LandscapeDiffCurrentAssessmentQueue,
    /// Current assessment work items from a diff run.
    LandscapeDiffCurrentAssessmentWork,
    /// Current witness surface items from a diff run.
    LandscapeDiffCurrentWitnessSurface,
    /// Current provenance root items from a diff run.
    LandscapeDiffCurrentProvenanceRoots,
    /// Current conservative gap items from a diff run.
    LandscapeDiffCurrentGaps,
    /// Current view-model receipt from a diff run.
    LandscapeDiffCurrentViewModelReceipt,
    /// Current UI IR projection from a diff run.
    LandscapeDiffCurrentUiIr,
    /// Current UI IR receipt from a diff run.
    LandscapeDiffCurrentUiIrReceipt,
    /// Landscape diff stage certificate.
    LandscapeDiffCertificate,
    /// Complete landscape diff run certificate.
    LandscapeDiffRunCertificate,
    /// Landscape diff run artifact manifest.
    LandscapeDiffManifest,
    /// Current-snapshot landscape graph from a diff run.
    LandscapeDiffLandscape,
    /// Current-snapshot landscape receipt from a diff run.
    LandscapeDiffLandscapeReceipt,
    /// Diff review view model from a diff run.
    LandscapeDiffViewModel,
    /// Current-snapshot backend coverage items from a diff run.
    LandscapeDiffBackendCoverage,
    /// Current-snapshot claim explorer items from a diff run.
    LandscapeDiffClaimExplorer,
    /// Current-snapshot claim detail deep-dive items from a diff run.
    LandscapeDiffClaimDetails,
    /// Current-snapshot filter facets from a diff run.
    LandscapeDiffFilterFacets,
    /// Current-snapshot source navigation map from a diff run.
    LandscapeDiffSourceNavigation,
    /// Current-snapshot proof-chain deep-dive items from a diff run.
    LandscapeDiffProofChains,
    /// Landscape diff review workflow queue.
    LandscapeDiffReviewWorkflow,
    /// Current-snapshot assessment history items from a diff run.
    LandscapeDiffAssessmentHistory,
    /// Current-snapshot assessment queue items from a diff run.
    LandscapeDiffAssessmentQueue,
    /// Current-snapshot assessment work items from a diff run.
    LandscapeDiffAssessmentWork,
    /// Current-snapshot witness surface items from a diff run.
    LandscapeDiffWitnessSurface,
    /// Current-snapshot provenance root items from a diff run.
    LandscapeDiffProvenanceRoots,
    /// Current-snapshot conservative gap items from a diff run.
    LandscapeDiffGaps,
    /// Landscape diff review-model receipt.
    LandscapeDiffViewModelReceipt,
    /// Landscape diff UI IR projection.
    LandscapeDiffUiIr,
    /// Landscape diff UI IR receipt.
    LandscapeDiffUiIrReceipt,
}

/// Output sink selected by the CLI output flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BelvedereCliOutputSink {
    /// Write JSON to stdout.
    Stdio,
    /// Write JSON to the supplied artifact path.
    File(PathBuf),
}

impl BelvedereCliOutput {
    /// Build an artifact output contract.
    #[must_use]
    pub const fn artifact(artifact: BelvedereCliArtifact, sink: BelvedereCliOutputSink) -> Self {
        Self::Artifact { artifact, sink }
    }
}

#[instrument(level = "info", skip(output, report, writer))]
pub(super) fn write_cli_output(
    output: BelvedereCliOutput,
    report: &BelvedereCliReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match output {
        BelvedereCliOutput::Help => write_cli_help(writer),
        BelvedereCliOutput::LogSummary => Ok(()),
        BelvedereCliOutput::Artifact { artifact, sink } => match report {
            BelvedereCliReport::Run(run) => write_run_output(artifact, &sink, run, writer),
            BelvedereCliReport::LandscapeDiff(diff) => {
                write_landscape_diff_output(artifact, &sink, diff, writer)
            }
        },
    }
}

#[instrument(level = "info", skip(writer))]
pub(super) fn write_cli_help(mut writer: impl Write) -> Result<(), BelvedereError> {
    writer
        .write_all(BELVEDERE_CLI_HELP.as_bytes())
        .map_err(BelvedereCliOutputError::write)?;
    Ok(())
}

#[instrument(level = "info", skip(value, sink, writer))]
pub(super) fn write_json_to_sink(
    value: &impl Serialize,
    sink: &BelvedereCliOutputSink,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match sink {
        BelvedereCliOutputSink::Stdio => write_json(value, writer),
        BelvedereCliOutputSink::File(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(value, file)
        }
    }
}

#[instrument(level = "info", skip(value, writer))]
fn write_json(value: &impl Serialize, mut writer: impl Write) -> Result<(), BelvedereError> {
    serde_json::to_writer_pretty(&mut writer, value).map_err(BelvedereCliOutputError::serialize)?;
    writer
        .write_all(b"\n")
        .map_err(BelvedereCliOutputError::write)?;
    Ok(())
}
