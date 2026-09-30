//! Landscape modeling exchange implementations.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    certify::{LandscapeReceipt, LandscapeReceiptBuilder, LandscapeReceiptCounts},
    contracts::{
        BelvedereContract, ClaimEdge, KaniProofSummary, LandscapeAssessment, LandscapeClaim,
        LandscapeContractFragment, LandscapeGap, LandscapeGapKind, LandscapeGraph,
        LandscapeGraphBuilder, LandscapeInput, LandscapeModelingContract, Recommendation,
        ResolutionPath, WitnessSurfaceSummary,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};

/// Exchange for decoded Amenable records into a defensive landscape graph.
#[derive(Debug, Clone, Copy, Default)]
pub struct LandscapeModeling;

impl LandscapeModeling {
    /// Model a defensive landscape from decoded ingestion evidence.
    ///
    /// # Errors
    ///
    /// Returns an error if crate-internal evidence builders reject the modeled
    /// landscape or receipt. Conservative uncertainty is represented as modeled
    /// gaps rather than exchange failure.
    #[instrument(level = "debug", skip(self, input))]
    pub fn model(
        &self,
        input: LandscapeInput,
    ) -> Result<(LandscapeGraph, LandscapeReceipt), BelvedereError> {
        <Self as BelvedereExchange<LandscapeModelingContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<LandscapeModelingContract> for LandscapeModeling {
    type Error = BelvedereError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: LandscapeInput,
    ) -> Result<(LandscapeGraph, LandscapeReceipt), Self::Error> {
        let mut claims = BTreeMap::new();
        let mut edges = Vec::new();
        let mut assessments = Vec::new();
        let mut source_record_ids = input.registry().source_record_ids();
        source_record_ids.extend(input.assessments().source_record_ids());

        for record in input.registry().evidence_links() {
            ensure_claim(&mut claims, record.name());
            ensure_claim(&mut claims, record.basis());
            claim_mut(&mut claims, record.name()).add_basis(record.basis());
            edges.push(ClaimEdge::new(
                record.name(),
                record.basis(),
                record.index(),
            ));
        }

        for record in input.registry().proof_records() {
            ensure_claim(&mut claims, record.evidence());
            claim_mut(&mut claims, record.evidence()).add_verifier(record.verifier());
        }

        for record in input.registry().contract_records() {
            ensure_claim(&mut claims, record.evidence());
            claim_mut(&mut claims, record.evidence()).add_contract(LandscapeContractFragment::new(
                record.verifier(),
                record.kind(),
                record.fragment(),
            ));
        }

        for record in input.registry().witness_export_records() {
            ensure_claim(&mut claims, record.evidence());
            claim_mut(&mut claims, record.evidence())
                .add_witness_surface(WitnessSurfaceSummary::from_record(record));
        }

        for record in input.registry().kani_proofs() {
            ensure_claim(&mut claims, record.id());
            let proof = KaniProofSummary::new(record.id(), record.harness(), record.package());
            let claim = claim_mut(&mut claims, record.id());
            claim.add_verifier("kani");
            claim.add_kani_proof(proof);
        }

        for record in input.assessments().assessments() {
            ensure_claim(&mut claims, record.proof_id());
            let assessment = LandscapeAssessment::from_record(record);
            claim_mut(&mut claims, record.proof_id()).add_assessment(assessment.id());
            assessments.push(assessment);
        }

        mark_assessment_recency(&mut assessments);
        let gaps = model_gaps(&claims, &assessments);
        let graph = LandscapeGraphBuilder::default()
            .claims(claims)
            .edges(edges)
            .assessments(assessments)
            .gaps(gaps)
            .source_record_ids(source_record_ids)
            .build()
            .map_err(|error| BelvedereConstructionError::new("LandscapeGraph", error))?;
        let counts = LandscapeReceiptCounts::new(
            graph.claims().len(),
            graph.edges().len(),
            graph.root_claim_ids().len(),
            graph.proof_count(),
            graph.assessments().len(),
            graph.gaps().len(),
        );
        let receipt = LandscapeReceiptBuilder::default()
            .contract(LandscapeModelingContract::NAME)
            .counts(counts)
            .source_record_ids(graph.source_record_ids().to_vec())
            .warnings(Vec::new())
            .build()
            .map_err(|error| BelvedereConstructionError::new("LandscapeReceipt", error))?;

        Ok((graph, receipt))
    }
}

#[instrument(level = "debug", skip(assessments))]
fn mark_assessment_recency(assessments: &mut [LandscapeAssessment]) {
    let mut latest_by_proof: BTreeMap<String, (u64, String)> = BTreeMap::new();
    for assessment in assessments.iter() {
        latest_by_proof
            .entry(assessment.proof_id().to_owned())
            .and_modify(|latest| {
                let timestamp = assessment.timestamp();
                let id = assessment.id();
                if timestamp > latest.0
                    || (timestamp == latest.0 && id.as_str() > latest.1.as_str())
                {
                    *latest = (timestamp, id.to_owned());
                }
            })
            .or_insert_with(|| (assessment.timestamp(), assessment.id().to_owned()));
    }

    for assessment in assessments {
        let latest_id = latest_by_proof
            .get(assessment.proof_id())
            .map(|(_timestamp, id)| id.as_str());
        match latest_id {
            Some(id) if id == assessment.id() => assessment.mark_current(),
            Some(id) => assessment.mark_superseded_by(id),
            None => assessment.mark_current(),
        }
    }
}

#[instrument(level = "debug", skip(claims))]
fn ensure_claim(claims: &mut BTreeMap<String, LandscapeClaim>, id: &str) {
    claims
        .entry(id.to_owned())
        .or_insert_with(|| LandscapeClaim::new(id));
}

#[instrument(level = "debug", skip(claims))]
fn claim_mut<'a>(
    claims: &'a mut BTreeMap<String, LandscapeClaim>,
    id: &str,
) -> &'a mut LandscapeClaim {
    claims
        .entry(id.to_owned())
        .or_insert_with(|| LandscapeClaim::new(id))
}

#[instrument(level = "debug", skip(claims, assessments))]
fn model_gaps(
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
