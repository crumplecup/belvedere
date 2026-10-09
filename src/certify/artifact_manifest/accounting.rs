//! Manifest accounting and invariant checks.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::certify::BelvedereInvariantCheck;

use super::{
    BelvedereManifestInputArtifact, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate,
};

#[instrument(level = "trace", skip(input_artifacts))]
pub(super) fn input_role_counts(
    input_artifacts: &[BelvedereManifestInputArtifact],
) -> BTreeMap<String, usize> {
    count_by(input_artifacts.iter().map(|artifact| {
        format!(
            "manifest_input_role_{}",
            normalized_count_key(artifact.role())
        )
    }))
}

#[instrument(level = "trace", skip(stage_certificates))]
pub(super) fn stage_role_counts(
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> BTreeMap<String, usize> {
    count_by(stage_certificates.iter().map(|certificate| {
        format!(
            "manifest_stage_role_{}",
            normalized_count_key(certificate.role())
        )
    }))
}

#[instrument(level = "trace", skip(stage_certificates))]
pub(super) fn stage_kind_counts(
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> BTreeMap<String, usize> {
    count_by(stage_certificates.iter().map(|certificate| {
        format!(
            "manifest_stage_kind_{}",
            normalized_count_key(certificate.stage())
        )
    }))
}

#[instrument(level = "trace", skip(output_artifacts))]
pub(super) fn output_role_counts(
    output_artifacts: &[BelvedereManifestOutputArtifact],
) -> BTreeMap<String, usize> {
    count_by(output_artifacts.iter().map(|artifact| {
        format!(
            "manifest_output_role_{}",
            normalized_count_key(artifact.role())
        )
    }))
}

#[instrument(level = "trace", skip(output_artifacts))]
pub(super) fn output_kind_counts(
    output_artifacts: &[BelvedereManifestOutputArtifact],
) -> BTreeMap<String, usize> {
    count_by(output_artifacts.iter().map(|artifact| {
        format!(
            "manifest_output_kind_{}",
            normalized_count_key(artifact.artifact_kind())
        )
    }))
}

#[instrument(level = "trace", skip(output_artifacts))]
pub(super) fn output_source_stage_counts(
    output_artifacts: &[BelvedereManifestOutputArtifact],
) -> BTreeMap<String, usize> {
    count_by(output_artifacts.iter().map(|artifact| {
        format!(
            "manifest_output_source_stage_{}",
            normalized_count_key(artifact.source_stage_role())
        )
    }))
}

#[instrument(
    level = "trace",
    skip(counts, input_artifacts, stage_certificates, output_artifacts)
)]
pub(super) fn manifest_invariant_checks(
    counts: &BTreeMap<String, usize>,
    input_artifacts: &[BelvedereManifestInputArtifact],
    stage_certificates: &[BelvedereManifestStageCertificate],
    output_artifacts: &[BelvedereManifestOutputArtifact],
) -> Vec<BelvedereInvariantCheck> {
    vec![
        BelvedereInvariantCheck::new(
            "artifact_manifest_counts_match_entries".to_owned(),
            counts.get("input_artifacts") == Some(&input_artifacts.len())
                && counts.get("stage_certificates") == Some(&stage_certificates.len())
                && counts.get("output_artifacts") == Some(&output_artifacts.len()),
            format!(
                "{} inputs, {} stage certificates, {} outputs indexed",
                input_artifacts.len(),
                stage_certificates.len(),
                output_artifacts.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "artifact_manifest_roles_unique".to_owned(),
            unique_roles(
                input_artifacts
                    .iter()
                    .map(BelvedereManifestInputArtifact::role),
            ) && unique_roles(
                stage_certificates
                    .iter()
                    .map(BelvedereManifestStageCertificate::role),
            ) && unique_roles(
                output_artifacts
                    .iter()
                    .map(BelvedereManifestOutputArtifact::role),
            ),
            format!(
                "{} input roles, {} stage roles, {} output roles indexed",
                input_artifacts.len(),
                stage_certificates.len(),
                output_artifacts.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "artifact_manifest_output_sources_accounted".to_owned(),
            output_sources_accounted(stage_certificates, output_artifacts),
            format!("{} output source links checked", output_artifacts.len()),
        ),
        BelvedereInvariantCheck::new(
            "artifact_manifest_output_artifacts_identified".to_owned(),
            output_artifacts_identified(output_artifacts),
            format!(
                "{} output artifact identities checked",
                output_artifacts.len()
            ),
        ),
    ]
}

#[instrument(level = "debug", skip(keys))]
fn count_by(keys: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for key in keys {
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

#[instrument(level = "debug", skip(roles))]
fn unique_roles<'role>(mut roles: impl Iterator<Item = &'role String>) -> bool {
    let mut seen = BTreeSet::new();
    roles.all(|role| !role.is_empty() && seen.insert(role))
}

#[instrument(level = "debug", skip(stage_certificates, output_artifacts))]
fn output_sources_accounted(
    stage_certificates: &[BelvedereManifestStageCertificate],
    output_artifacts: &[BelvedereManifestOutputArtifact],
) -> bool {
    output_artifacts.iter().all(|artifact| {
        if artifact.source_stage_role() == "run_certificate" {
            return artifact
                .source_stage_certificate_id()
                .starts_with("belvedere.run:")
                && !artifact.source_stage_certificate_digest().is_empty();
        }
        stage_certificates.iter().any(|certificate| {
            artifact.source_stage_role() == certificate.role()
                && artifact.source_stage_certificate_id() == certificate.certificate_id()
                && artifact.source_stage_certificate_digest() == certificate.digest()
        })
    })
}

#[instrument(level = "debug", skip(output_artifacts))]
fn output_artifacts_identified(output_artifacts: &[BelvedereManifestOutputArtifact]) -> bool {
    output_artifacts.iter().all(|artifact| {
        artifact
            .artifact_id()
            .starts_with("belvedere.manifest.output_artifact:")
            && artifact.artifact_id()
                == &format!(
                    "belvedere.manifest.output_artifact:{}",
                    artifact.artifact_digest()
                )
    })
}

#[instrument(level = "debug")]
fn normalized_count_key(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}
