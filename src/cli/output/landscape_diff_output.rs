//! Landscape diff CLI output writers.

use std::io::Write;

use tracing::instrument;

use crate::{BelvedereError, BelvedereLandscapeDiffRunReport};

use super::{BelvedereCliArtifact, BelvedereCliOutputSink, write_json_to_sink};
use crate::cli::BelvedereCliUsageError;

#[instrument(level = "info", skip(artifact, sink, diff, writer))]
pub(super) fn write_landscape_diff_output(
    artifact: BelvedereCliArtifact,
    sink: &BelvedereCliOutputSink,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    let mut writer = writer;
    if write_landscape_diff_certificate_output(artifact, sink, diff, &mut writer)? {
        return Ok(());
    }
    if write_previous_snapshot_output(artifact, sink, diff, &mut writer)? {
        return Ok(());
    }
    if write_current_snapshot_output(artifact, sink, diff, &mut writer)? {
        return Ok(());
    }
    if write_current_alias_output(artifact, sink, diff, &mut writer)? {
        return Ok(());
    }
    Err(BelvedereCliUsageError::new().into())
}

#[instrument(level = "info", skip(artifact, sink, diff, writer))]
fn write_landscape_diff_certificate_output(
    artifact: BelvedereCliArtifact,
    sink: &BelvedereCliOutputSink,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match artifact {
        BelvedereCliArtifact::LandscapeDiffCertificate => {
            write_json_to_sink(diff.diff_certificate(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffRunCertificate => {
            write_json_to_sink(diff.certificate(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffManifest => {
            write_json_to_sink(diff.manifest(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiff => {
            write_json_to_sink(diff.diff(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffReceipt => {
            write_json_to_sink(diff.diff_receipt(), sink, writer)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(artifact, sink, diff, writer))]
fn write_previous_snapshot_output(
    artifact: BelvedereCliArtifact,
    sink: &BelvedereCliOutputSink,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match artifact {
        BelvedereCliArtifact::LandscapeDiffPreviousLandscape => {
            write_json_to_sink(diff.previous_landscape(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousLandscapeReceipt => {
            write_json_to_sink(diff.previous_landscape_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousViewModel => {
            write_json_to_sink(diff.previous_view_model(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousClaimDetails => {
            write_json_to_sink(diff.previous_view_model().claim_details(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousFilterFacets => {
            write_json_to_sink(diff.previous_view_model().filter_facets(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousSourceNavigation => {
            write_json_to_sink(diff.previous_view_model().source_navigation(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousProofChains => {
            write_json_to_sink(diff.previous_view_model().proof_chains(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousViewModelReceipt => {
            write_json_to_sink(diff.previous_view_model_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIr => {
            write_json_to_sink(diff.previous_projection(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIrReceipt => {
            write_json_to_sink(diff.previous_ui_ir_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousRegistryReceipt => {
            write_json_to_sink(diff.previous_registry_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousAssessmentReceipt => {
            write_json_to_sink(diff.previous_assessment_receipt(), sink, writer)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(artifact, sink, diff, writer))]
fn write_current_snapshot_output(
    artifact: BelvedereCliArtifact,
    sink: &BelvedereCliOutputSink,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match artifact {
        BelvedereCliArtifact::LandscapeDiffCurrentLandscapeReceipt => {
            write_json_to_sink(diff.current_landscape_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentLandscape => {
            write_json_to_sink(diff.current_landscape(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentViewModel => {
            write_json_to_sink(diff.current_view_model(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentClaimDetails => {
            write_json_to_sink(diff.current_view_model().claim_details(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentFilterFacets => {
            write_json_to_sink(diff.current_view_model().filter_facets(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentSourceNavigation => {
            write_json_to_sink(diff.current_view_model().source_navigation(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentProofChains => {
            write_json_to_sink(diff.current_view_model().proof_chains(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentViewModelReceipt => {
            write_json_to_sink(diff.current_view_model_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIr => {
            write_json_to_sink(diff.current_projection(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIrReceipt => {
            write_json_to_sink(diff.current_ui_ir_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentRegistryReceipt => {
            write_json_to_sink(diff.current_registry_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentAssessmentReceipt => {
            write_json_to_sink(diff.current_assessment_receipt(), sink, writer)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[instrument(level = "info", skip(artifact, sink, diff, writer))]
fn write_current_alias_output(
    artifact: BelvedereCliArtifact,
    sink: &BelvedereCliOutputSink,
    diff: &BelvedereLandscapeDiffRunReport,
    writer: impl Write,
) -> Result<bool, BelvedereError> {
    match artifact {
        BelvedereCliArtifact::LandscapeDiffLandscapeReceipt => {
            write_json_to_sink(diff.current_landscape_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffLandscape => {
            write_json_to_sink(diff.current_landscape(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffViewModel => {
            write_json_to_sink(diff.diff_view_model(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffClaimDetails => {
            write_json_to_sink(diff.current_view_model().claim_details(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffFilterFacets => {
            write_json_to_sink(diff.current_view_model().filter_facets(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffSourceNavigation => {
            write_json_to_sink(diff.current_view_model().source_navigation(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffProofChains => {
            write_json_to_sink(diff.current_view_model().proof_chains(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffReviewWorkflow => {
            write_json_to_sink(diff.diff_view_model().items(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffViewModelReceipt => {
            write_json_to_sink(diff.diff_view_model_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIr => {
            write_json_to_sink(diff.diff_projection(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIrReceipt => {
            write_json_to_sink(diff.diff_ui_ir_receipt(), sink, writer)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
