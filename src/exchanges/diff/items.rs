//! Diff item construction helpers.

use tracing::instrument;

use crate::contracts::{
    LandscapeAssessment, LandscapeClaim, LandscapeDiffChangeKind, LandscapeDiffItem, LandscapeGap,
    TraceRef,
};

#[instrument(level = "debug", skip(items, change_kind))]
pub(super) fn push_claim_item(
    items: &mut Vec<LandscapeDiffItem>,
    change_kind: LandscapeDiffChangeKind,
    claim_id: &str,
    summary: String,
    source_record_ids: Vec<String>,
) {
    items.push(LandscapeDiffItem::new(
        diff_item_id(change_kind, "claim", claim_id),
        change_kind,
        "claim",
        claim_id,
        summary,
        TraceRef::new(claim_id, source_record_ids),
    ));
}

#[instrument(level = "debug", skip(items, change_kind, assessment))]
pub(super) fn push_assessment_item(
    items: &mut Vec<LandscapeDiffItem>,
    change_kind: LandscapeDiffChangeKind,
    assessment: &LandscapeAssessment,
    action: &str,
    source_record_ids: Vec<String>,
) {
    let summary = format!(
        "{action} {} for {} with recommendation {:?}",
        assessment.id(),
        assessment.proof_id(),
        assessment.recommendation()
    );
    items.push(LandscapeDiffItem::new(
        diff_item_id(change_kind, "assessment", assessment.id()),
        change_kind,
        "assessment",
        assessment.id(),
        summary,
        TraceRef::new(assessment.proof_id(), source_record_ids),
    ));
}

#[instrument(level = "debug", skip(items, change_kind, gap))]
pub(super) fn push_gap_item(
    items: &mut Vec<LandscapeDiffItem>,
    change_kind: LandscapeDiffChangeKind,
    gap_key: &str,
    gap: &LandscapeGap,
    action: &str,
    source_record_ids: Vec<String>,
) {
    let summary = format!("{action} {:?} for {}", gap.kind(), gap.claim_id());
    items.push(LandscapeDiffItem::new(
        diff_item_id(change_kind, "gap", gap_key),
        change_kind,
        "gap",
        gap.claim_id(),
        summary,
        TraceRef::new(gap.claim_id(), source_record_ids),
    ));
}

#[instrument(level = "trace", skip(claim))]
pub(super) fn claim_summary(action: &str, claim: &LandscapeClaim) -> String {
    format!(
        "{action} claim {} with {} bases, {} verifier proofs, {} assessments",
        claim.id(),
        claim.bases().len(),
        claim.proofs().len(),
        claim.assessment_ids().len()
    )
}

#[instrument(level = "trace", skip(previous, current))]
pub(super) fn claim_change_summary(previous: &LandscapeClaim, current: &LandscapeClaim) -> String {
    format!(
        "changed claim {} from {} bases, {} verifier proofs, {} assessments to {} bases, {} verifier proofs, {} assessments",
        current.id(),
        previous.bases().len(),
        previous.proofs().len(),
        previous.assessment_ids().len(),
        current.bases().len(),
        current.proofs().len(),
        current.assessment_ids().len()
    )
}

#[instrument(level = "debug", skip(change_kind))]
fn diff_item_id(
    change_kind: LandscapeDiffChangeKind,
    subject_kind: &str,
    subject_id: &str,
) -> String {
    format!("belvedere.diff.{change_kind:?}.{subject_kind}:{subject_id}")
}
