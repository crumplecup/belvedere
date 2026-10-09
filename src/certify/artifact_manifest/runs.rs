//! Manifest builders for Belvedere run reports.

use std::collections::BTreeMap;

use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{
    BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID, BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    BelvedereInvariantCheck, BelvedereRunCertificate, IngestionReceipt, LandscapeReceipt,
    UiIrReceipt, ViewModelReceipt, hex_digest, update_field,
};

use super::accounting::{
    input_role_counts, manifest_invariant_checks, output_kind_counts, output_role_counts,
    output_source_stage_counts, stage_kind_counts, stage_role_counts,
};
use super::indexed_artifacts::{
    diff_stage_roles, manifest_inputs, manifest_stage_certificates, normal_stage_roles,
    output_artifact, run_certificate_output,
};
use super::{
    BelvedereArtifactManifest, BelvedereDiffManifestReceipts, BelvedereManifestInputArtifact,
    BelvedereManifestOutputArtifact, BelvedereManifestStageCertificate,
};

impl BelvedereArtifactManifest {
    /// Build a manifest for a normal Belvedere pipeline run.
    #[must_use]
    #[instrument(
        level = "debug",
        skip(
            certificate,
            registry_receipt,
            assessment_receipt,
            landscape_receipt,
            view_model_receipt,
            ui_ir_receipt
        )
    )]
    pub(crate) fn normal_run(
        certificate: &BelvedereRunCertificate,
        registry_receipt: &IngestionReceipt,
        assessment_receipt: &IngestionReceipt,
        landscape_receipt: &LandscapeReceipt,
        view_model_receipt: &ViewModelReceipt,
        ui_ir_receipt: &UiIrReceipt,
    ) -> Self {
        let input_artifacts = manifest_inputs(certificate);
        let stage_certificates = manifest_stage_certificates(certificate, &normal_stage_roles());
        let output_artifacts = vec![
            run_certificate_output(certificate),
            output_artifact(
                "registry_receipt",
                "ingestion_receipt",
                0,
                registry_receipt.source_record_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "assessment_receipt",
                "ingestion_receipt",
                1,
                assessment_receipt.source_record_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "landscape_graph",
                "landscape_graph",
                2,
                landscape_receipt.landscape_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "landscape_receipt",
                "landscape_receipt",
                2,
                landscape_receipt.landscape_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "view_model",
                "auditor_navigation_model",
                3,
                view_model_receipt.view_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "view_model_receipt",
                "view_model_receipt",
                3,
                view_model_receipt.view_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "ui_ir_projection",
                "elicit_ui_projection",
                4,
                ui_ir_receipt.trace_widget_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "ui_ir_receipt",
                "ui_ir_receipt",
                4,
                ui_ir_receipt.trace_widget_ids().len(),
                &stage_certificates,
            ),
        ];
        Self::new(
            certificate,
            input_artifacts,
            stage_certificates,
            output_artifacts,
        )
    }

    /// Build a manifest for a landscape diff run.
    #[must_use]
    #[instrument(level = "debug", skip(certificate, receipts))]
    pub(crate) fn landscape_diff_run(
        certificate: &BelvedereRunCertificate,
        receipts: BelvedereDiffManifestReceipts<'_>,
    ) -> Self {
        let input_artifacts = manifest_inputs(certificate);
        let stage_certificates = manifest_stage_certificates(certificate, &diff_stage_roles());
        let output_artifacts = vec![
            run_certificate_output(certificate),
            output_artifact(
                "diff_certificate",
                "stage_certificate",
                10,
                receipts.diff().diff_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_registry_receipt",
                "ingestion_receipt",
                0,
                receipts.previous().registry().source_record_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_assessment_receipt",
                "ingestion_receipt",
                1,
                receipts.previous().assessment().source_record_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_landscape_graph",
                "landscape_graph",
                2,
                receipts.previous().landscape().landscape_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_landscape_receipt",
                "landscape_receipt",
                2,
                receipts.previous().landscape().landscape_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_view_model",
                "auditor_navigation_model",
                3,
                receipts.previous().view_model().view_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_view_model_receipt",
                "view_model_receipt",
                3,
                receipts.previous().view_model().view_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_ui_ir_projection",
                "elicit_ui_projection",
                4,
                receipts.previous().ui_ir().trace_widget_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "previous_ui_ir_receipt",
                "ui_ir_receipt",
                4,
                receipts.previous().ui_ir().trace_widget_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_registry_receipt",
                "ingestion_receipt",
                5,
                receipts.current().registry().source_record_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_assessment_receipt",
                "ingestion_receipt",
                6,
                receipts.current().assessment().source_record_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_landscape_graph",
                "landscape_graph",
                7,
                receipts.current().landscape().landscape_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_landscape_receipt",
                "landscape_receipt",
                7,
                receipts.current().landscape().landscape_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_view_model",
                "auditor_navigation_model",
                8,
                receipts.current().view_model().view_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_view_model_receipt",
                "view_model_receipt",
                8,
                receipts.current().view_model().view_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_ui_ir_projection",
                "elicit_ui_projection",
                9,
                receipts.current().ui_ir().trace_widget_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "current_ui_ir_receipt",
                "ui_ir_receipt",
                9,
                receipts.current().ui_ir().trace_widget_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "landscape_diff",
                "landscape_diff",
                10,
                receipts.diff().diff_item_ids().len(),
                &stage_certificates,
            ),
            output_artifact(
                "landscape_diff_receipt",
                "landscape_diff_receipt",
                10,
                receipts.diff().diff_item_ids().len(),
                &stage_certificates,
            ),
        ];
        Self::new(
            certificate,
            input_artifacts,
            stage_certificates,
            output_artifacts,
        )
    }

    #[instrument(
        level = "debug",
        skip(certificate, input_artifacts, stage_certificates, output_artifacts)
    )]
    fn new(
        certificate: &BelvedereRunCertificate,
        input_artifacts: Vec<BelvedereManifestInputArtifact>,
        stage_certificates: Vec<BelvedereManifestStageCertificate>,
        output_artifacts: Vec<BelvedereManifestOutputArtifact>,
    ) -> Self {
        let counts = BTreeMap::from([
            ("input_artifacts".to_owned(), input_artifacts.len()),
            ("output_artifacts".to_owned(), output_artifacts.len()),
            ("stage_certificates".to_owned(), stage_certificates.len()),
        ]);
        let input_role_counts = input_role_counts(&input_artifacts);
        let stage_role_counts = stage_role_counts(&stage_certificates);
        let stage_kind_counts = stage_kind_counts(&stage_certificates);
        let output_role_counts = output_role_counts(&output_artifacts);
        let output_kind_counts = output_kind_counts(&output_artifacts);
        let output_source_stage_counts = output_source_stage_counts(&output_artifacts);
        let invariant_checks = manifest_invariant_checks(
            &counts,
            &input_artifacts,
            &stage_certificates,
            &output_artifacts,
        );
        let digest = manifest_digest(ManifestDigestEvidence {
            run_certificate_id: certificate.id(),
            run_certificate_digest: certificate.digest(),
            counts: &counts,
            input_role_counts: &input_role_counts,
            stage_role_counts: &stage_role_counts,
            stage_kind_counts: &stage_kind_counts,
            output_role_counts: &output_role_counts,
            output_kind_counts: &output_kind_counts,
            output_source_stage_counts: &output_source_stage_counts,
            invariant_checks: &invariant_checks,
            input_artifacts: &input_artifacts,
            stage_certificates: &stage_certificates,
            output_artifacts: &output_artifacts,
        });
        let id = format!("belvedere.manifest.artifacts:{digest}");
        Self {
            schema_id: BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID.to_owned(),
            schema_version: BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
            id,
            run_certificate_id: certificate.id().to_owned(),
            run_certificate_digest: certificate.digest().to_owned(),
            counts,
            input_role_counts,
            stage_role_counts,
            stage_kind_counts,
            output_role_counts,
            output_kind_counts,
            output_source_stage_counts,
            invariant_checks,
            input_artifacts,
            stage_certificates,
            output_artifacts,
            digest,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ManifestDigestEvidence<'evidence> {
    run_certificate_id: &'evidence str,
    run_certificate_digest: &'evidence str,
    counts: &'evidence BTreeMap<String, usize>,
    input_role_counts: &'evidence BTreeMap<String, usize>,
    stage_role_counts: &'evidence BTreeMap<String, usize>,
    stage_kind_counts: &'evidence BTreeMap<String, usize>,
    output_role_counts: &'evidence BTreeMap<String, usize>,
    output_kind_counts: &'evidence BTreeMap<String, usize>,
    output_source_stage_counts: &'evidence BTreeMap<String, usize>,
    invariant_checks: &'evidence [BelvedereInvariantCheck],
    input_artifacts: &'evidence [BelvedereManifestInputArtifact],
    stage_certificates: &'evidence [BelvedereManifestStageCertificate],
    output_artifacts: &'evidence [BelvedereManifestOutputArtifact],
}

#[instrument(level = "debug", skip(evidence))]
fn manifest_digest(evidence: ManifestDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID);
    update_field(
        &mut hasher,
        &BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION.to_string(),
    );
    update_field(&mut hasher, evidence.run_certificate_id);
    update_field(&mut hasher, evidence.run_certificate_digest);
    update_counts(&mut hasher, evidence.counts);
    update_counts(&mut hasher, evidence.input_role_counts);
    update_counts(&mut hasher, evidence.stage_role_counts);
    update_counts(&mut hasher, evidence.stage_kind_counts);
    update_counts(&mut hasher, evidence.output_role_counts);
    update_counts(&mut hasher, evidence.output_kind_counts);
    update_counts(&mut hasher, evidence.output_source_stage_counts);
    for check in evidence.invariant_checks {
        update_field(&mut hasher, check.name());
        update_field(&mut hasher, &check.passed().to_string());
        update_field(&mut hasher, check.detail());
    }
    for artifact in evidence.input_artifacts {
        update_field(&mut hasher, artifact.role());
        update_field(&mut hasher, artifact.artifact_id());
        update_field(&mut hasher, artifact.digest());
    }
    for certificate in evidence.stage_certificates {
        update_field(&mut hasher, certificate.role());
        update_field(&mut hasher, certificate.stage());
        update_field(&mut hasher, certificate.certificate_id());
        update_field(&mut hasher, certificate.digest());
        update_field(&mut hasher, certificate.contract());
        update_field(&mut hasher, &certificate.output_count().to_string());
    }
    for artifact in evidence.output_artifacts {
        update_field(&mut hasher, artifact.role());
        update_field(&mut hasher, artifact.artifact_kind());
        update_field(&mut hasher, artifact.artifact_id());
        update_field(&mut hasher, artifact.artifact_digest());
        update_field(&mut hasher, artifact.source_stage_role());
        update_field(&mut hasher, artifact.source_stage_certificate_id());
        update_field(&mut hasher, artifact.source_stage_certificate_digest());
        update_field(&mut hasher, &artifact.item_count().to_string());
    }
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(hasher, counts))]
fn update_counts(hasher: &mut Sha256, counts: &BTreeMap<String, usize>) {
    for (name, count) in counts {
        update_field(hasher, name);
        update_field(hasher, &count.to_string());
    }
}
