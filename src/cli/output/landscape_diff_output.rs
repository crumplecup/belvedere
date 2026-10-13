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
        BelvedereCliArtifact::LandscapeDiffPreviousRegistryIngestionCertificate => {
            write_landscape_diff_stage_certificate(diff, 0, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousAssessmentIngestionCertificate => {
            write_landscape_diff_stage_certificate(diff, 1, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousLandscapeModelingCertificate => {
            write_landscape_diff_stage_certificate(diff, 2, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousAuditorNavigationCertificate => {
            write_landscape_diff_stage_certificate(diff, 3, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIrProjectionCertificate => {
            write_landscape_diff_stage_certificate(diff, 4, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentRegistryIngestionCertificate => {
            write_landscape_diff_stage_certificate(diff, 5, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentAssessmentIngestionCertificate => {
            write_landscape_diff_stage_certificate(diff, 6, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentLandscapeModelingCertificate => {
            write_landscape_diff_stage_certificate(diff, 7, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentAuditorNavigationCertificate => {
            write_landscape_diff_stage_certificate(diff, 8, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIrProjectionCertificate => {
            write_landscape_diff_stage_certificate(diff, 9, sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffReviewCertificate => {
            write_json_to_sink(diff.diff_view_model_certificate(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffReviewUiIrCertificate => {
            write_json_to_sink(diff.diff_ui_ir_certificate(), sink, writer)?;
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

#[instrument(level = "debug", skip(diff, sink, writer))]
fn write_landscape_diff_stage_certificate(
    diff: &BelvedereLandscapeDiffRunReport,
    source_stage_index: usize,
    sink: &BelvedereCliOutputSink,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    let Some(certificate) = diff
        .certificate()
        .stage_certificates()
        .get(source_stage_index)
    else {
        return Err(BelvedereCliUsageError::new().into());
    };

    write_json_to_sink(certificate, sink, writer)
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
        BelvedereCliArtifact::LandscapeDiffPreviousBackendCoverage => {
            write_json_to_sink(diff.previous_view_model().backend_coverage(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousClaimExplorer => {
            write_json_to_sink(diff.previous_view_model().claim_explorer(), sink, writer)?;
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
        BelvedereCliArtifact::LandscapeDiffPreviousReviewWorkflow => {
            write_json_to_sink(diff.previous_view_model().review_workflow(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousAssessmentHistory => {
            write_json_to_sink(
                diff.previous_view_model().assessment_history(),
                sink,
                writer,
            )?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousAssessmentQueue => {
            write_json_to_sink(diff.previous_view_model().assessment_queue(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousAssessmentWork => {
            write_json_to_sink(diff.previous_view_model().assessment_work(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousWitnessSurface => {
            write_json_to_sink(diff.previous_view_model().witness_surface(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousProvenanceRoots => {
            write_json_to_sink(diff.previous_view_model().provenance_roots(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousGaps => {
            write_json_to_sink(diff.previous_view_model().gaps(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousViewModelReceipt => {
            write_json_to_sink(diff.previous_view_model_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIr => {
            write_json_to_sink(diff.previous_projection(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIrSections => {
            write_json_to_sink(diff.previous_projection().sections(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIrTraceWidgets => {
            write_json_to_sink(diff.previous_projection().trace_widgets(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIrViewports => {
            write_json_to_sink(
                diff.previous_projection().viewport_verifications(),
                sink,
                writer,
            )?;
        }
        BelvedereCliArtifact::LandscapeDiffPreviousUiIrMintedStandards => {
            write_json_to_sink(diff.previous_projection().minted_standards(), sink, writer)?;
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
        BelvedereCliArtifact::LandscapeDiffCurrentBackendCoverage => {
            write_json_to_sink(diff.current_view_model().backend_coverage(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentClaimExplorer => {
            write_json_to_sink(diff.current_view_model().claim_explorer(), sink, writer)?;
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
        BelvedereCliArtifact::LandscapeDiffCurrentReviewWorkflow => {
            write_json_to_sink(diff.current_view_model().review_workflow(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentAssessmentHistory => {
            write_json_to_sink(diff.current_view_model().assessment_history(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentAssessmentQueue => {
            write_json_to_sink(diff.current_view_model().assessment_queue(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentAssessmentWork => {
            write_json_to_sink(diff.current_view_model().assessment_work(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentWitnessSurface => {
            write_json_to_sink(diff.current_view_model().witness_surface(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentProvenanceRoots => {
            write_json_to_sink(diff.current_view_model().provenance_roots(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentGaps => {
            write_json_to_sink(diff.current_view_model().gaps(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentViewModelReceipt => {
            write_json_to_sink(diff.current_view_model_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIr => {
            write_json_to_sink(diff.current_projection(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIrSections => {
            write_json_to_sink(diff.current_projection().sections(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIrTraceWidgets => {
            write_json_to_sink(diff.current_projection().trace_widgets(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIrViewports => {
            write_json_to_sink(
                diff.current_projection().viewport_verifications(),
                sink,
                writer,
            )?;
        }
        BelvedereCliArtifact::LandscapeDiffCurrentUiIrMintedStandards => {
            write_json_to_sink(diff.current_projection().minted_standards(), sink, writer)?;
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
        BelvedereCliArtifact::LandscapeDiffBackendCoverage => {
            write_json_to_sink(diff.current_view_model().backend_coverage(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffClaimExplorer => {
            write_json_to_sink(diff.current_view_model().claim_explorer(), sink, writer)?;
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
        BelvedereCliArtifact::LandscapeDiffAssessmentHistory => {
            write_json_to_sink(diff.current_view_model().assessment_history(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffAssessmentQueue => {
            write_json_to_sink(diff.current_view_model().assessment_queue(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffAssessmentWork => {
            write_json_to_sink(diff.current_view_model().assessment_work(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffWitnessSurface => {
            write_json_to_sink(diff.current_view_model().witness_surface(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffProvenanceRoots => {
            write_json_to_sink(diff.current_view_model().provenance_roots(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffGaps => {
            write_json_to_sink(diff.current_view_model().gaps(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffViewModelReceipt => {
            write_json_to_sink(diff.diff_view_model_receipt(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIr => {
            write_json_to_sink(diff.diff_projection(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIrSections => {
            write_json_to_sink(diff.diff_projection().sections(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIrTraceWidgets => {
            write_json_to_sink(diff.diff_projection().trace_widgets(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIrViewports => {
            write_json_to_sink(
                diff.diff_projection().viewport_verifications(),
                sink,
                writer,
            )?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIrMintedStandards => {
            write_json_to_sink(diff.diff_projection().minted_standards(), sink, writer)?;
        }
        BelvedereCliArtifact::LandscapeDiffUiIrReceipt => {
            write_json_to_sink(diff.diff_ui_ir_receipt(), sink, writer)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
