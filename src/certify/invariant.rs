//! Invariant check evidence embedded in stage certificates.

use std::collections::{BTreeMap, BTreeSet};

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

/// Named invariant result included in a stage certificate.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct BelvedereInvariantCheck {
    name: String,
    #[getter(skip)]
    passed: bool,
    detail: String,
}

impl BelvedereInvariantCheck {
    /// Whether the invariant passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.passed
    }
}

/// Evidence used to derive invariant checks for a run certificate.
#[derive(Debug, Clone, Copy, new)]
pub(crate) struct RunInvariantEvidence<'evidence> {
    component_versions: &'evidence BTreeMap<String, String>,
    invocation_surface: &'evidence str,
    invocation_arguments: &'evidence [String],
    pipeline_claim_ids: &'evidence [String],
    input_artifacts: RunInputArtifactEvidence<'evidence>,
    stage_certificates: RunStageCertificateEvidence<'evidence>,
}

/// Evidence for run input artifact accounting.
#[derive(Debug, Clone, Copy, new)]
pub(crate) struct RunInputArtifactEvidence<'evidence> {
    ids: &'evidence [String],
    digests: &'evidence [String],
}

/// Evidence for stage certificate digest accounting.
#[derive(Debug, Clone, Copy, new)]
pub(crate) struct RunStageCertificateEvidence<'evidence> {
    ids: &'evidence [String],
    digests: &'evidence [String],
    digest_matches: bool,
}

#[instrument(level = "trace", skip(requires, ensures, input_ids, counts))]
pub(crate) fn stage_invariant_checks(
    requires: &[String],
    ensures: &[String],
    input_ids: &[String],
    counts: &BTreeMap<String, usize>,
    warning_count: usize,
) -> Vec<BelvedereInvariantCheck> {
    vec![
        BelvedereInvariantCheck::new(
            "contract_obligations_named".to_owned(),
            !requires.is_empty() && !ensures.is_empty(),
            format!(
                "{} requires clauses and {} ensures clauses recorded",
                requires.len(),
                ensures.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "input_evidence_accounted".to_owned(),
            !input_ids.is_empty(),
            format!("{} input evidence IDs recorded", input_ids.len()),
        ),
        BelvedereInvariantCheck::new(
            "stage_counts_recorded".to_owned(),
            !counts.is_empty(),
            format!("{} count dimensions recorded", counts.len()),
        ),
        BelvedereInvariantCheck::new(
            "warning_count_consistent".to_owned(),
            warning_count_is_consistent(counts, warning_count),
            warning_count_detail(counts, warning_count),
        ),
    ]
}

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

#[instrument(
    level = "trace",
    skip(trace_source_counts, source_record_ids, trace_source_record_ids)
)]
pub(crate) fn diff_trace_invariant_checks(
    diff_item_count: usize,
    trace_source_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
    trace_source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let traceable_items = count_value(trace_source_counts, "diff_traceable_items");
    let items_with_sources = count_value(trace_source_counts, "diff_trace_items_with_sources");
    let source_links = count_value(trace_source_counts, "diff_trace_source_links");
    let unique_source_records =
        count_value(trace_source_counts, "diff_trace_unique_source_records");
    let input_source_records = source_record_ids.iter().collect::<BTreeSet<_>>();
    let trace_sources_are_admissible = trace_source_record_ids
        .iter()
        .all(|source_record_id| input_source_records.contains(source_record_id));
    vec![
        BelvedereInvariantCheck::new(
            "landscape_diff_items_sourced".to_owned(),
            diff_item_count > 0
                && traceable_items == diff_item_count
                && items_with_sources == diff_item_count,
            format!("{items_with_sources} of {diff_item_count} diff items carry source links"),
        ),
        BelvedereInvariantCheck::new(
            "landscape_diff_trace_links_preserved".to_owned(),
            source_links >= items_with_sources,
            format!(
                "{source_links} source links recorded across {items_with_sources} sourced diff items"
            ),
        ),
        BelvedereInvariantCheck::new(
            "landscape_diff_trace_sources_admissible".to_owned(),
            trace_sources_are_admissible && unique_source_records == trace_source_record_ids.len(),
            format!(
                "{} unique diff trace sources checked against {} compared snapshot source records",
                trace_source_record_ids.len(),
                input_source_records.len()
            ),
        ),
    ]
}

#[instrument(
    level = "trace",
    skip(section_trace_counts, trace_source_counts, source_record_ids)
)]
pub(crate) fn ui_ir_trace_invariant_checks(
    trace_widget_count: usize,
    trace_ref_count: usize,
    section_trace_counts: &BTreeMap<String, usize>,
    trace_source_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let unassigned_trace_widgets = count_value(
        section_trace_counts,
        "ui_ir_section_unassigned_trace_widgets",
    );
    let sections_without_source_widgets = count_value(
        section_trace_counts,
        "ui_ir_sections_without_source_widgets",
    );
    let section_source_links_total = count_value(
        section_trace_counts,
        "ui_ir_section_trace_source_links_total",
    );
    let widgets_with_sources = count_value(trace_source_counts, "trace_widgets_with_sources");
    let source_links = count_value(trace_source_counts, "trace_source_links");
    let unique_source_records = count_value(trace_source_counts, "trace_unique_source_records");
    let input_source_records = unique_string_count(source_record_ids);
    vec![
        BelvedereInvariantCheck::new(
            "ui_ir_trace_widgets_sourced".to_owned(),
            trace_widget_count > 0 && widgets_with_sources == trace_widget_count,
            format!(
                "{widgets_with_sources} of {trace_widget_count} trace widgets carry source links"
            ),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_trace_refs_preserved".to_owned(),
            source_links == trace_ref_count,
            format!("{source_links} source links recorded for {trace_ref_count} trace references"),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_trace_widgets_sectioned".to_owned(),
            unassigned_trace_widgets == 0 && section_source_links_total == trace_ref_count,
            format!(
                "{unassigned_trace_widgets} unassigned trace widgets; {section_source_links_total} section source links recorded for {trace_ref_count} trace references"
            ),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_sections_traceable".to_owned(),
            trace_widget_count > 0 && sections_without_source_widgets == 0,
            format!("{sections_without_source_widgets} UI IR sections lack sourced widgets"),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_input_sources_reachable".to_owned(),
            input_source_records > 0 && unique_source_records == input_source_records,
            format!(
                "{unique_source_records} of {input_source_records} input source records reachable from UI IR traces"
            ),
        ),
    ]
}

#[instrument(level = "trace", skip(evidence))]
pub(crate) fn run_invariant_checks(
    evidence: RunInvariantEvidence<'_>,
) -> Vec<BelvedereInvariantCheck> {
    vec![
        BelvedereInvariantCheck::new(
            "run_component_versions_recorded".to_owned(),
            evidence
                .component_versions
                .values()
                .all(|version| !version.trim().is_empty() && version != "unknown"),
            format!(
                "{} component versions recorded",
                evidence.component_versions.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "run_invocation_recorded".to_owned(),
            !evidence.invocation_surface.is_empty(),
            format!(
                "surface={} argument_count={}",
                evidence.invocation_surface,
                evidence.invocation_arguments.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "run_input_artifacts_accounted".to_owned(),
            !evidence.input_artifacts.ids.is_empty()
                && evidence.input_artifacts.ids.len() == evidence.input_artifacts.digests.len(),
            format!(
                "{} input IDs and {} input digests recorded",
                evidence.input_artifacts.ids.len(),
                evidence.input_artifacts.digests.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "run_pipeline_claims_recorded".to_owned(),
            !evidence.pipeline_claim_ids.is_empty()
                && evidence.pipeline_claim_ids.len()
                    == unique_string_count(evidence.pipeline_claim_ids),
            format!(
                "{} unique pipeline claim records emitted",
                evidence.pipeline_claim_ids.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "run_stage_certificate_digests_match".to_owned(),
            !evidence.stage_certificates.ids.is_empty()
                && evidence.stage_certificates.ids.len()
                    == evidence.stage_certificates.digests.len()
                && evidence.stage_certificates.digest_matches,
            format!(
                "{} stage IDs and {} stage digests recorded",
                evidence.stage_certificates.ids.len(),
                evidence.stage_certificates.digests.len()
            ),
        ),
    ]
}

#[instrument(level = "trace", skip(counts))]
fn warning_count_is_consistent(counts: &BTreeMap<String, usize>, warning_count: usize) -> bool {
    counts
        .get("warnings")
        .is_none_or(|count| *count == warning_count)
}

#[instrument(level = "trace", skip(counts))]
fn warning_count_detail(counts: &BTreeMap<String, usize>, warning_count: usize) -> String {
    let recorded = counts.get("warnings").copied();
    format!(
        "warning_count={} count_dimension={}",
        warning_count,
        recorded
            .map(|count| count.to_string())
            .unwrap_or_else(|| "not_recorded".to_owned())
    )
}

#[instrument(level = "trace", skip(counts))]
fn count_value(counts: &BTreeMap<String, usize>, key: &str) -> usize {
    counts.get(key).copied().unwrap_or_default()
}

#[instrument(level = "trace", skip(values))]
fn unique_string_count(values: &[String]) -> usize {
    values.iter().collect::<BTreeSet<_>>().len()
}
