//! Auditor navigation view-model exchange implementations.

use tracing::instrument;

use crate::{
    certify::{ViewModelReceipt, ViewModelReceiptBuilder, ViewModelReceiptCountsBuilder},
    contracts::{
        AssessmentRecencySummary, AuditorNavigationContract, AuditorNavigationModel,
        AuditorNavigationModelBuilder, BelvedereContract, LandscapeAssessment, OverviewView,
        ProvenanceRootItem, TraceRef, ViewModelInput,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};

mod assessments;
mod claims;
mod coverage;
mod facets;
mod gaps;
mod sources;
mod witness;

use assessments::{project_assessment_queue, project_assessment_work};
use claims::{project_claim_explorer, project_proof_chains};
use coverage::project_backend_coverage;
use facets::project_filter_facets;
use gaps::project_gaps;
use sources::project_source_navigation;
use witness::project_witness_surface;

/// Exchange from a landscape graph into UI-neutral auditor navigation evidence.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditorNavigationProjection;

impl AuditorNavigationProjection {
    /// Project a landscape graph into auditor navigation evidence.
    ///
    /// # Errors
    ///
    /// Returns an error if crate-internal evidence builders reject the projected
    /// navigation model or receipt. Missing or uncertain review facts are
    /// represented as view-model gap items.
    #[instrument(level = "debug", skip(self, input))]
    pub fn project(
        &self,
        input: ViewModelInput,
    ) -> Result<(AuditorNavigationModel, ViewModelReceipt), BelvedereError> {
        <Self as BelvedereExchange<AuditorNavigationContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<AuditorNavigationContract> for AuditorNavigationProjection {
    type Error = BelvedereError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: ViewModelInput,
    ) -> Result<(AuditorNavigationModel, ViewModelReceipt), Self::Error> {
        let landscape = input.landscape();
        let source_record_ids = landscape.source_record_ids().to_vec();
        let gaps = project_gaps(landscape.gaps(), &source_record_ids)?;
        let filter_facets = project_filter_facets(
            landscape.claims().values(),
            landscape.assessments(),
            &gaps,
            &source_record_ids,
        )?;
        let high_priority_gap_count = gaps.iter().filter(|gap| gap.high_priority()).count();
        let overview = OverviewView::new(
            landscape.claims().len(),
            landscape.root_claim_ids().len(),
            landscape.proof_count(),
            assessment_recency_summary(landscape.assessments()),
            gaps.len(),
            high_priority_gap_count,
        );

        let backend_coverage =
            project_backend_coverage(landscape.claims().values(), &source_record_ids)?;

        let claim_explorer = project_claim_explorer(
            landscape.claims().values(),
            landscape.assessments(),
            &gaps,
            &source_record_ids,
        );
        let proof_chains = project_proof_chains(landscape.claims(), &gaps, &source_record_ids)?;
        let assessment_queue =
            project_assessment_queue(landscape.assessments(), &source_record_ids)?;
        let assessment_work =
            project_assessment_work(landscape.assessments(), &gaps, &source_record_ids)?;
        let witness_surface =
            project_witness_surface(landscape.claims().values(), &source_record_ids)?;

        let provenance_roots: Vec<ProvenanceRootItem> = landscape
            .root_claim_ids()
            .into_iter()
            .filter_map(|claim_id| landscape.claims().get(claim_id))
            .map(|claim| {
                ProvenanceRootItem::new(
                    claim.id(),
                    provenance_root_kind(claim),
                    claim.proofs().len(),
                    claim.assessment_ids().len(),
                    gaps.iter()
                        .filter(|gap| gap.claim_id() == claim.id())
                        .count(),
                    trace(claim.id(), &source_record_ids),
                )
            })
            .collect();

        let source_navigation = project_source_navigation(&source_record_ids)?;

        let counts = ViewModelReceiptCountsBuilder::default()
            .claim_cards(landscape.claims().len())
            .filter_facets(filter_facets.len())
            .backend_coverage_items(backend_coverage.len())
            .proof_chain_items(proof_chains.len())
            .assessment_items(landscape.assessments().len())
            .assessment_work_items(assessment_work.len())
            .witness_items(witness_surface.len())
            .root_items(landscape.root_claim_ids().len())
            .gap_items(gaps.len())
            .source_items(source_navigation.len())
            .build()
            .map_err(|error| BelvedereConstructionError::new("ViewModelReceiptCounts", error))?;

        let model = AuditorNavigationModelBuilder::default()
            .overview(overview)
            .filter_facets(filter_facets)
            .backend_coverage(backend_coverage)
            .claim_explorer(claim_explorer)
            .proof_chains(proof_chains)
            .assessment_queue(assessment_queue)
            .assessment_work(assessment_work)
            .witness_surface(witness_surface)
            .provenance_roots(provenance_roots)
            .gaps(gaps)
            .source_navigation(source_navigation)
            .source_record_ids(source_record_ids.clone())
            .build()
            .map_err(|error| BelvedereConstructionError::new("AuditorNavigationModel", error))?;
        let receipt = ViewModelReceiptBuilder::default()
            .contract(AuditorNavigationContract::NAME)
            .counts(counts)
            .source_record_ids(source_record_ids)
            .warnings(Vec::new())
            .build()
            .map_err(|error| BelvedereConstructionError::new("ViewModelReceipt", error))?;

        Ok((model, receipt))
    }
}

#[instrument(level = "debug", skip(assessments))]
fn assessment_recency_summary(assessments: &[LandscapeAssessment]) -> AssessmentRecencySummary {
    AssessmentRecencySummary::new(
        assessments
            .iter()
            .filter(|assessment| assessment.current())
            .count(),
        assessments
            .iter()
            .filter(|assessment| !assessment.current())
            .count(),
    )
}

#[instrument(level = "debug")]
pub(super) fn trace(claim_id: &str, source_record_ids: &[String]) -> TraceRef {
    TraceRef::new(claim_id, source_record_ids.to_vec())
}

#[instrument(level = "trace", skip(claim))]
fn provenance_root_kind(claim: &crate::contracts::LandscapeClaim) -> &'static str {
    if !claim.kani_proofs().is_empty() {
        "proof_registration_root"
    } else if !claim.proofs().is_empty() {
        "covered_root"
    } else if !claim.assessment_ids().is_empty() {
        "assessment_only_root"
    } else {
        "unmodeled_root"
    }
}
