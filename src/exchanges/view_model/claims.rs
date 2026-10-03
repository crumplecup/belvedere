//! Claim explorer and proof-chain projections.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    contracts::{
        ClaimExplorerCounts, ClaimExplorerItem, GapItem, GapSeverity, LandscapeAssessment,
        LandscapeClaim, ProofChainItem, ProofChainItemBuilder,
    },
    error::BelvedereConstructionError,
};

use super::trace;

#[instrument(level = "debug", skip(claims, assessments, gaps))]
pub(super) fn project_claim_explorer<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    assessments: &[LandscapeAssessment],
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Vec<ClaimExplorerItem> {
    let assessments_by_id = assessments_by_id(assessments);
    claims
        .map(|claim| {
            let claim_gap_count = claim_gap_count(claim.id(), gaps);
            let filter_ids = claim_filter_ids(claim, &assessments_by_id, gaps);
            ClaimExplorerItem::new(
                claim.id(),
                claim.is_root(),
                claim.bases().iter().cloned().collect(),
                claim.proofs().iter().cloned().collect(),
                filter_ids,
                ClaimExplorerCounts::new(
                    claim.contracts().len(),
                    claim.assessment_ids().len(),
                    claim_gap_count,
                ),
                trace(claim.id(), source_record_ids),
            )
        })
        .collect()
}

#[instrument(level = "debug", skip(assessments))]
fn assessments_by_id(assessments: &[LandscapeAssessment]) -> BTreeMap<&str, &LandscapeAssessment> {
    assessments
        .iter()
        .map(|assessment| (assessment.id().as_str(), assessment))
        .collect()
}

#[instrument(level = "debug", skip(claim, assessments_by_id, gaps))]
fn claim_filter_ids(
    claim: &LandscapeClaim,
    assessments_by_id: &BTreeMap<&str, &LandscapeAssessment>,
    gaps: &[GapItem],
) -> Vec<String> {
    let mut ids = BTreeSet::new();
    insert_filter_id(&mut ids, "claim_family", &claim_family(claim.id()));
    for verifier in claim.proofs() {
        insert_filter_id(&mut ids, "verifier", verifier);
    }
    for witness in claim.witness_surfaces() {
        insert_filter_id(&mut ids, "support_kind", witness.support_kind());
    }
    for proof in claim.kani_proofs() {
        insert_filter_id(&mut ids, "package", proof.package());
    }
    for assessment_id in claim.assessment_ids() {
        if let Some(assessment) = assessments_by_id.get(assessment_id.as_str()) {
            insert_filter_id(
                &mut ids,
                "recommendation",
                &format!("{:?}", assessment.recommendation()),
            );
        }
    }
    for gap in gaps.iter().filter(|gap| gap.claim_id() == claim.id()) {
        insert_filter_id(&mut ids, "gap_kind", &format!("{:?}", gap.kind()));
        if gap.high_priority() {
            insert_filter_id(&mut ids, "priority", "high");
        }
    }
    ids.into_iter().collect()
}

#[instrument(level = "trace", skip(ids))]
fn insert_filter_id(ids: &mut BTreeSet<String>, group: &str, value: &str) {
    ids.insert(format!("belvedere.filter.{group}:{value}"));
}

#[instrument(level = "trace", skip(claim_id))]
fn claim_family(claim_id: &str) -> String {
    claim_id
        .rsplit_once("::")
        .map_or_else(|| claim_id.to_owned(), |(family, _claim)| family.to_owned())
}

#[instrument(level = "debug", skip(claims, gaps, source_record_ids))]
pub(super) fn project_proof_chains(
    claims: &BTreeMap<String, LandscapeClaim>,
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<ProofChainItem>, BelvedereConstructionError> {
    claims
        .values()
        .map(|claim| {
            let review_state = chain_review_state(claim.id(), gaps);
            let next_action = chain_next_action(claim.id(), gaps);
            ProofChainItemBuilder::default()
                .claim_id(claim.id())
                .basis_ids(claim.bases().iter().cloned().collect::<Vec<_>>())
                .root_ids(chain_roots(claim.id(), claims))
                .verifiers(claim.proofs().iter().cloned().collect::<Vec<_>>())
                .review_state(review_state)
                .next_action(next_action)
                .gap_count(claim_gap_count(claim.id(), gaps))
                .trace(trace(claim.id(), source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("ProofChainItem", error))
        })
        .collect()
}

#[instrument(level = "trace", skip(gaps))]
fn claim_gap_count(claim_id: &str, gaps: &[GapItem]) -> usize {
    gaps.iter().filter(|gap| gap.claim_id() == claim_id).count()
}

#[instrument(level = "trace", skip(gaps))]
fn chain_review_state(claim_id: &str, gaps: &[GapItem]) -> &'static str {
    let claim_gaps = gaps.iter().filter(|gap| gap.claim_id() == claim_id);
    if claim_gaps
        .clone()
        .any(|gap| gap.severity() == GapSeverity::Blocking)
    {
        "blocked"
    } else if claim_gaps
        .clone()
        .any(|gap| gap.severity() == GapSeverity::Elevated)
    {
        "elevated"
    } else if claim_gaps.count() > 0 {
        "review"
    } else {
        "ready"
    }
}

#[instrument(level = "trace", skip(gaps))]
fn chain_next_action(claim_id: &str, gaps: &[GapItem]) -> String {
    gaps.iter()
        .filter(|gap| gap.claim_id() == claim_id)
        .find(|gap| gap.high_priority())
        .or_else(|| gaps.iter().find(|gap| gap.claim_id() == claim_id))
        .map_or_else(
            || "continue review".to_owned(),
            |gap| gap.action().to_owned(),
        )
}

#[instrument(level = "debug", skip(claims))]
fn chain_roots(claim_id: &str, claims: &BTreeMap<String, LandscapeClaim>) -> Vec<String> {
    let mut roots = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    collect_chain_roots(claim_id, claims, &mut visiting, &mut roots);
    roots.into_iter().collect()
}

#[instrument(level = "trace", skip(claims, visiting, roots))]
fn collect_chain_roots(
    claim_id: &str,
    claims: &BTreeMap<String, LandscapeClaim>,
    visiting: &mut BTreeSet<String>,
    roots: &mut BTreeSet<String>,
) {
    if !visiting.insert(claim_id.to_owned()) {
        roots.insert(claim_id.to_owned());
        return;
    }
    let Some(claim) = claims.get(claim_id) else {
        roots.insert(claim_id.to_owned());
        visiting.remove(claim_id);
        return;
    };
    if claim.is_root() {
        roots.insert(claim.id().to_owned());
        visiting.remove(claim_id);
        return;
    }
    for basis in claim.bases() {
        collect_chain_roots(basis, claims, visiting, roots);
    }
    visiting.remove(claim_id);
}
