//! Claim explorer and proof-chain projections.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    contracts::{
        ClaimDetailItem, ClaimDetailItemBuilder, ClaimExplorerCounts, ClaimExplorerItem, GapItem,
        GapSeverity, LandscapeAssessment, LandscapeClaim, ProofChainBackendCoverage,
        ProofChainBackendCoverageBuilder, ProofChainItem, ProofChainItemBuilder,
        ProofChainTopology,
    },
    error::BelvedereConstructionError,
};

use super::{coverage::TARGET_VERIFIERS, trace};

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

#[instrument(level = "debug", skip(claims, gaps, source_record_ids))]
pub(super) fn project_claim_details(
    claims: &BTreeMap<String, LandscapeClaim>,
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<ClaimDetailItem>, BelvedereConstructionError> {
    claims
        .values()
        .map(|claim| {
            let trace = trace(claim.id(), source_record_ids);
            ClaimDetailItemBuilder::default()
                .claim_id(claim.id())
                .root(claim.is_root())
                .basis_ids(claim.bases().iter().cloned().collect::<Vec<_>>())
                .root_ids(chain_roots(claim.id(), claims))
                .verifiers(claim.proofs().iter().cloned().collect::<Vec<_>>())
                .contract_fragments(contract_fragment_labels(claim))
                .assessment_ids(claim.assessment_ids().to_vec())
                .gap_summaries(claim_gap_summaries(claim.id(), gaps))
                .review_state(chain_review_state(claim.id(), gaps))
                .next_action(chain_next_action(claim.id(), gaps))
                .source_record_count(trace.source_record_ids().len())
                .trace(trace)
                .build()
                .map_err(|error| BelvedereConstructionError::new("ClaimDetailItem", error))
        })
        .collect()
}

#[instrument(level = "trace", skip(claim))]
fn contract_fragment_labels(claim: &LandscapeClaim) -> Vec<String> {
    claim
        .contracts()
        .iter()
        .map(|contract| {
            format!(
                "{} {} {}",
                contract.verifier(),
                contract.kind(),
                contract.fragment()
            )
        })
        .collect()
}

#[instrument(level = "trace", skip(gaps))]
fn claim_gap_summaries(claim_id: &str, gaps: &[GapItem]) -> Vec<String> {
    gaps.iter()
        .filter(|gap| gap.claim_id() == claim_id)
        .map(|gap| {
            format!(
                "{:?} {}: {}",
                gap.kind(),
                gap.severity().label(),
                gap.action()
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
            let traversal = chain_traversal(claim.id(), claims);
            let backend_coverage = chain_backend_coverage(&traversal, claims)?;
            let review_state = chain_review_state(claim.id(), gaps);
            let next_action = chain_next_action(claim.id(), gaps);
            ProofChainItemBuilder::default()
                .claim_id(claim.id())
                .basis_ids(claim.bases().iter().cloned().collect::<Vec<_>>())
                .root_ids(chain_roots(claim.id(), claims))
                .verifiers(claim.proofs().iter().cloned().collect::<Vec<_>>())
                .backend_coverage(backend_coverage)
                .topology(chain_topology(&traversal, claims, gaps))
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChainTraversal {
    reached: BTreeSet<String>,
    max_depth: usize,
}

#[instrument(level = "debug", skip(claims))]
fn chain_traversal(claim_id: &str, claims: &BTreeMap<String, LandscapeClaim>) -> ChainTraversal {
    let mut reached = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    let max_depth = collect_chain_depth(claim_id, claims, &mut visiting, &mut reached);
    ChainTraversal { reached, max_depth }
}

#[instrument(level = "debug", skip(traversal, claims))]
fn chain_backend_coverage(
    traversal: &ChainTraversal,
    claims: &BTreeMap<String, LandscapeClaim>,
) -> Result<Vec<ProofChainBackendCoverage>, BelvedereConstructionError> {
    TARGET_VERIFIERS
        .iter()
        .map(|verifier| {
            let covered_claim_count = traversal
                .reached
                .iter()
                .filter_map(|claim_id| claims.get(claim_id))
                .filter(|claim| claim.proofs().contains(*verifier))
                .count();
            let missing_claim_count = traversal.reached.len().saturating_sub(covered_claim_count);
            ProofChainBackendCoverageBuilder::default()
                .verifier(*verifier)
                .complete(missing_claim_count == 0)
                .covered_claim_count(covered_claim_count)
                .missing_claim_count(missing_claim_count)
                .build()
                .map_err(|error| {
                    BelvedereConstructionError::new("ProofChainBackendCoverage", error)
                })
        })
        .collect()
}

#[instrument(level = "debug", skip(traversal, claims, gaps))]
fn chain_topology(
    traversal: &ChainTraversal,
    claims: &BTreeMap<String, LandscapeClaim>,
    gaps: &[GapItem],
) -> ProofChainTopology {
    let branching_claim_count = traversal
        .reached
        .iter()
        .filter_map(|reached_id| claims.get(reached_id))
        .filter(|claim| direct_basis_count(claim) > 1)
        .count();
    let chain_gap_count = gaps
        .iter()
        .filter(|gap| traversal.reached.contains(gap.claim_id()))
        .count();
    ProofChainTopology::new(
        traversal.reached.len(),
        traversal.max_depth,
        branching_claim_count,
        chain_gap_count,
    )
}

#[instrument(level = "trace", skip(claims, visiting, reached))]
fn collect_chain_depth(
    claim_id: &str,
    claims: &BTreeMap<String, LandscapeClaim>,
    visiting: &mut BTreeSet<String>,
    reached: &mut BTreeSet<String>,
) -> usize {
    reached.insert(claim_id.to_owned());
    if !visiting.insert(claim_id.to_owned()) {
        return 0;
    }

    let Some(claim) = claims.get(claim_id) else {
        visiting.remove(claim_id);
        return 0;
    };

    let max_depth = claim
        .bases()
        .iter()
        .filter(|basis| basis.as_str() != claim.id())
        .map(|basis| 1 + collect_chain_depth(basis, claims, visiting, reached))
        .max()
        .unwrap_or(0);
    visiting.remove(claim_id);
    max_depth
}

#[instrument(level = "trace", skip(claim))]
fn direct_basis_count(claim: &LandscapeClaim) -> usize {
    claim
        .bases()
        .iter()
        .filter(|basis| basis.as_str() != claim.id())
        .count()
}
