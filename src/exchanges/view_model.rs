//! Auditor navigation view-model exchange implementations.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    certify::{ViewModelReceipt, ViewModelReceiptBuilder, ViewModelReceiptCountsBuilder},
    contracts::{
        AssessmentQueueItem, AssessmentRecencySummary, AssessmentWorkItem, AuditorFilterFacet,
        AuditorNavigationContract, AuditorNavigationModel, AuditorNavigationModelBuilder,
        BelvedereContract, GapItem, LandscapeAssessment, OverviewView, ProofChainItem,
        ProvenanceRootItem, ReviewWorkflowItem, SourceNavigationItem, TraceRef, ViewModelInput,
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
mod traces;
mod witness;
mod workflow;

use assessments::{project_assessment_queue, project_assessment_work};
use claims::{project_claim_explorer, project_proof_chains};
use coverage::project_backend_coverage;
use facets::project_filter_facets;
use gaps::project_gaps;
use sources::project_source_navigation;
use traces::trace;
use witness::project_witness_surface;
use workflow::project_review_workflow;

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
        let review_workflow = project_review_workflow(&proof_chains, &gaps)?;
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
        let receipt_lane_counts = lane_counts(&assessment_queue, &assessment_work, &gaps);
        let receipt_facet_group_counts = facet_group_counts(&filter_facets);
        let receipt_source_artifact_counts = source_artifact_counts(&source_navigation);
        let receipt_source_kind_counts = source_kind_counts(&source_navigation);
        let receipt_proof_chain_topology_counts = proof_chain_topology_counts(&proof_chains);
        let receipt_provenance_root_kind_counts = provenance_root_kind_counts(&provenance_roots);
        let receipt_review_workflow_counts =
            review_workflow_counts(&proof_chains, &review_workflow, &assessment_work, &gaps);
        let receipt_trace_kind_counts = trace_kind_counts(&[
            ("filter_facet", filter_facets.len()),
            ("backend_coverage_item", backend_coverage.len()),
            ("claim_card", claim_explorer.len()),
            ("proof_chain_item", proof_chains.len()),
            ("review_workflow_item", review_workflow.len()),
            ("assessment_queue_item", assessment_queue.len()),
            ("assessment_work_item", assessment_work.len()),
            ("witness_item", witness_surface.len()),
            ("provenance_root_item", provenance_roots.len()),
            ("gap_item", gaps.len()),
            ("source_navigation_item", source_navigation.len()),
        ]);
        let receipt_trace_source_counts = trace_source_counts(
            filter_facets
                .iter()
                .map(|item| item.trace())
                .chain(backend_coverage.iter().map(|item| item.trace()))
                .chain(claim_explorer.iter().map(|item| item.trace()))
                .chain(proof_chains.iter().map(|item| item.trace()))
                .chain(review_workflow.iter().map(|item| item.trace()))
                .chain(assessment_queue.iter().map(|item| item.trace()))
                .chain(assessment_work.iter().map(|item| item.trace()))
                .chain(witness_surface.iter().map(|item| item.trace()))
                .chain(provenance_roots.iter().map(|item| item.trace()))
                .chain(gaps.iter().map(|item| item.trace()))
                .chain(source_navigation.iter().map(|item| item.trace())),
        );

        let counts = ViewModelReceiptCountsBuilder::default()
            .claim_cards(landscape.claims().len())
            .filter_facets(filter_facets.len())
            .backend_coverage_items(backend_coverage.len())
            .proof_chain_items(proof_chains.len())
            .review_workflow_items(review_workflow.len())
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
            .review_workflow(review_workflow)
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
            .lane_counts(receipt_lane_counts)
            .facet_group_counts(receipt_facet_group_counts)
            .source_artifact_counts(receipt_source_artifact_counts)
            .source_kind_counts(receipt_source_kind_counts)
            .proof_chain_topology_counts(receipt_proof_chain_topology_counts)
            .provenance_root_kind_counts(receipt_provenance_root_kind_counts)
            .review_workflow_counts(receipt_review_workflow_counts)
            .trace_kind_counts(receipt_trace_kind_counts)
            .trace_source_counts(receipt_trace_source_counts)
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

#[instrument(level = "debug", skip(assessment_queue, assessment_work, gaps))]
fn lane_counts(
    assessment_queue: &[AssessmentQueueItem],
    assessment_work: &[AssessmentWorkItem],
    gaps: &[GapItem],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in assessment_queue {
        increment_lane_count(&mut counts, "assessment_queue_lane", item.queue_lane());
    }
    for item in assessment_work {
        increment_lane_count(&mut counts, "assessment_work_lane", item.queue_lane());
    }
    for item in gaps {
        increment_lane_count(&mut counts, "gap_lane", item.queue_lane());
    }
    counts
}

#[instrument(level = "trace", skip(counts))]
fn increment_lane_count(counts: &mut BTreeMap<String, usize>, prefix: &str, lane: &str) {
    *counts.entry(format!("{prefix}_{lane}")).or_default() += 1;
}

#[instrument(level = "debug", skip(filter_facets))]
fn facet_group_counts(filter_facets: &[AuditorFilterFacet]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for facet in filter_facets {
        *counts
            .entry(format!("facet_group_{}", facet.group()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(source_navigation))]
fn source_artifact_counts(source_navigation: &[SourceNavigationItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in source_navigation {
        *counts
            .entry(format!("source_artifact_{}", item.artifact_kind().as_str()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(source_navigation))]
fn source_kind_counts(source_navigation: &[SourceNavigationItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in source_navigation {
        *counts
            .entry(format!("source_kind_{}", item.source_kind()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(families))]
fn trace_kind_counts(families: &[(&str, usize)]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for (kind, count) in families {
        counts.insert(format!("view_model_trace_kind_{kind}"), *count);
    }
    counts
}

#[instrument(level = "debug", skip(traces))]
fn trace_source_counts<'trace>(
    traces: impl IntoIterator<Item = &'trace TraceRef>,
) -> BTreeMap<String, usize> {
    let mut traceable_items = 0;
    let mut items_with_sources = 0;
    let mut source_links = 0;
    let mut unique_source_records = BTreeSet::new();
    for trace in traces {
        traceable_items += 1;
        if !trace.source_record_ids().is_empty() {
            items_with_sources += 1;
        }
        source_links += trace.source_record_ids().len();
        unique_source_records.extend(trace.source_record_ids().iter().cloned());
    }
    BTreeMap::from([
        ("view_model_traceable_items".to_owned(), traceable_items),
        (
            "view_model_trace_items_with_sources".to_owned(),
            items_with_sources,
        ),
        ("view_model_trace_source_links".to_owned(), source_links),
        (
            "view_model_trace_unique_source_records".to_owned(),
            unique_source_records.len(),
        ),
    ])
}

#[instrument(level = "debug", skip(provenance_roots))]
fn provenance_root_kind_counts(provenance_roots: &[ProvenanceRootItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in provenance_roots {
        *counts
            .entry(format!("provenance_root_kind_{}", item.root_kind()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(proof_chains))]
fn proof_chain_topology_counts(proof_chains: &[ProofChainItem]) -> BTreeMap<String, usize> {
    let mut total_transitive_claims = 0;
    let mut max_depth = 0;
    let mut branching_claims = 0;
    let mut chain_gaps = 0;
    for item in proof_chains {
        let topology = item.topology();
        total_transitive_claims += topology.transitive_claim_count();
        max_depth = max_depth.max(topology.max_depth());
        branching_claims += topology.branching_claim_count();
        chain_gaps += topology.chain_gap_count();
    }
    BTreeMap::from([
        (
            "proof_chain_total_transitive_claims".to_owned(),
            total_transitive_claims,
        ),
        ("proof_chain_max_depth".to_owned(), max_depth),
        ("proof_chain_branching_claims".to_owned(), branching_claims),
        ("proof_chain_chain_gaps".to_owned(), chain_gaps),
    ])
}

#[instrument(
    level = "debug",
    skip(proof_chains, review_workflow, assessment_work, gaps)
)]
fn review_workflow_counts(
    proof_chains: &[ProofChainItem],
    review_workflow: &[ReviewWorkflowItem],
    assessment_work: &[AssessmentWorkItem],
    gaps: &[GapItem],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in proof_chains {
        *counts
            .entry(format!(
                "review_workflow_proof_chain_state_{}",
                item.review_state()
            ))
            .or_default() += 1;
    }

    counts.insert(
        "review_workflow_task_total".to_owned(),
        review_workflow.len(),
    );
    for item in review_workflow {
        *counts
            .entry(format!(
                "review_workflow_task_priority_{}",
                item.priority().label()
            ))
            .or_default() += 1;
        *counts
            .entry(format!("review_workflow_task_lane_{}", item.lane()))
            .or_default() += 1;
    }

    counts.insert(
        "review_workflow_assessment_work_total".to_owned(),
        assessment_work.len(),
    );
    counts.insert(
        "review_workflow_assessment_work_high_priority".to_owned(),
        assessment_work
            .iter()
            .filter(|item| item.high_priority())
            .count(),
    );
    counts.insert(
        "review_workflow_assessment_work_standard_priority".to_owned(),
        assessment_work
            .iter()
            .filter(|item| !item.high_priority())
            .count(),
    );

    for item in gaps {
        *counts
            .entry(format!(
                "review_workflow_gap_severity_{}",
                item.severity().label()
            ))
            .or_default() += 1;
    }
    counts.insert(
        "review_workflow_gap_high_priority".to_owned(),
        gaps.iter().filter(|item| item.high_priority()).count(),
    );
    counts
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
