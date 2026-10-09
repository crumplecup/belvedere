//! Landscape and view-model invariant checks.

use std::collections::BTreeMap;

use tracing::instrument;

use super::{BelvedereInvariantCheck, count_value, unique_string_count};

#[instrument(level = "trace", skip(trace_source_counts, source_record_ids))]
pub(crate) fn view_model_trace_invariant_checks(
    trace_source_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let traceable_items = count_value(trace_source_counts, "view_model_traceable_items");
    let items_with_sources =
        count_value(trace_source_counts, "view_model_trace_items_with_sources");
    let source_links = count_value(trace_source_counts, "view_model_trace_source_links");
    let unique_source_records = count_value(
        trace_source_counts,
        "view_model_trace_unique_source_records",
    );
    let input_source_records = unique_string_count(source_record_ids);
    vec![
        BelvedereInvariantCheck::new(
            "view_model_trace_sources_complete".to_owned(),
            traceable_items > 0 && traceable_items == items_with_sources,
            format!(
                "{items_with_sources} of {traceable_items} traceable view-model items carry source links"
            ),
        ),
        BelvedereInvariantCheck::new(
            "view_model_trace_links_preserved".to_owned(),
            source_links >= items_with_sources,
            format!(
                "{source_links} source links recorded across {items_with_sources} sourced view-model items"
            ),
        ),
        BelvedereInvariantCheck::new(
            "view_model_input_sources_reachable".to_owned(),
            input_source_records > 0 && unique_source_records == input_source_records,
            format!(
                "{unique_source_records} of {input_source_records} input source records reachable from view-model traces"
            ),
        ),
    ]
}

#[instrument(level = "trace", skip(counts, item_kind_counts, view_item_ids))]
pub(crate) fn view_model_item_invariant_checks(
    counts: &BTreeMap<String, usize>,
    item_kind_counts: &BTreeMap<String, usize>,
    view_item_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let item_kind_total = item_kind_counts.values().sum::<usize>();
    vec![
        BelvedereInvariantCheck::new(
            "view_model_item_ids_classified".to_owned(),
            item_kind_total == view_item_ids.len(),
            format!(
                "{item_kind_total} classified view-model item IDs recorded for {} outputs",
                view_item_ids.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "view_model_core_counts_match_items".to_owned(),
            count_value(item_kind_counts, "view_model_item_kind_claim")
                == count_value(counts, "claim_cards")
                && count_value(item_kind_counts, "view_model_item_kind_claim_detail")
                    == count_value(counts, "claim_detail_items")
                && count_value(item_kind_counts, "view_model_item_kind_filter_facet")
                    == count_value(counts, "filter_facets")
                && count_value(item_kind_counts, "view_model_item_kind_backend_coverage")
                    == count_value(counts, "backend_coverage_items")
                && count_value(item_kind_counts, "view_model_item_kind_proof_chain")
                    == count_value(counts, "proof_chain_items")
                && count_value(item_kind_counts, "view_model_item_kind_review_task")
                    == count_value(counts, "review_workflow_items")
                && count_value(item_kind_counts, "view_model_item_kind_assessment_history")
                    == count_value(counts, "assessment_history_items")
                && count_value(item_kind_counts, "view_model_item_kind_assessment_queue")
                    == count_value(counts, "assessment_items")
                && count_value(item_kind_counts, "view_model_item_kind_assessment_work")
                    == count_value(counts, "assessment_work_items")
                && count_value(item_kind_counts, "view_model_item_kind_witness")
                    == count_value(counts, "witness_items")
                && count_value(item_kind_counts, "view_model_item_kind_provenance_root")
                    == count_value(counts, "root_items")
                && count_value(item_kind_counts, "view_model_item_kind_gap")
                    == count_value(counts, "gap_items")
                && count_value(item_kind_counts, "view_model_item_kind_source")
                    == count_value(counts, "source_items"),
            format!(
                "claims={}, details={}, facets={}, coverage={}, chains={}, tasks={}, history={}, queue={}, work={}, witnesses={}, roots={}, gaps={}, sources={}",
                count_value(item_kind_counts, "view_model_item_kind_claim"),
                count_value(item_kind_counts, "view_model_item_kind_claim_detail"),
                count_value(item_kind_counts, "view_model_item_kind_filter_facet"),
                count_value(item_kind_counts, "view_model_item_kind_backend_coverage"),
                count_value(item_kind_counts, "view_model_item_kind_proof_chain"),
                count_value(item_kind_counts, "view_model_item_kind_review_task"),
                count_value(item_kind_counts, "view_model_item_kind_assessment_history"),
                count_value(item_kind_counts, "view_model_item_kind_assessment_queue"),
                count_value(item_kind_counts, "view_model_item_kind_assessment_work"),
                count_value(item_kind_counts, "view_model_item_kind_witness"),
                count_value(item_kind_counts, "view_model_item_kind_provenance_root"),
                count_value(item_kind_counts, "view_model_item_kind_gap"),
                count_value(item_kind_counts, "view_model_item_kind_source")
            ),
        ),
    ]
}

#[instrument(level = "trace", skip(source_locator_counts, source_record_ids))]
pub(crate) fn view_model_source_navigation_invariant_checks(
    source_locator_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let located_sources = count_value(source_locator_counts, "source_locator_items");
    let registry_sources = count_value(source_locator_counts, "source_locator_registry_dump");
    let assessment_sources = count_value(source_locator_counts, "source_locator_proof_assessments");
    let input_source_records = source_record_ids.len();
    vec![BelvedereInvariantCheck::new(
        "view_model_source_locators_complete".to_owned(),
        located_sources == input_source_records
            && registry_sources + assessment_sources == located_sources,
        format!(
            "{located_sources} locator hints recorded for {input_source_records} source records"
        ),
    )]
}

#[instrument(level = "trace", skip(counts, item_kind_counts, landscape_item_ids))]
pub(crate) fn landscape_model_invariant_checks(
    counts: &BTreeMap<String, usize>,
    item_kind_counts: &BTreeMap<String, usize>,
    landscape_item_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let item_kind_total = item_kind_counts.values().sum::<usize>();
    let claim_items = count_value(item_kind_counts, "landscape_item_kind_claim");
    let root_items = count_value(item_kind_counts, "landscape_item_kind_root");
    let edge_items = count_value(item_kind_counts, "landscape_item_kind_edge");
    let proof_items = count_value(item_kind_counts, "landscape_item_kind_proof");
    let assessment_items = count_value(item_kind_counts, "landscape_item_kind_assessment");
    let gap_items = count_value(item_kind_counts, "landscape_item_kind_gap");
    vec![
        BelvedereInvariantCheck::new(
            "landscape_item_ids_classified".to_owned(),
            item_kind_total == landscape_item_ids.len(),
            format!(
                "{item_kind_total} classified landscape item IDs recorded for {} outputs",
                landscape_item_ids.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "landscape_core_counts_match_items".to_owned(),
            claim_items == count_value(counts, "claims")
                && root_items == count_value(counts, "roots")
                && edge_items == count_value(counts, "edges")
                && proof_items == count_value(counts, "proofs")
                && assessment_items == count_value(counts, "assessments")
                && gap_items == count_value(counts, "gaps"),
            format!(
                "claims={claim_items}, roots={root_items}, edges={edge_items}, proofs={proof_items}, assessments={assessment_items}, gaps={gap_items}"
            ),
        ),
        BelvedereInvariantCheck::new(
            "landscape_explicit_review_gaps_modeled".to_owned(),
            count_value(counts, "gaps") == 0 || gap_items == count_value(counts, "gaps"),
            format!(
                "{} gap outputs recorded for {} modeled gaps",
                gap_items,
                count_value(counts, "gaps")
            ),
        ),
    ]
}
