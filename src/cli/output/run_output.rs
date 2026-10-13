//! Normal run CLI output writers.

use std::io::Write;

use tracing::instrument;

use crate::{BelvedereError, BelvedereRunReport};

use super::{BelvedereCliArtifact, BelvedereCliOutputSink, write_json_to_sink};
use crate::cli::BelvedereCliUsageError;

#[instrument(level = "info", skip(artifact, sink, run, writer))]
pub(super) fn write_run_output(
    artifact: BelvedereCliArtifact,
    sink: &BelvedereCliOutputSink,
    run: &BelvedereRunReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match artifact {
        BelvedereCliArtifact::RunCertificate => write_json_to_sink(run.certificate(), sink, writer),
        BelvedereCliArtifact::RunRegistryIngestionCertificate => {
            write_run_stage_certificate(run, 0, sink, writer)
        }
        BelvedereCliArtifact::RunAssessmentIngestionCertificate => {
            write_run_stage_certificate(run, 1, sink, writer)
        }
        BelvedereCliArtifact::RunLandscapeModelingCertificate => {
            write_run_stage_certificate(run, 2, sink, writer)
        }
        BelvedereCliArtifact::RunAuditorNavigationCertificate => {
            write_run_stage_certificate(run, 3, sink, writer)
        }
        BelvedereCliArtifact::RunUiIrProjectionCertificate => {
            write_run_stage_certificate(run, 4, sink, writer)
        }
        BelvedereCliArtifact::RunManifest => write_json_to_sink(run.manifest(), sink, writer),
        BelvedereCliArtifact::RunRegistryReceipt => {
            write_json_to_sink(run.registry_receipt(), sink, writer)
        }
        BelvedereCliArtifact::RunAssessmentReceipt => {
            write_json_to_sink(run.assessment_receipt(), sink, writer)
        }
        BelvedereCliArtifact::RunLandscape => write_json_to_sink(run.landscape(), sink, writer),
        BelvedereCliArtifact::RunLandscapeReceipt => {
            write_json_to_sink(run.landscape_receipt(), sink, writer)
        }
        BelvedereCliArtifact::RunViewModel => write_json_to_sink(run.view_model(), sink, writer),
        BelvedereCliArtifact::RunOverview => {
            write_json_to_sink(run.view_model().overview(), sink, writer)
        }
        BelvedereCliArtifact::RunBackendCoverage => {
            write_json_to_sink(run.view_model().backend_coverage(), sink, writer)
        }
        BelvedereCliArtifact::RunClaimExplorer => {
            write_json_to_sink(run.view_model().claim_explorer(), sink, writer)
        }
        BelvedereCliArtifact::RunClaimDetails => {
            write_json_to_sink(run.view_model().claim_details(), sink, writer)
        }
        BelvedereCliArtifact::RunFilterFacets => {
            write_json_to_sink(run.view_model().filter_facets(), sink, writer)
        }
        BelvedereCliArtifact::RunSourceNavigation => {
            write_json_to_sink(run.view_model().source_navigation(), sink, writer)
        }
        BelvedereCliArtifact::RunProofChains => {
            write_json_to_sink(run.view_model().proof_chains(), sink, writer)
        }
        BelvedereCliArtifact::RunReviewWorkflow => {
            write_json_to_sink(run.view_model().review_workflow(), sink, writer)
        }
        BelvedereCliArtifact::RunAssessmentHistory => {
            write_json_to_sink(run.view_model().assessment_history(), sink, writer)
        }
        BelvedereCliArtifact::RunAssessmentQueue => {
            write_json_to_sink(run.view_model().assessment_queue(), sink, writer)
        }
        BelvedereCliArtifact::RunAssessmentWork => {
            write_json_to_sink(run.view_model().assessment_work(), sink, writer)
        }
        BelvedereCliArtifact::RunWitnessSurface => {
            write_json_to_sink(run.view_model().witness_surface(), sink, writer)
        }
        BelvedereCliArtifact::RunProvenanceRoots => {
            write_json_to_sink(run.view_model().provenance_roots(), sink, writer)
        }
        BelvedereCliArtifact::RunGaps => write_json_to_sink(run.view_model().gaps(), sink, writer),
        BelvedereCliArtifact::RunViewModelReceipt => {
            write_json_to_sink(run.view_model_receipt(), sink, writer)
        }
        BelvedereCliArtifact::RunUiIr => write_json_to_sink(run.projection(), sink, writer),
        BelvedereCliArtifact::RunUiIrSections => {
            write_json_to_sink(run.projection().sections(), sink, writer)
        }
        BelvedereCliArtifact::RunUiIrTraceWidgets => {
            write_json_to_sink(run.projection().trace_widgets(), sink, writer)
        }
        BelvedereCliArtifact::RunUiIrViewports => {
            write_json_to_sink(run.projection().viewport_verifications(), sink, writer)
        }
        BelvedereCliArtifact::RunUiIrMintedStandards => {
            write_json_to_sink(run.projection().minted_standards(), sink, writer)
        }
        BelvedereCliArtifact::RunUiIrReceipt => {
            write_json_to_sink(run.ui_ir_receipt(), sink, writer)
        }
        _ => Err(BelvedereCliUsageError::new().into()),
    }
}

#[instrument(level = "debug", skip(run, sink, writer))]
fn write_run_stage_certificate(
    run: &BelvedereRunReport,
    source_stage_index: usize,
    sink: &BelvedereCliOutputSink,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    let Some(certificate) = run
        .certificate()
        .stage_certificates()
        .get(source_stage_index)
    else {
        return Err(BelvedereCliUsageError::new().into());
    };

    write_json_to_sink(certificate, sink, writer)
}
