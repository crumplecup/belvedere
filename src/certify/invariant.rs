//! Invariant check evidence embedded in stage certificates.

use std::collections::BTreeMap;

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
