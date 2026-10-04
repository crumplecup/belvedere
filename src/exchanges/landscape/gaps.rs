//! Conservative gap modeling for landscape exchanges.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::contracts::{
    LandscapeAssessment, LandscapeClaim, LandscapeGap, LandscapeGapKind, Recommendation,
    ResolutionPath,
};

#[instrument(level = "debug", skip(gaps))]
pub(super) fn gap_kind_counts(gaps: &[LandscapeGap]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for gap in gaps {
        *counts
            .entry(format!("landscape_gap_kind_{:?}", gap.kind()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(claims, assessments))]
pub(super) fn model_gaps(
    claims: &BTreeMap<String, LandscapeClaim>,
    assessments: &[LandscapeAssessment],
) -> Vec<LandscapeGap> {
    let assessments_by_id: BTreeMap<&str, &LandscapeAssessment> = assessments
        .iter()
        .map(|assessment| (assessment.id().as_str(), assessment))
        .collect();

    let mut gaps = Vec::new();
    for claim in claims.values() {
        if claim.proofs().is_empty() && claim.kani_proofs().is_empty() {
            gaps.push(LandscapeGap::new(
                claim.id(),
                LandscapeGapKind::MissingProof,
                "claim has no modeled proof coverage",
            ));
        }

        if (!claim.proofs().is_empty() || !claim.kani_proofs().is_empty())
            && claim.assessment_ids().is_empty()
        {
            gaps.push(LandscapeGap::new(
                claim.id(),
                LandscapeGapKind::MissingAssessment,
                "claim has proof coverage but no assessment",
            ));
        }

        if let Some(assessment) = latest_assessment(claim, &assessments_by_id) {
            push_recommendation_gap(claim.id(), assessment, &mut gaps);
        }

        for witness in claim.witness_surfaces() {
            if witness.trusted() > 0 || witness.opaque() > 0 {
                gaps.push(LandscapeGap::new(
                    claim.id(),
                    LandscapeGapKind::TrustedOrOpaqueWitnessSupport,
                    format!(
                        "witness support includes {} trusted and {} opaque obligations",
                        witness.trusted(),
                        witness.opaque()
                    ),
                ));
            }
        }
    }
    gaps
}

#[instrument(level = "debug", skip(claim, assessments_by_id))]
fn latest_assessment<'a>(
    claim: &LandscapeClaim,
    assessments_by_id: &BTreeMap<&str, &'a LandscapeAssessment>,
) -> Option<&'a LandscapeAssessment> {
    claim
        .assessment_ids()
        .iter()
        .filter_map(|id| assessments_by_id.get(id.as_str()).copied())
        .max_by_key(|assessment| assessment.timestamp())
}

#[instrument(level = "debug", skip(assessment, gaps))]
fn push_recommendation_gap(
    claim_id: &str,
    assessment: &LandscapeAssessment,
    gaps: &mut Vec<LandscapeGap>,
) {
    match assessment.recommendation() {
        Recommendation::Accept | Recommendation::Strengthen => {}
        Recommendation::Replace => gaps.push(LandscapeGap::new(
            claim_id,
            LandscapeGapKind::ReplacementRecommended,
            resolution_detail("latest assessment recommends replacement", assessment),
        )),
        Recommendation::Retire => gaps.push(LandscapeGap::new(
            claim_id,
            LandscapeGapKind::RetirementRecommended,
            resolution_detail("latest assessment recommends retirement", assessment),
        )),
    }
}

#[instrument(level = "debug", skip(assessment))]
fn resolution_detail(prefix: &str, assessment: &LandscapeAssessment) -> String {
    match assessment.resolution_path() {
        Some(ResolutionPath::KeepCurrentProof) => {
            format!("{prefix}: keep current proof")
        }
        Some(ResolutionPath::StrengthenCurrentProof) => {
            format!("{prefix}: strengthen current proof")
        }
        Some(ResolutionPath::ReplaceWithProofSpecificModel) => {
            format!("{prefix}: replace with proof-specific model")
        }
        Some(ResolutionPath::ReplaceWithAccommodationModel) => {
            format!("{prefix}: replace with accommodation model")
        }
        Some(ResolutionPath::RetireClaim) => format!("{prefix}: retire claim"),
        None => prefix.to_owned(),
    }
}
