//! Artifact entries indexed by a Belvedere artifact manifest.

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{BelvederePipelineStage, BelvedereRunCertificate, hex_digest, update_field};

const MANIFEST_OUTPUT_ARTIFACT_SCHEMA_ID: &str = "belvedere.manifest.output_artifact.v1";
const MANIFEST_OUTPUT_ARTIFACT_ID_PREFIX: &str = "belvedere.manifest.output_artifact";

/// Input artifact indexed by a Belvedere artifact manifest.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereManifestInputArtifact {
    role: String,
    artifact_id: String,
    digest: String,
}

/// Stage certificate indexed by a Belvedere artifact manifest.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereManifestStageCertificate {
    role: String,
    stage: String,
    certificate_id: String,
    digest: String,
    contract: String,
    output_count: usize,
}

impl BelvedereManifestStageCertificate {
    #[instrument(level = "trace", skip(role, stage, certificate_id, digest, contract))]
    fn new(
        role: impl Into<String>,
        stage: BelvederePipelineStage,
        certificate_id: impl Into<String>,
        digest: impl Into<String>,
        contract: impl Into<String>,
        output_count: usize,
    ) -> Self {
        Self {
            role: role.into(),
            stage: stage.as_str().to_owned(),
            certificate_id: certificate_id.into(),
            digest: digest.into(),
            contract: contract.into(),
            output_count,
        }
    }
}

/// Requestable output artifact indexed by a Belvedere artifact manifest.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereManifestOutputArtifact {
    role: String,
    artifact_kind: String,
    artifact_id: String,
    artifact_digest: String,
    evidence_id: String,
    evidence_digest: String,
    source_stage_role: String,
    source_stage_certificate_id: String,
    source_stage_certificate_digest: String,
    item_count: usize,
}

impl BelvedereManifestOutputArtifact {
    #[instrument(level = "trace", skip(role, artifact_kind, evidence, source_stage))]
    fn new(
        role: impl Into<String>,
        artifact_kind: impl Into<String>,
        evidence: ManifestOutputEvidence,
        source_stage: ManifestOutputSourceStage,
        item_count: usize,
    ) -> Self {
        let role = role.into();
        let artifact_kind = artifact_kind.into();
        let artifact_digest = manifest_output_artifact_digest(OutputArtifactDigestEvidence {
            role: &role,
            artifact_kind: &artifact_kind,
            evidence_id: &evidence.id,
            evidence_digest: &evidence.digest,
            source_stage_role: &source_stage.role,
            source_stage_certificate_id: &source_stage.certificate_id,
            source_stage_certificate_digest: &source_stage.certificate_digest,
            item_count,
        });
        let artifact_id = format!("{MANIFEST_OUTPUT_ARTIFACT_ID_PREFIX}:{artifact_digest}");
        Self {
            role,
            artifact_kind,
            artifact_id,
            artifact_digest,
            evidence_id: evidence.id,
            evidence_digest: evidence.digest,
            source_stage_role: source_stage.role,
            source_stage_certificate_id: source_stage.certificate_id,
            source_stage_certificate_digest: source_stage.certificate_digest,
            item_count,
        }
    }
}

#[derive(Debug, Clone, new)]
struct ManifestOutputEvidence {
    #[new(into)]
    id: String,
    #[new(into)]
    digest: String,
}

#[derive(Debug, Clone, new)]
struct ManifestOutputSourceStage {
    #[new(into)]
    role: String,
    #[new(into)]
    certificate_id: String,
    #[new(into)]
    certificate_digest: String,
}

impl ManifestOutputSourceStage {
    #[instrument(level = "trace", skip(stage_certificate))]
    fn for_stage_certificate(stage_certificate: &BelvedereManifestStageCertificate) -> Self {
        Self::new(
            stage_certificate.role(),
            stage_certificate.certificate_id(),
            stage_certificate.digest(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct OutputArtifactDigestEvidence<'evidence> {
    role: &'evidence str,
    artifact_kind: &'evidence str,
    evidence_id: &'evidence str,
    evidence_digest: &'evidence str,
    source_stage_role: &'evidence str,
    source_stage_certificate_id: &'evidence str,
    source_stage_certificate_digest: &'evidence str,
    item_count: usize,
}

#[instrument(level = "trace", skip(evidence))]
fn manifest_output_artifact_digest(evidence: OutputArtifactDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, MANIFEST_OUTPUT_ARTIFACT_SCHEMA_ID);
    update_field(&mut hasher, evidence.role);
    update_field(&mut hasher, evidence.artifact_kind);
    update_field(&mut hasher, evidence.evidence_id);
    update_field(&mut hasher, evidence.evidence_digest);
    update_field(&mut hasher, evidence.source_stage_role);
    update_field(&mut hasher, evidence.source_stage_certificate_id);
    update_field(&mut hasher, evidence.source_stage_certificate_digest);
    update_field(&mut hasher, &evidence.item_count.to_string());
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(certificate))]
pub(super) fn manifest_inputs(
    certificate: &BelvedereRunCertificate,
) -> Vec<BelvedereManifestInputArtifact> {
    certificate
        .input_artifact_ids()
        .iter()
        .zip(certificate.input_artifact_digests())
        .map(|(artifact_id, digest)| {
            let role = artifact_id
                .split_once(':')
                .map_or(artifact_id.as_str(), |(role, _rest)| role);
            BelvedereManifestInputArtifact {
                role: role.to_owned(),
                artifact_id: artifact_id.to_owned(),
                digest: digest.to_owned(),
            }
        })
        .collect()
}

#[instrument(level = "trace", skip(certificate, roles))]
pub(super) fn manifest_stage_certificates(
    certificate: &BelvedereRunCertificate,
    roles: &[&str],
) -> Vec<BelvedereManifestStageCertificate> {
    certificate
        .stage_certificates()
        .iter()
        .enumerate()
        .map(|(index, stage_certificate)| {
            BelvedereManifestStageCertificate::new(
                roles.get(index).copied().unwrap_or("stage"),
                stage_certificate.stage(),
                stage_certificate.id(),
                stage_certificate.digest(),
                stage_certificate.contract(),
                stage_certificate.output_ids().len(),
            )
        })
        .collect()
}

#[instrument(level = "debug")]
pub(super) fn normal_stage_roles() -> [&'static str; 5] {
    [
        "registry_ingestion",
        "assessment_ingestion",
        "landscape_modeling",
        "auditor_navigation",
        "ui_ir_projection",
    ]
}

#[instrument(level = "debug")]
pub(super) fn diff_stage_roles() -> [&'static str; 13] {
    [
        "previous_registry_ingestion",
        "previous_assessment_ingestion",
        "previous_landscape_modeling",
        "previous_auditor_navigation",
        "previous_ui_ir_projection",
        "current_registry_ingestion",
        "current_assessment_ingestion",
        "current_landscape_modeling",
        "current_auditor_navigation",
        "current_ui_ir_projection",
        "landscape_diff",
        "landscape_diff_review",
        "landscape_diff_review_ui_ir_projection",
    ]
}

#[instrument(level = "trace", skip(certificate))]
pub(super) fn run_certificate_output(
    certificate: &BelvedereRunCertificate,
) -> BelvedereManifestOutputArtifact {
    BelvedereManifestOutputArtifact::new(
        "run_certificate",
        "run_certificate",
        ManifestOutputEvidence::new(certificate.id(), certificate.digest()),
        ManifestOutputSourceStage::new("run_certificate", certificate.id(), certificate.digest()),
        certificate.stage_certificates().len(),
    )
}

#[instrument(level = "trace", skip(role, artifact_kind, stage_certificates))]
pub(super) fn output_artifact(
    role: &str,
    artifact_kind: &str,
    evidence_id: &str,
    evidence_digest: &str,
    source_stage_index: usize,
    item_count: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> BelvedereManifestOutputArtifact {
    if let Some(stage_certificate) = stage_certificates.get(source_stage_index) {
        return BelvedereManifestOutputArtifact::new(
            role,
            artifact_kind,
            ManifestOutputEvidence::new(evidence_id, evidence_digest),
            ManifestOutputSourceStage::for_stage_certificate(stage_certificate),
            item_count,
        );
    }

    BelvedereManifestOutputArtifact::new(
        role,
        artifact_kind,
        ManifestOutputEvidence::new(evidence_id, evidence_digest),
        ManifestOutputSourceStage::new("unknown", "unknown", "unknown"),
        item_count,
    )
}

#[instrument(level = "trace", skip(role, artifact_kind, stage_certificates))]
pub(super) fn stage_certificate_output(
    role: &str,
    artifact_kind: &str,
    source_stage_index: usize,
    item_count: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> BelvedereManifestOutputArtifact {
    if let Some(stage_certificate) = stage_certificates.get(source_stage_index) {
        return BelvedereManifestOutputArtifact::new(
            role,
            artifact_kind,
            ManifestOutputEvidence::new(
                stage_certificate.certificate_id(),
                stage_certificate.digest(),
            ),
            ManifestOutputSourceStage::for_stage_certificate(stage_certificate),
            item_count,
        );
    }

    BelvedereManifestOutputArtifact::new(
        role,
        artifact_kind,
        ManifestOutputEvidence::new("unknown", "unknown"),
        ManifestOutputSourceStage::new("unknown", "unknown", "unknown"),
        item_count,
    )
}
