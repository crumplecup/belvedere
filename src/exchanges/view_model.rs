//! Auditor navigation view-model exchange implementations.

use tracing::instrument;

use crate::{
    certify::{ViewModelReceipt, ViewModelReceiptBuilder, ViewModelReceiptCounts},
    contracts::{
        AssessmentQueueItemBuilder, AuditorNavigationContract, AuditorNavigationModel,
        AuditorNavigationModelBuilder, BelvedereContract, ClaimExplorerCounts, ClaimExplorerItem,
        GapItem, GapItemBuilder, LandscapeGap, LandscapeGapKind, OverviewView, ProvenanceRootItem,
        Rubric, TraceRef, ViewModelInput, WitnessSurfaceItemBuilder,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};

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
        let high_priority_gap_count = gaps.iter().filter(|gap| gap.high_priority()).count();
        let overview = OverviewView::new(
            landscape.claims().len(),
            landscape.root_claim_ids().len(),
            landscape.proof_count(),
            landscape.assessments().len(),
            gaps.len(),
            high_priority_gap_count,
        );

        let claim_explorer: Vec<ClaimExplorerItem> = landscape
            .claims()
            .values()
            .map(|claim| {
                let claim_gap_count = gaps
                    .iter()
                    .filter(|gap| gap.claim_id() == claim.id())
                    .count();
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
                    trace(claim.id(), &source_record_ids),
                )
            })
            .collect();

        let assessment_queue = landscape
            .assessments()
            .iter()
            .map(|assessment| {
                AssessmentQueueItemBuilder::default()
                    .assessment_id(assessment.id())
                    .proof_id(assessment.proof_id())
                    .recommendation(assessment.recommendation())
                    .resolution_path(assessment.resolution_path())
                    .weak_dimensions(weak_dimensions(assessment.rubric()))
                    .trace(trace(assessment.proof_id(), &source_record_ids))
                    .build()
                    .map_err(|error| BelvedereConstructionError::new("AssessmentQueueItem", error))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut witness_surface = Vec::new();
        for claim in landscape.claims().values() {
            for witness in claim.witness_surfaces() {
                let item = WitnessSurfaceItemBuilder::default()
                    .claim_id(claim.id())
                    .verifier(witness.verifier())
                    .destination_module(witness.destination_module())
                    .checked(witness.checked())
                    .trusted(witness.trusted())
                    .opaque(witness.opaque())
                    .trace(trace(claim.id(), &source_record_ids))
                    .build()
                    .map_err(|error| {
                        BelvedereConstructionError::new("WitnessSurfaceItem", error)
                    })?;
                witness_surface.push(item);
            }
        }

        let provenance_roots: Vec<ProvenanceRootItem> = landscape
            .root_claim_ids()
            .into_iter()
            .map(|claim_id| ProvenanceRootItem::new(claim_id, trace(claim_id, &source_record_ids)))
            .collect();

        let counts = ViewModelReceiptCounts::new(
            landscape.claims().len(),
            landscape.assessments().len(),
            witness_surface.len(),
            landscape.root_claim_ids().len(),
            gaps.len(),
        );

        let model = AuditorNavigationModelBuilder::default()
            .overview(overview)
            .claim_explorer(claim_explorer)
            .assessment_queue(assessment_queue)
            .witness_surface(witness_surface)
            .provenance_roots(provenance_roots)
            .gaps(gaps)
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

#[instrument(level = "debug", skip(gaps))]
fn project_gaps(
    gaps: &[LandscapeGap],
    source_record_ids: &[String],
) -> Result<Vec<GapItem>, BelvedereConstructionError> {
    gaps.iter()
        .map(|gap| {
            GapItemBuilder::default()
                .claim_id(gap.claim_id())
                .kind(gap.kind())
                .detail(gap.detail())
                .high_priority(high_priority(gap.kind()))
                .trace(trace(gap.claim_id(), source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("GapItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(kind))]
fn high_priority(kind: LandscapeGapKind) -> bool {
    matches!(
        kind,
        LandscapeGapKind::ReplacementRecommended
            | LandscapeGapKind::RetirementRecommended
            | LandscapeGapKind::TrustedOrOpaqueWitnessSupport
    )
}

#[instrument(level = "debug")]
fn trace(claim_id: &str, source_record_ids: &[String]) -> TraceRef {
    TraceRef::new(claim_id, source_record_ids.to_vec())
}

#[instrument(level = "debug", skip(rubric))]
fn weak_dimensions(rubric: Rubric) -> Vec<String> {
    [
        ("claim_alignment", rubric.claim_alignment()),
        ("assumption_adequacy", rubric.assumption_adequacy()),
        ("model_fidelity", rubric.model_fidelity()),
        ("assertion_strength", rubric.assertion_strength()),
        ("adversarial_coverage", rubric.adversarial_coverage()),
        ("clarity", rubric.clarity()),
    ]
    .into_iter()
    .filter(|(_dimension, score)| *score < 4)
    .map(|(dimension, _score)| dimension.to_owned())
    .collect()
}
