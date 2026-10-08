//! Run-level invariant check evidence.

use std::collections::BTreeMap;

use derive_new::new;
use tracing::instrument;

use super::{BelvedereInvariantCheck, unique_string_count};

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
    stage_kind_counts: &'evidence BTreeMap<String, usize>,
    digest_matches: bool,
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
        BelvedereInvariantCheck::new(
            "run_stage_certificate_kinds_classified".to_owned(),
            !evidence.stage_certificates.stage_kind_counts.is_empty()
                && evidence
                    .stage_certificates
                    .stage_kind_counts
                    .values()
                    .all(|count| *count > 0)
                && evidence.stage_certificates.ids.len()
                    == evidence
                        .stage_certificates
                        .stage_kind_counts
                        .values()
                        .sum::<usize>(),
            format!(
                "{} stage certificates classified into {} stage kinds",
                evidence.stage_certificates.ids.len(),
                evidence.stage_certificates.stage_kind_counts.len()
            ),
        ),
    ]
}
