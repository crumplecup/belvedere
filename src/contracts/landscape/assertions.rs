//! Independent input-to-claim fidelity checks for landscape modeling.

use std::collections::BTreeMap;

use tracing::instrument;

use super::{
    KaniProofSummary, LandscapeClaim, LandscapeContractFragment, LandscapeGraph, LandscapeInput,
    WitnessSurfaceSummary,
};

#[instrument(level = "debug", skip(input, output))]
pub(super) fn claims_match_input(input: &LandscapeInput, output: &LandscapeGraph) -> bool {
    let mut expected = BTreeMap::<String, LandscapeClaim>::new();
    for link in input.registry().evidence_links() {
        ensure_claim(&mut expected, link.basis());
        claim(&mut expected, link.name()).add_basis(link.basis());
    }
    for proof in input.registry().proof_records() {
        claim(&mut expected, proof.evidence()).add_verifier(proof.verifier());
    }
    for contract in input.registry().contract_records() {
        claim(&mut expected, contract.evidence()).add_contract(LandscapeContractFragment::new(
            contract.verifier(),
            contract.kind(),
            contract.fragment(),
        ));
    }
    for witness in input.registry().witness_export_records() {
        claim(&mut expected, witness.evidence())
            .add_witness_surface(WitnessSurfaceSummary::from_record(witness));
    }
    for registration in input.registry().kani_proofs() {
        let proof = KaniProofSummary::from_record(registration);
        let modeled = claim(&mut expected, registration.id());
        if proof.provides_coverage() {
            modeled.add_verifier("kani");
        }
        modeled.add_kani_proof(proof);
    }
    for assessment in input.assessments().assessments() {
        let id = assessment.assessment_id().map_or_else(
            || {
                format!(
                    "assessment:{}@{}",
                    assessment.proof_id(),
                    assessment.timestamp()
                )
            },
            ToOwned::to_owned,
        );
        claim(&mut expected, assessment.proof_id()).add_assessment(id);
    }
    output.claims() == &expected
}

#[instrument(level = "trace", skip(claims))]
fn ensure_claim(claims: &mut BTreeMap<String, LandscapeClaim>, id: &str) {
    claims
        .entry(id.to_owned())
        .or_insert_with(|| LandscapeClaim::new(id));
}

#[instrument(level = "trace", skip(claims))]
fn claim<'a>(claims: &'a mut BTreeMap<String, LandscapeClaim>, id: &str) -> &'a mut LandscapeClaim {
    claims
        .entry(id.to_owned())
        .or_insert_with(|| LandscapeClaim::new(id))
}
