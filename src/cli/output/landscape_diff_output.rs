//! Landscape diff CLI output writers.

use std::{fs::File, io::Write};

use tracing::instrument;

use crate::{BelvedereError, BelvedereLandscapeDiffRunReport};

use super::{BelvedereCliOutput, write_json};
use crate::cli::{BelvedereCliOutputError, BelvedereCliUsageError};

#[instrument(level = "info", skip(output, diff, writer))]
pub(super) fn write_landscape_diff_output(
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
        BelvedereCliOutput::LandscapeDiffManifestJson => write_json(diff.manifest(), writer),
        BelvedereCliOutput::LandscapeDiffManifestFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.manifest(), file)
        }
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson => {
            write_json(diff.current_landscape_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffLandscapeJson => {
            write_json(diff.current_landscape(), writer)
        }
        BelvedereCliOutput::LandscapeDiffLandscapeFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeJson => {
            write_json(diff.previous_landscape(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_landscape(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptJson => {
            write_json(diff.previous_landscape_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_landscape_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelJson => {
            write_json(diff.previous_view_model(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_view_model(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptJson => {
            write_json(diff.previous_view_model_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_view_model_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrJson => {
            write_json(diff.previous_projection(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_projection(), file)
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptJson => {
            write_json(diff.previous_ui_ir_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.previous_ui_ir_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptJson => {
            write_json(diff.current_landscape_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeJson => {
            write_json(diff.current_landscape(), writer)
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.current_landscape(), file)
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
            write_json(diff.diff_view_model(), writer)
        }
        BelvedereCliOutput::LandscapeDiffViewModelFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_view_model(), file)
        }
        BelvedereCliOutput::LandscapeDiffReviewWorkflowJson => {
            write_json(diff.diff_view_model().items(), writer)
        }
        BelvedereCliOutput::LandscapeDiffReviewWorkflowFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_view_model().items(), file)
        }
        BelvedereCliOutput::LandscapeDiffViewModelReceiptJson => {
            write_json(diff.diff_view_model_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_view_model_receipt(), file)
        }
        BelvedereCliOutput::LandscapeDiffUiIrJson => write_json(diff.diff_projection(), writer),
        BelvedereCliOutput::LandscapeDiffUiIrFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_projection(), file)
        }
        BelvedereCliOutput::LandscapeDiffUiIrReceiptJson => {
            write_json(diff.diff_ui_ir_receipt(), writer)
        }
        BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_ui_ir_receipt(), file)
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
