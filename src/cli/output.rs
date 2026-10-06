//! CLI output contracts and writers.

use std::{fs::File, io::Write, path::PathBuf};

use serde::Serialize;
use tracing::instrument;

use crate::BelvedereError;

use super::{BelvedereCliOutputError, BelvedereCliReport, BelvedereCliUsageError};

/// Stable CLI help text.
pub const BELVEDERE_CLI_HELP: &str = "\
usage: belvedere [--help] [--certificate-json | --certificate-file <path> | --registry-receipt-json | --registry-receipt-file <path> | --assessment-receipt-json | --assessment-receipt-file <path> | --landscape-receipt-json | --landscape-receipt-file <path> | --view-model-receipt-json | --view-model-receipt-file <path> | --ui-ir-receipt-json | --ui-ir-receipt-file <path>] <registry-dump.json> <proof-assessments.jsonl>
       belvedere diff [--certificate-json | --certificate-file <path> | --run-certificate-json | --run-certificate-file <path> | --diff-json | --diff-receipt-json | --diff-receipt-file <path> | --previous-registry-receipt-json | --previous-registry-receipt-file <path> | --previous-assessment-receipt-json | --previous-assessment-receipt-file <path> | --current-registry-receipt-json | --current-registry-receipt-file <path> | --current-assessment-receipt-json | --current-assessment-receipt-file <path> | --landscape-receipt-json | --landscape-receipt-file <path> | --view-model-receipt-json | --view-model-receipt-file <path> | --ui-ir-receipt-json | --ui-ir-receipt-file <path>] <previous-registry-dump.json> <previous-proof-assessments.jsonl> <current-registry-dump.json> <current-proof-assessments.jsonl>

Options:
  -h, --help                       Show this help text.
      --certificate-json           Write the requested certificate as JSON to stdout.
      --certificate-file           Write the requested certificate as JSON to a file.
      --registry-receipt-json      Write the registry ingestion receipt as JSON to stdout.
      --registry-receipt-file      Write the registry ingestion receipt as JSON to a file.
      --assessment-receipt-json    Write the assessment ingestion receipt as JSON to stdout.
      --assessment-receipt-file    Write the assessment ingestion receipt as JSON to a file.
      --run-certificate-json       Write the full diff run certificate as JSON to stdout.
      --run-certificate-file       Write the full diff run certificate as JSON to a file.
      --diff-json                  Write the landscape diff as JSON to stdout.
      --diff-receipt-json          Write the landscape diff receipt as JSON to stdout.
      --diff-receipt-file          Write the landscape diff receipt as JSON to a file.
      --previous-registry-receipt-json Write the previous registry ingestion receipt from a diff run as JSON to stdout.
      --previous-registry-receipt-file Write the previous registry ingestion receipt from a diff run as JSON to a file.
      --previous-assessment-receipt-json Write the previous assessment ingestion receipt from a diff run as JSON to stdout.
      --previous-assessment-receipt-file Write the previous assessment ingestion receipt from a diff run as JSON to a file.
      --current-registry-receipt-json Write the current registry ingestion receipt from a diff run as JSON to stdout.
      --current-registry-receipt-file Write the current registry ingestion receipt from a diff run as JSON to a file.
      --current-assessment-receipt-json Write the current assessment ingestion receipt from a diff run as JSON to stdout.
      --current-assessment-receipt-file Write the current assessment ingestion receipt from a diff run as JSON to a file.
      --landscape-receipt-json     Write the landscape receipt as JSON to stdout.
      --landscape-receipt-file     Write the landscape receipt as JSON to a file.
      --view-model-receipt-json    Write the view-model receipt as JSON to stdout.
      --view-model-receipt-file    Write the view-model receipt as JSON to a file.
      --ui-ir-receipt-json         Write the UI IR receipt as JSON to stdout.
      --ui-ir-receipt-file         Write the UI IR receipt as JSON to a file.
";

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
    /// Write the normal run registry ingestion receipt as JSON.
    RunRegistryReceiptJson,
    /// Write the normal run registry ingestion receipt as JSON to an artifact file.
    RunRegistryReceiptFile(PathBuf),
    /// Write the normal run assessment ingestion receipt as JSON.
    RunAssessmentReceiptJson,
    /// Write the normal run assessment ingestion receipt as JSON to an artifact file.
    RunAssessmentReceiptFile(PathBuf),
    /// Write the normal run landscape receipt as JSON.
    RunLandscapeReceiptJson,
    /// Write the normal run landscape receipt as JSON to an artifact file.
    RunLandscapeReceiptFile(PathBuf),
    /// Write the normal run view-model receipt as JSON.
    RunViewModelReceiptJson,
    /// Write the normal run view-model receipt as JSON to an artifact file.
    RunViewModelReceiptFile(PathBuf),
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
    /// Write the current registry ingestion receipt from a diff run as JSON.
    LandscapeDiffCurrentRegistryReceiptJson,
    /// Write the current registry ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentRegistryReceiptFile(PathBuf),
    /// Write the current assessment ingestion receipt from a diff run as JSON.
    LandscapeDiffCurrentAssessmentReceiptJson,
    /// Write the current assessment ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentAssessmentReceiptFile(PathBuf),
    /// Write the landscape diff stage certificate as JSON.
    LandscapeDiffCertificateJson,
    /// Write the landscape diff stage certificate as JSON to an artifact file.
    LandscapeDiffCertificateFile(PathBuf),
    /// Write the complete landscape diff run certificate as JSON.
    LandscapeDiffRunCertificateJson,
    /// Write the complete landscape diff run certificate as JSON to an artifact file.
    LandscapeDiffRunCertificateFile(PathBuf),
    /// Write the current-snapshot landscape receipt from a diff run as JSON.
    LandscapeDiffLandscapeReceiptJson,
    /// Write the current-snapshot landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffLandscapeReceiptFile(PathBuf),
    /// Write the current-snapshot view-model receipt from a diff run as JSON.
    LandscapeDiffViewModelReceiptJson,
    /// Write the current-snapshot view-model receipt from a diff run as JSON to an artifact file.
    LandscapeDiffViewModelReceiptFile(PathBuf),
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
    match (output, report) {
        (BelvedereCliOutput::Help, _) => write_cli_help(writer),
        (BelvedereCliOutput::LogSummary, _) => Ok(()),
        (BelvedereCliOutput::RunCertificateJson, BelvedereCliReport::Run(run)) => {
            write_json(run.certificate(), writer)
        }
        (BelvedereCliOutput::RunCertificateFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.certificate(), file)
        }
        (BelvedereCliOutput::RunRegistryReceiptJson, BelvedereCliReport::Run(run)) => {
            write_json(run.registry_receipt(), writer)
        }
        (BelvedereCliOutput::RunRegistryReceiptFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.registry_receipt(), file)
        }
        (BelvedereCliOutput::RunAssessmentReceiptJson, BelvedereCliReport::Run(run)) => {
            write_json(run.assessment_receipt(), writer)
        }
        (BelvedereCliOutput::RunAssessmentReceiptFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.assessment_receipt(), file)
        }
        (BelvedereCliOutput::RunLandscapeReceiptJson, BelvedereCliReport::Run(run)) => {
            write_json(run.landscape_receipt(), writer)
        }
        (BelvedereCliOutput::RunLandscapeReceiptFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.landscape_receipt(), file)
        }
        (BelvedereCliOutput::RunViewModelReceiptJson, BelvedereCliReport::Run(run)) => {
            write_json(run.view_model_receipt(), writer)
        }
        (BelvedereCliOutput::RunViewModelReceiptFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.view_model_receipt(), file)
        }
        (BelvedereCliOutput::RunUiIrReceiptJson, BelvedereCliReport::Run(run)) => {
            write_json(run.ui_ir_receipt(), writer)
        }
        (BelvedereCliOutput::RunUiIrReceiptFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.ui_ir_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffCertificateJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.diff_certificate(), writer),
        (
            BelvedereCliOutput::LandscapeDiffCertificateFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_certificate(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffRunCertificateJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.certificate(), writer),
        (
            BelvedereCliOutput::LandscapeDiffRunCertificateFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.certificate(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.current_landscape_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffViewModelReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.current_view_model_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_view_model_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffUiIrReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.current_ui_ir_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_ui_ir_receipt(), file)
        }
        (BelvedereCliOutput::LandscapeDiffJson, BelvedereCliReport::LandscapeDiff(diff)) => {
            write_json(diff.diff(), writer)
        }
        (BelvedereCliOutput::LandscapeDiffReceiptJson, BelvedereCliReport::LandscapeDiff(diff)) => {
            write_json(diff.diff_receipt(), writer)
        }
        (
            BelvedereCliOutput::LandscapeDiffReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.previous_registry_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_registry_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.previous_assessment_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_assessment_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.current_registry_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_registry_receipt(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.current_assessment_receipt(), writer),
        (
            BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_assessment_receipt(), file)
        }
        _ => Err(BelvedereCliUsageError::new().into()),
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
