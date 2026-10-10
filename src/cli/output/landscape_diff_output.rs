//! Landscape diff CLI output writers.

use std::{fs::File, io::Write, path::Path};

use serde::Serialize;
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
    let mut writer = writer;
    if write_landscape_diff_certificate_output(&output, diff, &mut writer)? {
        return Ok(());
    }
    if write_previous_snapshot_output(&output, diff, &mut writer)? {
        return Ok(());
    }
    if write_current_snapshot_output(&output, diff, &mut writer)? {
        return Ok(());
    }
    if write_current_alias_output(&output, diff, &mut writer)? {
        return Ok(());
    }
    Err(BelvedereCliUsageError::new().into())
}

#[instrument(level = "info", skip(output, diff, writer))]
fn write_landscape_diff_certificate_output(
    output: &BelvedereCliOutput,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match output {
        BelvedereCliOutput::LandscapeDiffCertificateJson => {
            write_json(diff.diff_certificate(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCertificateFile(path) => {
            write_json_file(diff.diff_certificate(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffRunCertificateJson => {
            write_json(diff.certificate(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffRunCertificateFile(path) => {
            write_json_file(diff.certificate(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffManifestJson => write_json(diff.manifest(), writer)?,
        BelvedereCliOutput::LandscapeDiffManifestFile(path) => {
            write_json_file(diff.manifest(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffJson => write_json(diff.diff(), writer)?,
        BelvedereCliOutput::LandscapeDiffReceiptJson => write_json(diff.diff_receipt(), writer)?,
        BelvedereCliOutput::LandscapeDiffReceiptFile(path) => {
            write_json_file(diff.diff_receipt(), path)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(output, diff, writer))]
fn write_previous_snapshot_output(
    output: &BelvedereCliOutput,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match output {
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeJson => {
            write_json(diff.previous_landscape(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeFile(path) => {
            write_json_file(diff.previous_landscape(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptJson => {
            write_json(diff.previous_landscape_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptFile(path) => {
            write_json_file(diff.previous_landscape_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelJson => {
            write_json(diff.previous_view_model(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelFile(path) => {
            write_json_file(diff.previous_view_model(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousFilterFacetsJson => {
            write_json(diff.previous_view_model().filter_facets(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousFilterFacetsFile(path) => {
            write_json_file(diff.previous_view_model().filter_facets(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousSourceNavigationJson => {
            write_json(diff.previous_view_model().source_navigation(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousSourceNavigationFile(path) => {
            write_json_file(diff.previous_view_model().source_navigation(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousProofChainsJson => {
            write_json(diff.previous_view_model().proof_chains(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousProofChainsFile(path) => {
            write_json_file(diff.previous_view_model().proof_chains(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptJson => {
            write_json(diff.previous_view_model_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptFile(path) => {
            write_json_file(diff.previous_view_model_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrJson => {
            write_json(diff.previous_projection(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrFile(path) => {
            write_json_file(diff.previous_projection(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptJson => {
            write_json(diff.previous_ui_ir_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptFile(path) => {
            write_json_file(diff.previous_ui_ir_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptJson => {
            write_json(diff.previous_registry_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptFile(path) => {
            write_json_file(diff.previous_registry_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptJson => {
            write_json(diff.previous_assessment_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptFile(path) => {
            write_json_file(diff.previous_assessment_receipt(), path)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(output, diff, writer))]
fn write_current_snapshot_output(
    output: &BelvedereCliOutput,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match output {
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptJson => {
            write_json(diff.current_landscape_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptFile(path) => {
            write_json_file(diff.current_landscape_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeJson => {
            write_json(diff.current_landscape(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentLandscapeFile(path) => {
            write_json_file(diff.current_landscape(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelJson => {
            write_json(diff.current_view_model(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelFile(path) => {
            write_json_file(diff.current_view_model(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentFilterFacetsJson => {
            write_json(diff.current_view_model().filter_facets(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentFilterFacetsFile(path) => {
            write_json_file(diff.current_view_model().filter_facets(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentSourceNavigationJson => {
            write_json(diff.current_view_model().source_navigation(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentSourceNavigationFile(path) => {
            write_json_file(diff.current_view_model().source_navigation(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentProofChainsJson => {
            write_json(diff.current_view_model().proof_chains(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentProofChainsFile(path) => {
            write_json_file(diff.current_view_model().proof_chains(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptJson => {
            write_json(diff.current_view_model_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptFile(path) => {
            write_json_file(diff.current_view_model_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrJson => {
            write_json(diff.current_projection(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrFile(path) => {
            write_json_file(diff.current_projection(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptJson => {
            write_json(diff.current_ui_ir_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptFile(path) => {
            write_json_file(diff.current_ui_ir_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptJson => {
            write_json(diff.current_registry_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptFile(path) => {
            write_json_file(diff.current_registry_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptJson => {
            write_json(diff.current_assessment_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptFile(path) => {
            write_json_file(diff.current_assessment_receipt(), path)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(output, diff, writer))]
fn write_current_alias_output(
    output: &BelvedereCliOutput,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match output {
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson => {
            write_json(diff.current_landscape_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(path) => {
            write_json_file(diff.current_landscape_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffLandscapeJson => {
            write_json(diff.current_landscape(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffLandscapeFile(path) => {
            write_json_file(diff.current_landscape(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffViewModelJson => {
            write_json(diff.diff_view_model(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffViewModelFile(path) => {
            write_json_file(diff.diff_view_model(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffFilterFacetsJson => {
            write_json(diff.current_view_model().filter_facets(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffFilterFacetsFile(path) => {
            write_json_file(diff.current_view_model().filter_facets(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffSourceNavigationJson => {
            write_json(diff.current_view_model().source_navigation(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffSourceNavigationFile(path) => {
            write_json_file(diff.current_view_model().source_navigation(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffProofChainsJson => {
            write_json(diff.current_view_model().proof_chains(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffProofChainsFile(path) => {
            write_json_file(diff.current_view_model().proof_chains(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffReviewWorkflowJson => {
            write_json(diff.diff_view_model().items(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffReviewWorkflowFile(path) => {
            write_json_file(diff.diff_view_model().items(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffViewModelReceiptJson => {
            write_json(diff.diff_view_model_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(path) => {
            write_json_file(diff.diff_view_model_receipt(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffUiIrJson => write_json(diff.diff_projection(), writer)?,
        BelvedereCliOutput::LandscapeDiffUiIrFile(path) => {
            write_json_file(diff.diff_projection(), path)?;
        }
        BelvedereCliOutput::LandscapeDiffUiIrReceiptJson => {
            write_json(diff.diff_ui_ir_receipt(), writer)?;
        }
        BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(path) => {
            write_json_file(diff.diff_ui_ir_receipt(), path)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(value, path))]
fn write_json_file(value: &impl Serialize, path: impl AsRef<Path>) -> Result<(), BelvedereError> {
    let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
    write_json(value, file)
}
