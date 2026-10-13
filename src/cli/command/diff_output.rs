//! Diff command output option parsing.

use tracing::instrument;

use crate::BelvedereError;
use crate::cli::{BelvedereCliArtifact, BelvedereCliOutputSink};

use super::BelvedereCli;

impl BelvedereCli {
    #[instrument(level = "debug", skip(artifact, sink))]
    pub(in crate::cli::command) fn parse_diff_output_arg(
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
                let Some(next) = diff_artifact(flag) else {
                    return Ok(None);
                };
                Self::set_artifact(artifact, next)?;
                Ok(Some(0))
            }
        }
    }
}

#[instrument(level = "trace")]
fn diff_artifact(flag: &str) -> Option<BelvedereCliArtifact> {
    match flag {
        "--certificate" => Some(BelvedereCliArtifact::LandscapeDiffCertificate),
        "--diff-review-certificate" => Some(BelvedereCliArtifact::LandscapeDiffReviewCertificate),
        "--diff-review-ui-ir-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffReviewUiIrCertificate)
        }
        "--run-certificate" => Some(BelvedereCliArtifact::LandscapeDiffRunCertificate),
        "--manifest" => Some(BelvedereCliArtifact::LandscapeDiffManifest),
        "--diff" => Some(BelvedereCliArtifact::LandscapeDiff),
        "--diff-receipt" => Some(BelvedereCliArtifact::LandscapeDiffReceipt),
        "--previous-registry-ingestion-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousRegistryIngestionCertificate)
        }
        "--previous-assessment-ingestion-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentIngestionCertificate)
        }
        "--previous-landscape-modeling-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousLandscapeModelingCertificate)
        }
        "--previous-auditor-navigation-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousAuditorNavigationCertificate)
        }
        "--previous-ui-ir-projection-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrProjectionCertificate)
        }
        "--previous-registry-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousRegistryReceipt)
        }
        "--previous-assessment-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentReceipt)
        }
        "--previous-landscape" => Some(BelvedereCliArtifact::LandscapeDiffPreviousLandscape),
        "--previous-landscape-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousLandscapeReceipt)
        }
        "--previous-view-model" => Some(BelvedereCliArtifact::LandscapeDiffPreviousViewModel),
        "--previous-overview" => Some(BelvedereCliArtifact::LandscapeDiffPreviousOverview),
        "--previous-backend-coverage" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousBackendCoverage)
        }
        "--previous-claim-explorer" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousClaimExplorer)
        }
        "--previous-claim-details" => Some(BelvedereCliArtifact::LandscapeDiffPreviousClaimDetails),
        "--previous-filter-facets" => Some(BelvedereCliArtifact::LandscapeDiffPreviousFilterFacets),
        "--previous-source-navigation" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousSourceNavigation)
        }
        "--previous-proof-chains" => Some(BelvedereCliArtifact::LandscapeDiffPreviousProofChains),
        "--previous-review-workflow" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousReviewWorkflow)
        }
        "--previous-assessment-history" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentHistory)
        }
        "--previous-assessment-queue" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentQueue)
        }
        "--previous-assessment-work" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentWork)
        }
        "--previous-witness-surface" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousWitnessSurface)
        }
        "--previous-provenance-roots" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousProvenanceRoots)
        }
        "--previous-gaps" => Some(BelvedereCliArtifact::LandscapeDiffPreviousGaps),
        "--previous-view-model-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousViewModelReceipt)
        }
        "--previous-ui-ir" => Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIr),
        "--previous-ui-ir-sections" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrSections)
        }
        "--previous-ui-ir-trace-widgets" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrTraceWidgets)
        }
        "--previous-ui-ir-viewports" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrViewports)
        }
        "--previous-ui-ir-minted-standards" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrMintedStandards)
        }
        "--previous-ui-ir-receipt" => Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrReceipt),
        "--current-registry-ingestion-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentRegistryIngestionCertificate)
        }
        "--current-assessment-ingestion-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentIngestionCertificate)
        }
        "--current-landscape-modeling-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentLandscapeModelingCertificate)
        }
        "--current-auditor-navigation-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentAuditorNavigationCertificate)
        }
        "--current-ui-ir-projection-certificate" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrProjectionCertificate)
        }
        "--current-registry-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentRegistryReceipt)
        }
        "--current-assessment-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentReceipt)
        }
        "--current-landscape" => Some(BelvedereCliArtifact::LandscapeDiffCurrentLandscape),
        "--current-landscape-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentLandscapeReceipt)
        }
        "--current-view-model" => Some(BelvedereCliArtifact::LandscapeDiffCurrentViewModel),
        "--current-overview" => Some(BelvedereCliArtifact::LandscapeDiffCurrentOverview),
        "--current-backend-coverage" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentBackendCoverage)
        }
        "--current-claim-explorer" => Some(BelvedereCliArtifact::LandscapeDiffCurrentClaimExplorer),
        "--current-claim-details" => Some(BelvedereCliArtifact::LandscapeDiffCurrentClaimDetails),
        "--current-filter-facets" => Some(BelvedereCliArtifact::LandscapeDiffCurrentFilterFacets),
        "--current-source-navigation" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentSourceNavigation)
        }
        "--current-proof-chains" => Some(BelvedereCliArtifact::LandscapeDiffCurrentProofChains),
        "--current-review-workflow" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentReviewWorkflow)
        }
        "--current-assessment-history" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentHistory)
        }
        "--current-assessment-queue" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentQueue)
        }
        "--current-assessment-work" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentWork)
        }
        "--current-witness-surface" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentWitnessSurface)
        }
        "--current-provenance-roots" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentProvenanceRoots)
        }
        "--current-gaps" => Some(BelvedereCliArtifact::LandscapeDiffCurrentGaps),
        "--current-view-model-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentViewModelReceipt)
        }
        "--current-ui-ir" => Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIr),
        "--current-ui-ir-sections" => Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrSections),
        "--current-ui-ir-trace-widgets" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrTraceWidgets)
        }
        "--current-ui-ir-viewports" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrViewports)
        }
        "--current-ui-ir-minted-standards" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrMintedStandards)
        }
        "--current-ui-ir-receipt" => Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrReceipt),
        "--landscape" => Some(BelvedereCliArtifact::LandscapeDiffLandscape),
        "--landscape-receipt" => Some(BelvedereCliArtifact::LandscapeDiffLandscapeReceipt),
        "--view-model" => Some(BelvedereCliArtifact::LandscapeDiffViewModel),
        "--overview" => Some(BelvedereCliArtifact::LandscapeDiffOverview),
        "--backend-coverage" => Some(BelvedereCliArtifact::LandscapeDiffBackendCoverage),
        "--claim-explorer" => Some(BelvedereCliArtifact::LandscapeDiffClaimExplorer),
        "--claim-details" => Some(BelvedereCliArtifact::LandscapeDiffClaimDetails),
        "--filter-facets" => Some(BelvedereCliArtifact::LandscapeDiffFilterFacets),
        "--source-navigation" => Some(BelvedereCliArtifact::LandscapeDiffSourceNavigation),
        "--proof-chains" => Some(BelvedereCliArtifact::LandscapeDiffProofChains),
        "--review-workflow" => Some(BelvedereCliArtifact::LandscapeDiffReviewWorkflow),
        "--assessment-history" => Some(BelvedereCliArtifact::LandscapeDiffAssessmentHistory),
        "--assessment-queue" => Some(BelvedereCliArtifact::LandscapeDiffAssessmentQueue),
        "--assessment-work" => Some(BelvedereCliArtifact::LandscapeDiffAssessmentWork),
        "--witness-surface" => Some(BelvedereCliArtifact::LandscapeDiffWitnessSurface),
        "--provenance-roots" => Some(BelvedereCliArtifact::LandscapeDiffProvenanceRoots),
        "--gaps" => Some(BelvedereCliArtifact::LandscapeDiffGaps),
        "--view-model-receipt" => Some(BelvedereCliArtifact::LandscapeDiffViewModelReceipt),
        "--ui-ir" => Some(BelvedereCliArtifact::LandscapeDiffUiIr),
        "--ui-ir-sections" => Some(BelvedereCliArtifact::LandscapeDiffUiIrSections),
        "--ui-ir-trace-widgets" => Some(BelvedereCliArtifact::LandscapeDiffUiIrTraceWidgets),
        "--ui-ir-viewports" => Some(BelvedereCliArtifact::LandscapeDiffUiIrViewports),
        "--ui-ir-minted-standards" => Some(BelvedereCliArtifact::LandscapeDiffUiIrMintedStandards),
        "--ui-ir-receipt" => Some(BelvedereCliArtifact::LandscapeDiffUiIrReceipt),
        _ => None,
    }
}
