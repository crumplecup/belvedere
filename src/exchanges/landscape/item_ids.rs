//! Stable output IDs emitted by the landscape modeling exchange.

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

#[instrument(level = "debug", skip(kind))]
fn gap_kind_id(kind: LandscapeGapKind) -> &'static str {
    match kind {
        LandscapeGapKind::MissingProof => "missing_proof",
        LandscapeGapKind::MissingAssessment => "missing_assessment",
        LandscapeGapKind::ReplacementRecommended => "replacement_recommended",
        LandscapeGapKind::RetirementRecommended => "retirement_recommended",
        LandscapeGapKind::TrustedOrOpaqueWitnessSupport => "trusted_or_opaque_witness_support",
    }
}
