//! CLI output contracts and writers.

use std::{fs::File, io::Write, path::PathBuf};

use serde::Serialize;
use tracing::instrument;

use crate::{BelvedereError, BelvedereLandscapeDiffRunReport, BelvedereRunReport};

use super::{BelvedereCliOutputError, BelvedereCliReport, BelvedereCliUsageError};

/// Stable CLI help text.
pub const BELVEDERE_CLI_HELP: &str = "\
usage: belvedere [--help] [--certificate-json | --certificate-file <path> | --registry-receipt-json | --registry-receipt-file <path> | --assessment-receipt-json | --assessment-receipt-file <path> | --landscape-receipt-json | --landscape-receipt-file <path> | --view-model-json | --view-model-file <path> | --view-model-receipt-json | --view-model-receipt-file <path> | --ui-ir-json | --ui-ir-file <path> | --ui-ir-receipt-json | --ui-ir-receipt-file <path>] <registry-dump.json> <proof-assessments.jsonl>
       belvedere diff [--certificate-json | --certificate-file <path> | --run-certificate-json | --run-certificate-file <path> | --diff-json | --diff-receipt-json | --diff-receipt-file <path> | --previous-registry-receipt-json | --previous-registry-receipt-file <path> | --previous-assessment-receipt-json | --previous-assessment-receipt-file <path> | --current-registry-receipt-json | --current-registry-receipt-file <path> | --current-assessment-receipt-json | --current-assessment-receipt-file <path> | --previous-landscape-receipt-json | --previous-landscape-receipt-file <path> | --current-landscape-receipt-json | --current-landscape-receipt-file <path> | --current-view-model-json | --current-view-model-file <path> | --current-view-model-receipt-json | --current-view-model-receipt-file <path> | --current-ui-ir-json | --current-ui-ir-file <path> | --current-ui-ir-receipt-json | --current-ui-ir-receipt-file <path> | --landscape-receipt-json | --landscape-receipt-file <path> | --view-model-json | --view-model-file <path> | --view-model-receipt-json | --view-model-receipt-file <path> | --ui-ir-json | --ui-ir-file <path> | --ui-ir-receipt-json | --ui-ir-receipt-file <path>] <previous-registry-dump.json> <previous-proof-assessments.jsonl> <current-registry-dump.json> <current-proof-assessments.jsonl>

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
      --previous-landscape-receipt-json Write the previous landscape receipt from a diff run as JSON to stdout.
      --previous-landscape-receipt-file Write the previous landscape receipt from a diff run as JSON to a file.
      --current-landscape-receipt-json Write the current landscape receipt from a diff run as JSON to stdout.
      --current-landscape-receipt-file Write the current landscape receipt from a diff run as JSON to a file.
      --current-view-model-json Write the current view model from a diff run as JSON to stdout.
      --current-view-model-file Write the current view model from a diff run as JSON to a file.
      --current-view-model-receipt-json Write the current view-model receipt from a diff run as JSON to stdout.
      --current-view-model-receipt-file Write the current view-model receipt from a diff run as JSON to a file.
      --current-ui-ir-json      Write the current UI IR projection from a diff run as JSON to stdout.
      --current-ui-ir-file      Write the current UI IR projection from a diff run as JSON to a file.
      --current-ui-ir-receipt-json Write the current UI IR receipt from a diff run as JSON to stdout.
      --current-ui-ir-receipt-file Write the current UI IR receipt from a diff run as JSON to a file.
      --landscape-receipt-json     Write the landscape receipt as JSON to stdout.
      --landscape-receipt-file     Write the landscape receipt as JSON to a file.
      --view-model-json        Write the view model as JSON to stdout.
      --view-model-file        Write the view model as JSON to a file.
      --view-model-receipt-json    Write the view-model receipt as JSON to stdout.
      --view-model-receipt-file    Write the view-model receipt as JSON to a file.
      --ui-ir-json             Write the UI IR projection as JSON to stdout.
      --ui-ir-file             Write the UI IR projection as JSON to a file.
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
    /// Write the normal run view model as JSON.
    RunViewModelJson,
    /// Write the normal run view model as JSON to an artifact file.
    RunViewModelFile(PathBuf),
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
    /// Write the current registry ingestion receipt from a diff run as JSON.
    LandscapeDiffCurrentRegistryReceiptJson,
    /// Write the current registry ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentRegistryReceiptFile(PathBuf),
    /// Write the current assessment ingestion receipt from a diff run as JSON.
    LandscapeDiffCurrentAssessmentReceiptJson,
    /// Write the current assessment ingestion receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentAssessmentReceiptFile(PathBuf),
    /// Write the previous landscape receipt from a diff run as JSON.
    LandscapeDiffPreviousLandscapeReceiptJson,
    /// Write the previous landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffPreviousLandscapeReceiptFile(PathBuf),
    /// Write the current landscape receipt from a diff run as JSON.
    LandscapeDiffCurrentLandscapeReceiptJson,
    /// Write the current landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentLandscapeReceiptFile(PathBuf),
    /// Write the current view model from a diff run as JSON.
    LandscapeDiffCurrentViewModelJson,
    /// Write the current view model from a diff run as JSON to an artifact file.
    LandscapeDiffCurrentViewModelFile(PathBuf),
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
    /// Write the current-snapshot landscape receipt from a diff run as JSON.
    LandscapeDiffLandscapeReceiptJson,
    /// Write the current-snapshot landscape receipt from a diff run as JSON to an artifact file.
    LandscapeDiffLandscapeReceiptFile(PathBuf),
    /// Write the current-snapshot view model from a diff run as JSON.
    LandscapeDiffViewModelJson,
    /// Write the current-snapshot view model from a diff run as JSON to an artifact file.
    LandscapeDiffViewModelFile(PathBuf),
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

#[instrument(level = "info", skip(output, run, writer))]
fn write_run_output(
    output: BelvedereCliOutput,
    run: &BelvedereRunReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match output {
        BelvedereCliOutput::RunCertificateJson => write_json(run.certificate(), writer),
        BelvedereCliOutput::RunCertificateFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.certificate(), file)
        }
        BelvedereCliOutput::RunRegistryReceiptJson => write_json(run.registry_receipt(), writer),
        BelvedereCliOutput::RunRegistryReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.registry_receipt(), file)
        }
        BelvedereCliOutput::RunAssessmentReceiptJson => {
            write_json(run.assessment_receipt(), writer)
        }
        BelvedereCliOutput::RunAssessmentReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.assessment_receipt(), file)
        }
        BelvedereCliOutput::RunLandscapeReceiptJson => write_json(run.landscape_receipt(), writer),
        BelvedereCliOutput::RunLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.landscape_receipt(), file)
        }
        BelvedereCliOutput::RunViewModelJson => write_json(run.view_model(), writer),
        BelvedereCliOutput::RunViewModelFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.view_model(), file)
        }
        BelvedereCliOutput::RunViewModelReceiptJson => write_json(run.view_model_receipt(), writer),
        BelvedereCliOutput::RunViewModelReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.view_model_receipt(), file)
        }
        BelvedereCliOutput::RunUiIrJson => write_json(run.projection(), writer),
        BelvedereCliOutput::RunUiIrFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.projection(), file)
        }
        BelvedereCliOutput::RunUiIrReceiptJson => write_json(run.ui_ir_receipt(), writer),
        BelvedereCliOutput::RunUiIrReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.ui_ir_receipt(), file)
        }
        _ => Err(BelvedereCliUsageError::new().into()),
    }
}

#[instrument(level = "info", skip(output, diff, writer))]
fn write_landscape_diff_output(
    output: BelvedereCliOutput,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match output {
        BelvedereCliOutput::LandscapeDiffCertificateJson => {
            write_json(diff.diff_certificate(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCertificateFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_certificate(), file)
        }
        BelvedereCliOutput::LandscapeDiffRunCertificateJson => {
            write_json(diff.certificate(), writer)
        }
        BelvedereCliOutput::LandscapeDiffRunCertificateFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.certificate(), file)
        }
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson => {
            write_json(diff.current_landscape_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptJson => {
            write_json(diff.previous_landscape_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_landscape_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptJson => {
            write_json(diff.current_landscape_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelJson => {
            write_json(diff.current_view_model(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_view_model(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptJson => {
            write_json(diff.current_view_model_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_view_model_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrJson => {
            write_json(diff.current_projection(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_projection(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptJson => {
            write_json(diff.current_ui_ir_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_ui_ir_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffViewModelJson => {
            write_json(diff.current_view_model(), writer)
        }
        BelvedereCliOutput::LandscapeDiffViewModelFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_view_model(), file)
        }
        BelvedereCliOutput::LandscapeDiffViewModelReceiptJson => {
            write_json(diff.current_view_model_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_view_model_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffUiIrJson => write_json(diff.current_projection(), writer),
        BelvedereCliOutput::LandscapeDiffUiIrFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_projection(), file)
        }
        BelvedereCliOutput::LandscapeDiffUiIrReceiptJson => {
            write_json(diff.current_ui_ir_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_ui_ir_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffJson => write_json(diff.diff(), writer),
        BelvedereCliOutput::LandscapeDiffReceiptJson => write_json(diff.diff_receipt(), writer),
        BelvedereCliOutput::LandscapeDiffReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptJson => {
            write_json(diff.previous_registry_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_registry_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptJson => {
            write_json(diff.previous_assessment_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_assessment_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptJson => {
            write_json(diff.current_registry_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_registry_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptJson => {
            write_json(diff.current_assessment_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptFile(path) => {
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
