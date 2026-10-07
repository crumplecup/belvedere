//! Artifact entries indexed by a Belvedere artifact manifest.

use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

use crate::certify::{BelvederePipelineStage, BelvedereRunCertificate};

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
    source_stage_role: String,
    source_stage_certificate_id: String,
    source_stage_certificate_digest: String,
    item_count: usize,
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
pub(super) fn diff_stage_roles() -> [&'static str; 11] {
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
    ]
}

#[instrument(level = "trace", skip(certificate))]
pub(super) fn run_certificate_output(
    certificate: &BelvedereRunCertificate,
) -> BelvedereManifestOutputArtifact {
    BelvedereManifestOutputArtifact {
        role: "run_certificate".to_owned(),
        artifact_kind: "run_certificate".to_owned(),
        source_stage_role: "run_certificate".to_owned(),
        source_stage_certificate_id: certificate.id().to_owned(),
        source_stage_certificate_digest: certificate.digest().to_owned(),
        item_count: certificate.stage_certificates().len(),
    }
}

#[instrument(level = "trace", skip(role, artifact_kind, stage_certificates))]
pub(super) fn output_artifact(
    role: &str,
    artifact_kind: &str,
    source_stage_index: usize,
    item_count: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> BelvedereManifestOutputArtifact {
    if let Some(stage_certificate) = stage_certificates.get(source_stage_index) {
        return BelvedereManifestOutputArtifact {
            role: role.to_owned(),
            artifact_kind: artifact_kind.to_owned(),
            source_stage_role: stage_certificate.role().to_owned(),
            source_stage_certificate_id: stage_certificate.certificate_id().to_owned(),
            source_stage_certificate_digest: stage_certificate.digest().to_owned(),
            item_count,
        };
    }

    BelvedereManifestOutputArtifact {
        role: role.to_owned(),
        artifact_kind: artifact_kind.to_owned(),
        source_stage_role: "unknown".to_owned(),
        source_stage_certificate_id: "unknown".to_owned(),
        source_stage_certificate_digest: "unknown".to_owned(),
        item_count,
    }
}
