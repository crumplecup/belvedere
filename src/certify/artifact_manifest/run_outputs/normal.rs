//! Normal-run output artifact lists.

use tracing::instrument;

use crate::certify::{
    BelvedereRunCertificate, IngestionReceipt, LandscapeReceipt, UiIrReceipt, ViewModelReceipt,
};

use super::super::{
    BelvedereManifestOutputArtifact, BelvedereManifestStageCertificate,
    indexed_artifacts::{output_artifact, run_certificate_output, stage_certificate_output},
    projection_outputs::{ui_ir_projection_outputs, view_model_projection_outputs},
};

#[instrument(
    level = "trace",
    skip(
        certificate,
        registry_receipt,
        assessment_receipt,
        landscape_receipt,
        view_model_receipt,
        ui_ir_receipt,
        stage_certificates
    )
)]
pub(in crate::certify::artifact_manifest) fn normal_run_outputs(
    certificate: &BelvedereRunCertificate,
    registry_receipt: &IngestionReceipt,
    assessment_receipt: &IngestionReceipt,
    landscape_receipt: &LandscapeReceipt,
    view_model_receipt: &ViewModelReceipt,
    ui_ir_receipt: &UiIrReceipt,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts = normal_stage_certificate_outputs(
        registry_receipt,
        assessment_receipt,
        landscape_receipt,
        view_model_receipt,
        ui_ir_receipt,
        stage_certificates,
    );
    output_artifacts.extend([
        run_certificate_output(certificate, "--certificate"),
        output_artifact(
            "registry_receipt",
            "ingestion_receipt",
            registry_receipt.receipt_id(),
            registry_receipt.receipt_digest(),
            0,
            registry_receipt.source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "assessment_receipt",
            "ingestion_receipt",
            assessment_receipt.receipt_id(),
            assessment_receipt.receipt_digest(),
            1,
            assessment_receipt.source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "landscape_graph",
            "landscape_graph",
            landscape_receipt.receipt_id(),
            landscape_receipt.receipt_digest(),
            2,
            landscape_receipt.landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "landscape_receipt",
            "landscape_receipt",
            landscape_receipt.receipt_id(),
            landscape_receipt.receipt_digest(),
            2,
            landscape_receipt.landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "view_model",
            "auditor_navigation_model",
            view_model_receipt.receipt_id(),
            view_model_receipt.receipt_digest(),
            3,
            view_model_receipt.view_item_ids().len(),
            stage_certificates,
        ),
    ]);
    output_artifacts.extend(view_model_projection_outputs(
        None,
        view_model_receipt,
        3,
        stage_certificates,
    ));
    output_artifacts.extend([
        output_artifact(
            "view_model_receipt",
            "view_model_receipt",
            view_model_receipt.receipt_id(),
            view_model_receipt.receipt_digest(),
            3,
            view_model_receipt.view_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "ui_ir_projection",
            "elicit_ui_projection",
            ui_ir_receipt.receipt_id(),
            ui_ir_receipt.receipt_digest(),
            4,
            ui_ir_receipt.trace_widget_ids().len(),
            stage_certificates,
        ),
    ]);
    output_artifacts.extend(ui_ir_projection_outputs(
        None,
        ui_ir_receipt,
        4,
        stage_certificates,
    ));
    output_artifacts.extend([output_artifact(
        "ui_ir_receipt",
        "ui_ir_receipt",
        ui_ir_receipt.receipt_id(),
        ui_ir_receipt.receipt_digest(),
        4,
        ui_ir_receipt.trace_widget_ids().len(),
        stage_certificates,
    )]);
    output_artifacts
}

#[instrument(
    level = "trace",
    skip(
        registry_receipt,
        assessment_receipt,
        landscape_receipt,
        view_model_receipt,
        ui_ir_receipt,
        stage_certificates
    )
)]
fn normal_stage_certificate_outputs(
    registry_receipt: &IngestionReceipt,
    assessment_receipt: &IngestionReceipt,
    landscape_receipt: &LandscapeReceipt,
    view_model_receipt: &ViewModelReceipt,
    ui_ir_receipt: &UiIrReceipt,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    vec![
        stage_certificate_output(
            "registry_ingestion_certificate",
            "stage_certificate",
            0,
            registry_receipt.source_record_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            "assessment_ingestion_certificate",
            "stage_certificate",
            1,
            assessment_receipt.source_record_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            "landscape_modeling_certificate",
            "stage_certificate",
            2,
            landscape_receipt.landscape_item_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            "auditor_navigation_certificate",
            "stage_certificate",
            3,
            view_model_receipt.view_item_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            "ui_ir_projection_certificate",
            "stage_certificate",
            4,
            ui_ir_receipt.output_ids().len(),
            stage_certificates,
        ),
    ]
}
