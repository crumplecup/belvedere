//! Independent input-to-claim fidelity checks for landscape modeling.

use std::collections::BTreeMap;

use tracing::instrument;

use super::{
    KaniProofSummary, LandscapeAssessment, LandscapeClaim, LandscapeContractFragment, LandscapeGap,
    LandscapeGapKind, LandscapeGraph, LandscapeInput, WitnessSurfaceSummary,
};
use crate::contracts::{Recommendation, ResolutionPath};

#[instrument(level = "debug", skip(input, output))]
pub(super) fn claims_match_input(input: &LandscapeInput, output: &LandscapeGraph) -> bool {
    output.claims() == &expected_claims(input)
}

#[instrument(level = "debug", skip(input, output))]
pub(super) fn assessments_match_input(input: &LandscapeInput, output: &LandscapeGraph) -> bool {
    output.assessments() == &expected_assessments(input)
}

#[instrument(level = "debug", skip(input, output))]
pub(super) fn gaps_match_input(input: &LandscapeInput, output: &LandscapeGraph) -> bool {
    let claims = expected_claims(input);
    let assessments = expected_assessments(input);
    output.gaps() == &expected_gaps(&claims, &assessments)
}

#[instrument(level = "trace", skip(input))]
fn expected_claims(input: &LandscapeInput) -> BTreeMap<String, LandscapeClaim> {
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
    expected
}

#[instrument(level = "trace", skip(input))]
fn expected_assessments(input: &LandscapeInput) -> Vec<LandscapeAssessment> {
    let mut latest = BTreeMap::<String, (u64, String)>::new();
    let mut expected = input
        .assessments()
        .assessments()
        .iter()
        .map(LandscapeAssessment::from_record)
        .collect::<Vec<_>>();
    for assessment in &expected {
        let candidate = (assessment.timestamp(), assessment.id().clone());
        let entry = latest
            .entry(assessment.proof_id().clone())
            .or_insert_with(|| candidate.clone());
        *entry = std::cmp::max(entry.clone(), candidate);
    }
    for assessment in &mut expected {
        if let Some((_, id)) = latest.get(assessment.proof_id()) {
            if id == assessment.id() {
                assessment.mark_current();
            } else {
                assessment.mark_superseded_by(id);
            }
        }
    }
    expected
}

#[instrument(level = "trace", skip(claims, assessments))]
fn expected_gaps(
    claims: &BTreeMap<String, LandscapeClaim>,
    assessments: &[LandscapeAssessment],
) -> Vec<LandscapeGap> {
    let by_id = assessments
        .iter()
        .map(|assessment| (assessment.id().as_str(), assessment))
        .collect::<BTreeMap<_, _>>();
    let mut expected = Vec::new();
    for claim in claims.values() {
        if claim.proofs().is_empty() {
            expected.push(LandscapeGap::new(
                claim.id(),
                LandscapeGapKind::MissingProof,
                "claim has no modeled proof coverage",
            ));
        }
        if (!claim.proofs().is_empty() || !claim.kani_proofs().is_empty())
            && claim.assessment_ids().is_empty()
        {
            expected.push(LandscapeGap::new(
                claim.id(),
                LandscapeGapKind::MissingAssessment,
                "claim has proof coverage but no assessment",
            ));
        }
        if let Some(assessment) = claim
            .assessment_ids()
            .iter()
            .filter_map(|id| by_id.get(id.as_str()).copied())
            .max_by_key(|assessment| (assessment.timestamp(), assessment.id()))
        {
            let disposition = match assessment.recommendation() {
                Recommendation::Replace => Some((
                    LandscapeGapKind::ReplacementRecommended,
                    "latest assessment recommends replacement",
                )),
                Recommendation::Retire => Some((
                    LandscapeGapKind::RetirementRecommended,
                    "latest assessment recommends retirement",
                )),
                Recommendation::Accept | Recommendation::Strengthen => None,
            };
            if let Some((kind, prefix)) = disposition {
                expected.push(
                    LandscapeGap::new(claim.id(), kind, resolution_detail(prefix, assessment))
                        .with_triggering_assessment(assessment.id()),
                );
            }
        }
        for witness in claim.witness_surfaces() {
            if witness.trusted() + witness.opaque() > 0 {
                expected.push(
                    LandscapeGap::new(
                        claim.id(),
                        LandscapeGapKind::TrustedOrOpaqueWitnessSupport,
                        format!(
                            "witness support includes {} trusted and {} opaque obligations",
                            witness.trusted(),
                            witness.opaque()
                        ),
                    )
                    .with_origin_key(format!(
                        "{}@{}",
                        witness.verifier(),
                        witness.destination_module()
                    )),
                );
            }
        }
        for proof in claim
            .kani_proofs()
            .iter()
            .filter(|proof| !proof.provides_coverage())
        {
            expected.push(
                LandscapeGap::new(
                    claim.id(),
                    LandscapeGapKind::ProofExecutionUnresolved,
                    format!(
                        "Kani proof {} is {} in scope {}: {}",
                        proof.harness(),
                        proof.status_label(),
                        proof.scope(),
                        proof
                            .failure_reason()
                            .unwrap_or("no backend reason reported")
                    ),
                )
                .with_origin_key(format!(
                    "{}@{}",
                    proof.package(),
                    proof.harness()
                )),
            );
        }
    }
    expected
}

#[instrument(level = "trace", skip(assessment))]
fn resolution_detail(prefix: &str, assessment: &LandscapeAssessment) -> String {
    let suffix = match assessment.resolution_path() {
        Some(ResolutionPath::KeepCurrentProof) => "keep current proof",
        Some(ResolutionPath::StrengthenCurrentProof) => "strengthen current proof",
        Some(ResolutionPath::ReplaceWithProofSpecificModel) => "replace with proof-specific model",
        Some(ResolutionPath::ReplaceWithAccommodationModel) => "replace with accommodation model",
        Some(ResolutionPath::RetireClaim) => "retire claim",
        None => return prefix.to_owned(),
    };
    format!("{prefix}: {suffix}")
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
