//! Run command output option parsing.

use tracing::instrument;

use crate::BelvedereError;
use crate::cli::{BelvedereCliArtifact, BelvedereCliOutputSink};

use super::BelvedereCli;

impl BelvedereCli {
    #[instrument(level = "debug", skip(artifact, sink))]
    pub(in crate::cli::command) fn parse_run_output_arg(
        arguments: &[String],
        index: usize,
        artifact: &mut Option<BelvedereCliArtifact>,
        sink: &mut Option<BelvedereCliOutputSink>,
    ) -> Result<Option<usize>, BelvedereError> {
        match arguments[index].as_str() {
            "--stdio" => {
                Self::set_sink(sink, BelvedereCliOutputSink::Stdio)?;
                Ok(Some(0))
            }
            "--file" => {
                Self::set_sink(
                    sink,
                    BelvedereCliOutputSink::File(Self::next_path(arguments, index)?),
                )?;
                Ok(Some(1))
            }
            flag => {
                let Some(next) = run_artifact(flag) else {
                    return Ok(None);
                };
                Self::set_artifact(artifact, next)?;
                Ok(Some(0))
            }
        }
    }
}

#[instrument(level = "trace")]
fn run_artifact(flag: &str) -> Option<BelvedereCliArtifact> {
    match flag {
        "--certificate" => Some(BelvedereCliArtifact::RunCertificate),
        "--registry-ingestion-certificate" => {
            Some(BelvedereCliArtifact::RunRegistryIngestionCertificate)
        }
        "--assessment-ingestion-certificate" => {
            Some(BelvedereCliArtifact::RunAssessmentIngestionCertificate)
        }
        "--landscape-modeling-certificate" => {
            Some(BelvedereCliArtifact::RunLandscapeModelingCertificate)
        }
        "--auditor-navigation-certificate" => {
            Some(BelvedereCliArtifact::RunAuditorNavigationCertificate)
        }
        "--ui-ir-projection-certificate" => {
            Some(BelvedereCliArtifact::RunUiIrProjectionCertificate)
        }
        "--manifest" => Some(BelvedereCliArtifact::RunManifest),
        "--registry-receipt" => Some(BelvedereCliArtifact::RunRegistryReceipt),
        "--assessment-receipt" => Some(BelvedereCliArtifact::RunAssessmentReceipt),
        "--landscape" => Some(BelvedereCliArtifact::RunLandscape),
        "--landscape-receipt" => Some(BelvedereCliArtifact::RunLandscapeReceipt),
        "--view-model" => Some(BelvedereCliArtifact::RunViewModel),
        "--backend-coverage" => Some(BelvedereCliArtifact::RunBackendCoverage),
        "--claim-explorer" => Some(BelvedereCliArtifact::RunClaimExplorer),
        "--claim-details" => Some(BelvedereCliArtifact::RunClaimDetails),
        "--filter-facets" => Some(BelvedereCliArtifact::RunFilterFacets),
        "--source-navigation" => Some(BelvedereCliArtifact::RunSourceNavigation),
        "--proof-chains" => Some(BelvedereCliArtifact::RunProofChains),
        "--review-workflow" => Some(BelvedereCliArtifact::RunReviewWorkflow),
        "--assessment-history" => Some(BelvedereCliArtifact::RunAssessmentHistory),
        "--assessment-queue" => Some(BelvedereCliArtifact::RunAssessmentQueue),
        "--assessment-work" => Some(BelvedereCliArtifact::RunAssessmentWork),
        "--witness-surface" => Some(BelvedereCliArtifact::RunWitnessSurface),
        "--provenance-roots" => Some(BelvedereCliArtifact::RunProvenanceRoots),
        "--gaps" => Some(BelvedereCliArtifact::RunGaps),
        "--view-model-receipt" => Some(BelvedereCliArtifact::RunViewModelReceipt),
        "--ui-ir" => Some(BelvedereCliArtifact::RunUiIr),
        "--ui-ir-sections" => Some(BelvedereCliArtifact::RunUiIrSections),
        "--ui-ir-trace-widgets" => Some(BelvedereCliArtifact::RunUiIrTraceWidgets),
        "--ui-ir-viewports" => Some(BelvedereCliArtifact::RunUiIrViewports),
        "--ui-ir-minted-standards" => Some(BelvedereCliArtifact::RunUiIrMintedStandards),
        "--ui-ir-receipt" => Some(BelvedereCliArtifact::RunUiIrReceipt),
        _ => None,
    }
}
