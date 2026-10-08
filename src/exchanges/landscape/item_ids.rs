//! Stable output IDs emitted by the landscape modeling exchange.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::contracts::{LandscapeGapKind, LandscapeGraph};

#[instrument(level = "debug", skip(graph))]
pub(super) fn landscape_item_ids(graph: &LandscapeGraph) -> Vec<String> {
    let mut ids = Vec::new();

    ids.extend(
        graph
            .claims()
            .keys()
            .map(|claim_id| format!("landscape.claim:{claim_id}")),
    );
    ids.extend(
        graph
            .root_claim_ids()
            .into_iter()
            .map(|claim_id| format!("landscape.root:{claim_id}")),
    );
    ids.extend(graph.edges().iter().map(|edge| {
        format!(
            "landscape.edge:{}:{}:{}",
            edge.claim_id(),
            edge.basis_id(),
            edge.source_index()
        )
    }));

    for claim in graph.claims().values() {
        ids.extend(
            claim
                .proofs()
                .iter()
                .map(|verifier| format!("landscape.proof:{}:{verifier}", claim.id())),
        );
        ids.extend(
            claim
                .contracts()
                .iter()
                .enumerate()
                .map(|(index, contract)| {
                    format!(
                        "landscape.contract:{}:{}:{}:{}",
                        claim.id(),
                        contract.verifier(),
                        contract.kind(),
                        index
                    )
                }),
        );
        ids.extend(claim.kani_proofs().iter().map(|proof| {
            format!(
                "landscape.kani_proof:{}:{}:{}",
                proof.id(),
                proof.harness(),
                proof.package()
            )
        }));
        ids.extend(
            claim
                .witness_surfaces()
                .iter()
                .enumerate()
                .map(|(index, witness)| {
                    format!(
                        "landscape.witness:{}:{}:{}:{}:{}",
                        claim.id(),
                        witness.verifier(),
                        witness.destination_module(),
                        witness.support_kind(),
                        index
                    )
                }),
        );
    }

    ids.extend(
        graph
            .assessments()
            .iter()
            .map(|assessment| format!("landscape.assessment:{}", assessment.id())),
    );
    ids.extend(graph.gaps().iter().enumerate().map(|(index, gap)| {
        format!(
            "landscape.gap:{}:{}:{}",
            gap.claim_id(),
            gap_kind_id(gap.kind()),
            index
        )
    }));

    ids
}

#[instrument(level = "debug", skip(item_ids))]
pub(super) fn landscape_item_kind_counts(item_ids: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item_id in item_ids {
        let Some(kind) = item_id
            .strip_prefix("landscape.")
            .and_then(|suffix| suffix.split_once(':').map(|(kind, _id)| kind))
        else {
            continue;
        };
        *counts
            .entry(format!("landscape_item_kind_{kind}"))
            .or_default() += 1;
    }
    counts
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
