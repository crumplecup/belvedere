//! Gap queue projections.

use tracing::instrument;

use crate::{
    contracts::{GapItem, GapItemBuilder, GapSeverity, LandscapeGap, LandscapeGapKind, TraceRef},
    error::BelvedereConstructionError,
    exchanges::source_records::subject_source_record_ids_for_gap,
};

#[instrument(level = "debug", skip(gaps, source_record_ids))]
pub(super) fn project_gaps(
    gaps: &[LandscapeGap],
    source_record_ids: &[String],
) -> Result<Vec<GapItem>, BelvedereConstructionError> {
    gaps.iter()
        .map(|gap| {
            let severity = gap_severity(gap.kind());
            GapItemBuilder::default()
                .gap_id(stable_gap_id(gap))
                .claim_id(gap.claim_id())
                .kind(gap.kind())
                .severity(severity)
                .queue_lane(gap_queue_lane(gap.kind()))
                .action(gap_action(gap.kind()))
                .detail(gap.detail())
                .high_priority(severity.high_priority())
                .trace(TraceRef::new(
                    gap.claim_id(),
                    subject_source_record_ids_for_gap(gap, source_record_ids),
                ))
                .build()
                .map_err(|error| BelvedereConstructionError::new("GapItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(gap))]
fn stable_gap_id(gap: &LandscapeGap) -> String {
    let origin = gap
        .origin_key()
        .map_or_else(String::new, |key| format!(":{key}"));
    format!(
        "view_model.gap:{}:{}{origin}",
        gap.claim_id(),
        gap_kind_id(gap.kind())
    )
}

#[instrument(level = "debug", skip(kind))]
fn gap_kind_id(kind: LandscapeGapKind) -> &'static str {
    match kind {
        LandscapeGapKind::MissingProof => "missing_proof",
        LandscapeGapKind::MissingAssessment => "missing_assessment",
        LandscapeGapKind::ReplacementRecommended => "replacement_recommended",
        LandscapeGapKind::RetirementRecommended => "retirement_recommended",
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => "trusted_or_opaque_witness_support",
        LandscapeGapKind::ProofExecutionUnresolved => "proof_execution_unresolved",
    }
}

#[instrument(level = "debug", skip(kind))]
fn gap_severity(kind: LandscapeGapKind) -> GapSeverity {
    match kind {
        LandscapeGapKind::MissingProof
        | LandscapeGapKind::ProofExecutionUnresolved
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
        LandscapeGapKind::MissingProof | LandscapeGapKind::ProofExecutionUnresolved => {
            "proof_coverage"
        }
        LandscapeGapKind::ReplacementRecommended => "replacement",
        LandscapeGapKind::RetirementRecommended => "retirement",
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => "witness_review",
    }
}

#[instrument(level = "debug", skip(kind))]
fn gap_action(kind: LandscapeGapKind) -> &'static str {
    match kind {
        LandscapeGapKind::MissingProof => "establish modeled proof coverage",
        LandscapeGapKind::ProofExecutionUnresolved => "resolve proof execution status",
        LandscapeGapKind::MissingAssessment => "assess proof coverage",
        LandscapeGapKind::ReplacementRecommended => "replace proof or model",
        LandscapeGapKind::RetirementRecommended => "retire claim or dependency",
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => {
            "review trusted or opaque witness support"
        }
    }
}
