//! Gap queue projections.

use tracing::instrument;

use crate::{
    contracts::{GapItem, GapItemBuilder, GapSeverity, LandscapeGap, LandscapeGapKind},
    error::BelvedereConstructionError,
};

use super::trace;

#[instrument(level = "debug", skip(gaps))]
pub(super) fn project_gaps(
    gaps: &[LandscapeGap],
    source_record_ids: &[String],
) -> Result<Vec<GapItem>, BelvedereConstructionError> {
    gaps.iter()
        .map(|gap| {
            let severity = gap_severity(gap.kind());
            GapItemBuilder::default()
                .claim_id(gap.claim_id())
                .kind(gap.kind())
                .severity(severity)
                .queue_lane(gap_queue_lane(gap.kind()))
                .action(gap_action(gap.kind()))
                .detail(gap.detail())
                .high_priority(severity.high_priority())
                .trace(trace(gap.claim_id(), source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("GapItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(kind))]
fn gap_severity(kind: LandscapeGapKind) -> GapSeverity {
    match kind {
        LandscapeGapKind::MissingProof
        | LandscapeGapKind::ReplacementRecommended
        | LandscapeGapKind::RetirementRecommended => GapSeverity::Blocking,
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => GapSeverity::Elevated,
        LandscapeGapKind::MissingAssessment => GapSeverity::Review,
    }
}

#[instrument(level = "debug", skip(kind))]
pub(super) fn gap_queue_lane(kind: LandscapeGapKind) -> &'static str {
    match kind {
        LandscapeGapKind::MissingAssessment => "assessment",
        LandscapeGapKind::MissingProof => "proof_coverage",
        LandscapeGapKind::ReplacementRecommended => "replacement",
        LandscapeGapKind::RetirementRecommended => "retirement",
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => "witness_review",
    }
}

#[instrument(level = "debug", skip(kind))]
fn gap_action(kind: LandscapeGapKind) -> &'static str {
    match kind {
        LandscapeGapKind::MissingProof => "establish modeled proof coverage",
        LandscapeGapKind::MissingAssessment => "assess proof coverage",
        LandscapeGapKind::ReplacementRecommended => "replace proof or model",
        LandscapeGapKind::RetirementRecommended => "retire claim or dependency",
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => {
            "review trusted or opaque witness support"
        }
    }
}
