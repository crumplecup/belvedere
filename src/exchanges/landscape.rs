//! Landscape modeling exchange implementations.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    certify::{LandscapeReceipt, LandscapeReceiptBuilder, LandscapeReceiptCounts},
    contracts::{
        BelvedereContract, ClaimEdge, KaniProofSummary, LandscapeAssessment, LandscapeClaim,
        LandscapeContractFragment, LandscapeGraph, LandscapeGraphBuilder, LandscapeInput,
        LandscapeModelingContract, WitnessSurfaceSummary,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};

mod gaps;
mod item_ids;

use gaps::{gap_kind_counts, model_gaps};
use item_ids::landscape_item_ids;

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
            let proof = KaniProofSummary::from_record(record);
            let claim = claim_mut(&mut claims, record.id());
            if proof.provides_coverage() {
                claim.add_verifier("kani");
            }
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
            .gap_kind_counts(gap_kind_counts(graph.gaps()))
            .assessment_recency_counts(assessment_recency_counts(graph.assessments()))
            .landscape_item_ids(landscape_item_ids(&graph))
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

#[instrument(level = "debug", skip(assessments))]
fn assessment_recency_counts(assessments: &[LandscapeAssessment]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for assessment in assessments {
        let key = if assessment.current() {
            "assessment_recency_current"
        } else {
            "assessment_recency_superseded"
        };
        *counts.entry(key.to_owned()).or_default() += 1;
    }
    counts
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
