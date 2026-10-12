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
        "--run-certificate" => Some(BelvedereCliArtifact::LandscapeDiffRunCertificate),
        "--manifest" => Some(BelvedereCliArtifact::LandscapeDiffManifest),
        "--diff" => Some(BelvedereCliArtifact::LandscapeDiff),
        "--diff-receipt" => Some(BelvedereCliArtifact::LandscapeDiffReceipt),
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
        "--previous-claim-details" => Some(BelvedereCliArtifact::LandscapeDiffPreviousClaimDetails),
        "--previous-filter-facets" => Some(BelvedereCliArtifact::LandscapeDiffPreviousFilterFacets),
        "--previous-source-navigation" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousSourceNavigation)
        }
        "--previous-proof-chains" => Some(BelvedereCliArtifact::LandscapeDiffPreviousProofChains),
        "--previous-view-model-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffPreviousViewModelReceipt)
        }
        "--previous-ui-ir" => Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIr),
        "--previous-ui-ir-receipt" => Some(BelvedereCliArtifact::LandscapeDiffPreviousUiIrReceipt),
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
        "--current-claim-details" => Some(BelvedereCliArtifact::LandscapeDiffCurrentClaimDetails),
        "--current-filter-facets" => Some(BelvedereCliArtifact::LandscapeDiffCurrentFilterFacets),
        "--current-source-navigation" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentSourceNavigation)
        }
        "--current-proof-chains" => Some(BelvedereCliArtifact::LandscapeDiffCurrentProofChains),
        "--current-view-model-receipt" => {
            Some(BelvedereCliArtifact::LandscapeDiffCurrentViewModelReceipt)
        }
        "--current-ui-ir" => Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIr),
        "--current-ui-ir-receipt" => Some(BelvedereCliArtifact::LandscapeDiffCurrentUiIrReceipt),
        "--landscape" => Some(BelvedereCliArtifact::LandscapeDiffLandscape),
        "--landscape-receipt" => Some(BelvedereCliArtifact::LandscapeDiffLandscapeReceipt),
        "--view-model" => Some(BelvedereCliArtifact::LandscapeDiffViewModel),
        "--claim-details" => Some(BelvedereCliArtifact::LandscapeDiffClaimDetails),
        "--filter-facets" => Some(BelvedereCliArtifact::LandscapeDiffFilterFacets),
        "--source-navigation" => Some(BelvedereCliArtifact::LandscapeDiffSourceNavigation),
        "--proof-chains" => Some(BelvedereCliArtifact::LandscapeDiffProofChains),
        "--review-workflow" => Some(BelvedereCliArtifact::LandscapeDiffReviewWorkflow),
        "--view-model-receipt" => Some(BelvedereCliArtifact::LandscapeDiffViewModelReceipt),
        "--ui-ir" => Some(BelvedereCliArtifact::LandscapeDiffUiIr),
        "--ui-ir-receipt" => Some(BelvedereCliArtifact::LandscapeDiffUiIrReceipt),
        _ => None,
    }
}
