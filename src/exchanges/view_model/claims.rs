//! Claim explorer and proof-chain projections.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    contracts::{
        ClaimExplorerCounts, ClaimExplorerItem, GapItem, LandscapeClaim, ProofChainItem,
        ProofChainItemBuilder,
    },
    error::BelvedereConstructionError,
};

use super::trace;

#[instrument(level = "debug", skip(claims, gaps, source_record_ids))]
pub(super) fn project_claim_explorer<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Vec<ClaimExplorerItem> {
    claims
        .map(|claim| {
            let claim_gap_count = claim_gap_count(claim.id(), gaps);
            ClaimExplorerItem::new(
                claim.id(),
                claim.is_root(),
                claim.bases().iter().cloned().collect(),
                claim.proofs().iter().cloned().collect(),
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
pub(super) fn project_proof_chains(
    claims: &BTreeMap<String, LandscapeClaim>,
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<ProofChainItem>, BelvedereConstructionError> {
    claims
        .values()
        .map(|claim| {
            ProofChainItemBuilder::default()
                .claim_id(claim.id())
                .basis_ids(claim.bases().iter().cloned().collect::<Vec<_>>())
                .root_ids(chain_roots(claim.id(), claims))
                .verifiers(claim.proofs().iter().cloned().collect::<Vec<_>>())
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
