//! Invariant check evidence embedded in stage certificates.

use std::collections::{BTreeMap, BTreeSet};

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

mod diff;
mod model;
mod run;
mod ui_ir;

pub(crate) use diff::{
    diff_item_invariant_checks, diff_review_invariant_checks, diff_trace_invariant_checks,
};
pub(crate) use model::{
    landscape_model_invariant_checks, view_model_item_invariant_checks,
    view_model_source_navigation_invariant_checks, view_model_trace_invariant_checks,
};
pub(crate) use run::{
    RunInputArtifactEvidence, RunInvariantEvidence, RunStageCertificateEvidence,
    run_invariant_checks,
};
pub(crate) use ui_ir::{
    ui_ir_output_invariant_checks, ui_ir_source_locator_invariant_checks,
    ui_ir_trace_invariant_checks,
};

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

#[instrument(
    level = "trace",
    skip(decoded_counts, source_record_kind_counts, source_record_ids)
)]
pub(crate) fn ingestion_record_invariant_checks(
    decoded_counts: &BTreeMap<String, usize>,
    source_record_kind_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let decoded_total = decoded_counts.values().sum::<usize>();
    let source_record_kind_total = source_record_kind_counts.values().sum::<usize>();
    vec![
        BelvedereInvariantCheck::new(
            "ingestion_source_record_ids_classified".to_owned(),
            source_record_kind_total == source_record_ids.len(),
            format!(
                "{source_record_kind_total} classified ingestion source-record IDs recorded for {} outputs",
                source_record_ids.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "ingestion_decoded_counts_match_source_records".to_owned(),
            decoded_total == source_record_ids.len(),
            format!(
                "{decoded_total} decoded records accounted for by {} source-record IDs",
                source_record_ids.len()
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
pub(super) fn count_value(counts: &BTreeMap<String, usize>, key: &str) -> usize {
    counts.get(key).copied().unwrap_or_default()
}

#[instrument(level = "trace", skip(values))]
pub(super) fn unique_string_count(values: &[String]) -> usize {
    values.iter().collect::<BTreeSet<_>>().len()
}
