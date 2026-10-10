//! CLI output contracts and writers.

mod help;
mod landscape_diff_output;
mod run_output;

use std::{io::Write, path::PathBuf};

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
    /// Write the complete run certificate as JSON.
    RunCertificateJson,
    /// Write the complete run certificate as JSON to an artifact file.
    RunCertificateFile(PathBuf),
    /// Write the normal run artifact manifest as JSON.
    RunManifestJson,
    /// Write the normal run artifact manifest as JSON to an artifact file.
    RunManifestFile(PathBuf),
    /// Write the normal run registry ingestion receipt as JSON.
    RunRegistryReceiptJson,
    /// Write the normal run registry ingestion receipt as JSON to an artifact file.
    RunRegistryReceiptFile(PathBuf),
    /// Write the normal run assessment ingestion receipt as JSON.
    RunAssessmentReceiptJson,
    /// Write the normal run assessment ingestion receipt as JSON to an artifact file.
    RunAssessmentReceiptFile(PathBuf),
    /// Write the normal run landscape graph as JSON.
    RunLandscapeJson,
    /// Write the normal run landscape graph as JSON to an artifact file.
    RunLandscapeFile(PathBuf),
    /// Write the normal run landscape receipt as JSON.
    RunLandscapeReceiptJson,
    /// Write the normal run landscape receipt as JSON to an artifact file.
    RunLandscapeReceiptFile(PathBuf),
    /// Write the normal run view model as JSON.
    RunViewModelJson,
    /// Write the normal run view model as JSON to an artifact file.
    RunViewModelFile(PathBuf),
    /// Write the normal run filter facets as JSON.
    RunFilterFacetsJson,
    /// Write the normal run filter facets as JSON to an artifact file.
    RunFilterFacetsFile(PathBuf),
    /// Write the normal run source navigation map as JSON.
    RunSourceNavigationJson,
    /// Write the normal run source navigation map as JSON to an artifact file.
    RunSourceNavigationFile(PathBuf),
    /// Write the normal run proof-chain deep-dive items as JSON.
    RunProofChainsJson,
    /// Write the normal run proof-chain deep-dive items as JSON to an artifact file.
    RunProofChainsFile(PathBuf),
    /// Write the normal run auditor review workflow queue as JSON.
    RunReviewWorkflowJson,
    /// Write the normal run auditor review workflow queue as JSON to an artifact file.
    RunReviewWorkflowFile(PathBuf),
    /// Write the normal run view-model receipt as JSON.
    RunViewModelReceiptJson,
    /// Write the normal run view-model receipt as JSON to an artifact file.
    RunViewModelReceiptFile(PathBuf),
    /// Write the normal run UI IR projection as JSON.
    RunUiIrJson,
    /// Write the normal run UI IR projection as JSON to an artifact file.
    RunUiIrFile(PathBuf),
    /// Write the normal run UI IR receipt as JSON.
    RunUiIrReceiptJson,
    /// Write the normal run UI IR receipt as JSON to an artifact file.
    RunUiIrReceiptFile(PathBuf),
    /// Write the certified landscape diff as JSON.
    LandscapeDiffJson,
    /// Write the landscape diff receipt as JSON.
    LandscapeDiffReceiptJson,
    /// Write the landscape diff receipt as JSON to an artifact file.
    LandscapeDiffReceiptFile(PathBuf),
    /// Write the previous registry ingestion receipt from a diff run as JSON.
    LandscapeDiffPreviousRegistryReceiptJson,
    /// Write the previous registry ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousRegistryReceiptFile(PathBuf),
    /// Write the previous assessment ingestion receipt from a diff run as JSON.
    LandscapeDiffPreviousAssessmentReceiptJson,
    /// Write the previous assessment ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousAssessmentReceiptFile(PathBuf),
    /// Write the previous landscape graph from a diff run as JSON.
    LandscapeDiffPreviousLandscapeJson,
    /// Write the previous landscape graph from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousLandscapeFile(PathBuf),
    /// Write the current registry ingestion receipt from a diff run as JSON.
    LandscapeDiffCurrentRegistryReceiptJson,
    /// Write the current registry ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentRegistryReceiptFile(PathBuf),
    /// Write the current assessment ingestion receipt from a diff run as JSON.
    LandscapeDiffCurrentAssessmentReceiptJson,
    /// Write the current assessment ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentAssessmentReceiptFile(PathBuf),
    /// Write the current landscape graph from a diff run as JSON.
    LandscapeDiffCurrentLandscapeJson,
    /// Write the current landscape graph from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentLandscapeFile(PathBuf),
    /// Write the previous landscape receipt from a diff run as JSON.
    LandscapeDiffPreviousLandscapeReceiptJson,
    /// Write the previous landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousLandscapeReceiptFile(PathBuf),
    /// Write the previous view model from a diff run as JSON.
    LandscapeDiffPreviousViewModelJson,
    /// Write the previous view model from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousViewModelFile(PathBuf),
    /// Write the previous filter facets from a diff run as JSON.
    LandscapeDiffPreviousFilterFacetsJson,
    /// Write the previous filter facets from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousFilterFacetsFile(PathBuf),
    /// Write the previous source navigation map from a diff run as JSON.
    LandscapeDiffPreviousSourceNavigationJson,
    /// Write the previous source navigation map from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousSourceNavigationFile(PathBuf),
    /// Write the previous proof-chain deep-dive items from a diff run as JSON.
    LandscapeDiffPreviousProofChainsJson,
    /// Write the previous proof-chain deep-dive items from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousProofChainsFile(PathBuf),
    /// Write the previous view-model receipt from a diff run as JSON.
    LandscapeDiffPreviousViewModelReceiptJson,
    /// Write the previous view-model receipt from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousViewModelReceiptFile(PathBuf),
    /// Write the previous UI IR projection from a diff run as JSON.
    LandscapeDiffPreviousUiIrJson,
    /// Write the previous UI IR projection from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousUiIrFile(PathBuf),
    /// Write the previous UI IR receipt from a diff run as JSON.
    LandscapeDiffPreviousUiIrReceiptJson,
    /// Write the previous UI IR receipt from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousUiIrReceiptFile(PathBuf),
    /// Write the current landscape receipt from a diff run as JSON.
    LandscapeDiffCurrentLandscapeReceiptJson,
    /// Write the current landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentLandscapeReceiptFile(PathBuf),
    /// Write the current view model from a diff run as JSON.
    LandscapeDiffCurrentViewModelJson,
    /// Write the current view model from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentViewModelFile(PathBuf),
    /// Write the current filter facets from a diff run as JSON.
    LandscapeDiffCurrentFilterFacetsJson,
    /// Write the current filter facets from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentFilterFacetsFile(PathBuf),
    /// Write the current source navigation map from a diff run as JSON.
    LandscapeDiffCurrentSourceNavigationJson,
    /// Write the current source navigation map from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentSourceNavigationFile(PathBuf),
    /// Write the current proof-chain deep-dive items from a diff run as JSON.
    LandscapeDiffCurrentProofChainsJson,
    /// Write the current proof-chain deep-dive items from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentProofChainsFile(PathBuf),
    /// Write the current view-model receipt from a diff run as JSON.
    LandscapeDiffCurrentViewModelReceiptJson,
    /// Write the current view-model receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentViewModelReceiptFile(PathBuf),
    /// Write the current UI IR projection from a diff run as JSON.
    LandscapeDiffCurrentUiIrJson,
    /// Write the current UI IR projection from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentUiIrFile(PathBuf),
    /// Write the current UI IR receipt from a diff run as JSON.
    LandscapeDiffCurrentUiIrReceiptJson,
    /// Write the current UI IR receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentUiIrReceiptFile(PathBuf),
    /// Write the landscape diff stage certificate as JSON.
    LandscapeDiffCertificateJson,
    /// Write the landscape diff stage certificate as JSON to an artifact file.
    LandscapeDiffCertificateFile(PathBuf),
    /// Write the complete landscape diff run certificate as JSON.
    LandscapeDiffRunCertificateJson,
    /// Write the complete landscape diff run certificate as JSON to an artifact file.
    LandscapeDiffRunCertificateFile(PathBuf),
    /// Write the landscape diff run artifact manifest as JSON.
    LandscapeDiffManifestJson,
    /// Write the landscape diff run artifact manifest as JSON to an artifact file.
    LandscapeDiffManifestFile(PathBuf),
    /// Write the current-snapshot landscape graph from a diff run as JSON.
    LandscapeDiffLandscapeJson,
    /// Write the current-snapshot landscape graph from a diff run as JSON to an artifact file.
    LandscapeDiffLandscapeFile(PathBuf),
    /// Write the current-snapshot landscape receipt from a diff run as JSON.
    LandscapeDiffLandscapeReceiptJson,
    /// Write the current-snapshot landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffLandscapeReceiptFile(PathBuf),
    /// Write the current-snapshot view model from a diff run as JSON.
    LandscapeDiffViewModelJson,
    /// Write the current-snapshot view model from a diff run as JSON to an artifact file.
    LandscapeDiffViewModelFile(PathBuf),
    /// Write the current-snapshot filter facets from a diff run as JSON.
    LandscapeDiffFilterFacetsJson,
    /// Write the current-snapshot filter facets from a diff run as JSON to an artifact file.
    LandscapeDiffFilterFacetsFile(PathBuf),
    /// Write the current-snapshot source navigation map from a diff run as JSON.
    LandscapeDiffSourceNavigationJson,
    /// Write the current-snapshot source navigation map from a diff run as JSON to an artifact file.
    LandscapeDiffSourceNavigationFile(PathBuf),
    /// Write the current-snapshot proof-chain deep-dive items from a diff run as JSON.
    LandscapeDiffProofChainsJson,
    /// Write the current-snapshot proof-chain deep-dive items from a diff run as JSON to an artifact file.
    LandscapeDiffProofChainsFile(PathBuf),
    /// Write the landscape diff review workflow queue as JSON.
    LandscapeDiffReviewWorkflowJson,
    /// Write the landscape diff review workflow queue as JSON to an artifact file.
    LandscapeDiffReviewWorkflowFile(PathBuf),
    /// Write the current-snapshot view-model receipt from a diff run as JSON.
    LandscapeDiffViewModelReceiptJson,
    /// Write the current-snapshot view-model receipt from a diff run as JSON to an artifact file.
    LandscapeDiffViewModelReceiptFile(PathBuf),
    /// Write the current-snapshot UI IR projection from a diff run as JSON.
    LandscapeDiffUiIrJson,
    /// Write the current-snapshot UI IR projection from a diff run as JSON to an artifact file.
    LandscapeDiffUiIrFile(PathBuf),
    /// Write the current-snapshot UI IR receipt from a diff run as JSON.
    LandscapeDiffUiIrReceiptJson,
    /// Write the current-snapshot UI IR receipt from a diff run as JSON to an artifact file.
    LandscapeDiffUiIrReceiptFile(PathBuf),
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
        _ => match report {
            BelvedereCliReport::Run(run) => write_run_output(output, run, writer),
            BelvedereCliReport::LandscapeDiff(diff) => {
                write_landscape_diff_output(output, diff, writer)
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

#[instrument(level = "info", skip(value, writer))]
fn write_json(value: &impl Serialize, mut writer: impl Write) -> Result<(), BelvedereError> {
    serde_json::to_writer_pretty(&mut writer, value).map_err(BelvedereCliOutputError::serialize)?;
    writer
        .write_all(b"\n")
        .map_err(BelvedereCliOutputError::write)?;
    Ok(())
}
