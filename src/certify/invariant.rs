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

/// Evidence used to derive invariant checks for a run certificate.
#[derive(Debug, Clone, Copy, new)]
pub(crate) struct RunInvariantEvidence<'evidence> {
    component_versions: &'evidence BTreeMap<String, String>,
    invocation_surface: &'evidence str,
    invocation_arguments: &'evidence [String],
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
