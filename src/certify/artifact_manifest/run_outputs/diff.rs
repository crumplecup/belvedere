//! Diff-run output artifact lists.

use tracing::instrument;

use crate::certify::{BelvedereRunCertificate, BelvedereSnapshotManifestReceipts};

use super::super::{
    BelvedereDiffManifestReceipts, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate,
    indexed_artifacts::{output_artifact, run_certificate_output, stage_certificate_output},
    projection_outputs::{ui_ir_projection_outputs, view_model_projection_outputs},
};

#[instrument(level = "trace", skip(certificate, receipts, stage_certificates))]
pub(in crate::certify::artifact_manifest) fn diff_run_outputs(
    certificate: &BelvedereRunCertificate,
    receipts: BelvedereDiffManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts = vec![run_certificate_output(certificate, "--run-certificate")];
    output_artifacts.extend(diff_stage_certificate_outputs(receipts, stage_certificates));
    output_artifacts.extend(previous_snapshot_outputs(
        *receipts.previous(),
        stage_certificates,
    ));
    output_artifacts.extend(current_snapshot_outputs(
        *receipts.current(),
        stage_certificates,
    ));
    output_artifacts.extend(view_model_projection_outputs(
        None,
        receipts.current().view_model(),
        8,
        stage_certificates,
    ));
    output_artifacts.extend(diff_outputs(receipts, stage_certificates));
    output_artifacts
}

#[instrument(level = "trace", skip(receipts, stage_certificates))]
fn diff_stage_certificate_outputs(
    receipts: BelvedereDiffManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    vec![
        stage_certificate_output(
            "diff_certificate",
            "stage_certificate",
            10,
            receipts.diff().diff_item_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            "diff_review_certificate",
            "stage_certificate",
            11,
            receipts.diff_review().review_item_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            "diff_review_ui_ir_certificate",
            "stage_certificate",
            12,
            receipts.diff_review_ui_ir().output_ids().len(),
            stage_certificates,
        ),
    ]
}

#[instrument(level = "trace", skip(snapshot, stage_certificates))]
fn previous_snapshot_outputs(
    snapshot: BelvedereSnapshotManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts = vec![
        output_artifact(
            "previous_registry_receipt",
            "ingestion_receipt",
            snapshot.registry().receipt_id(),
            snapshot.registry().receipt_digest(),
            0,
            snapshot.registry().source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "previous_assessment_receipt",
            "ingestion_receipt",
            snapshot.assessment().receipt_id(),
            snapshot.assessment().receipt_digest(),
            1,
            snapshot.assessment().source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "previous_landscape_graph",
            "landscape_graph",
            snapshot.landscape().receipt_id(),
            snapshot.landscape().receipt_digest(),
            2,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "previous_landscape_receipt",
            "landscape_receipt",
            snapshot.landscape().receipt_id(),
            snapshot.landscape().receipt_digest(),
            2,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "previous_view_model",
            "auditor_navigation_model",
            snapshot.view_model().receipt_id(),
            snapshot.view_model().receipt_digest(),
            3,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
    ];
    output_artifacts.extend(view_model_projection_outputs(
        Some("previous"),
        snapshot.view_model(),
        3,
        stage_certificates,
    ));
    output_artifacts.extend([
        output_artifact(
            "previous_view_model_receipt",
            "view_model_receipt",
            snapshot.view_model().receipt_id(),
            snapshot.view_model().receipt_digest(),
            3,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "previous_ui_ir_projection",
            "elicit_ui_projection",
            snapshot.ui_ir().receipt_id(),
            snapshot.ui_ir().receipt_digest(),
            4,
            snapshot.ui_ir().trace_widget_ids().len(),
            stage_certificates,
        ),
    ]);
    output_artifacts.extend(ui_ir_projection_outputs(
        Some("previous"),
        snapshot.ui_ir(),
        4,
        stage_certificates,
    ));
    output_artifacts.extend([output_artifact(
        "previous_ui_ir_receipt",
        "ui_ir_receipt",
        snapshot.ui_ir().receipt_id(),
        snapshot.ui_ir().receipt_digest(),
        4,
        snapshot.ui_ir().trace_widget_ids().len(),
        stage_certificates,
    )]);
    output_artifacts
}

#[instrument(level = "trace", skip(snapshot, stage_certificates))]
fn current_snapshot_outputs(
    snapshot: BelvedereSnapshotManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts = vec![
        output_artifact(
            "current_registry_receipt",
            "ingestion_receipt",
            snapshot.registry().receipt_id(),
            snapshot.registry().receipt_digest(),
            5,
            snapshot.registry().source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "current_assessment_receipt",
            "ingestion_receipt",
            snapshot.assessment().receipt_id(),
            snapshot.assessment().receipt_digest(),
            6,
            snapshot.assessment().source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "current_landscape_graph",
            "landscape_graph",
            snapshot.landscape().receipt_id(),
            snapshot.landscape().receipt_digest(),
            7,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "current_landscape_receipt",
            "landscape_receipt",
            snapshot.landscape().receipt_id(),
            snapshot.landscape().receipt_digest(),
            7,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "current_view_model",
            "auditor_navigation_model",
            snapshot.view_model().receipt_id(),
            snapshot.view_model().receipt_digest(),
            8,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
    ];
    output_artifacts.extend(view_model_projection_outputs(
        Some("current"),
        snapshot.view_model(),
        8,
        stage_certificates,
    ));
    output_artifacts.extend([
        output_artifact(
            "current_view_model_receipt",
            "view_model_receipt",
            snapshot.view_model().receipt_id(),
            snapshot.view_model().receipt_digest(),
            8,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "current_ui_ir_projection",
            "elicit_ui_projection",
            snapshot.ui_ir().receipt_id(),
            snapshot.ui_ir().receipt_digest(),
            9,
            snapshot.ui_ir().trace_widget_ids().len(),
            stage_certificates,
        ),
    ]);
    output_artifacts.extend(ui_ir_projection_outputs(
        Some("current"),
        snapshot.ui_ir(),
        9,
        stage_certificates,
    ));
    output_artifacts.extend([output_artifact(
        "current_ui_ir_receipt",
        "ui_ir_receipt",
        snapshot.ui_ir().receipt_id(),
        snapshot.ui_ir().receipt_digest(),
        9,
        snapshot.ui_ir().trace_widget_ids().len(),
        stage_certificates,
    )]);
    output_artifacts
}

#[instrument(level = "trace", skip(receipts, stage_certificates))]
fn diff_outputs(
    receipts: BelvedereDiffManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts = vec![
        output_artifact(
            "landscape_diff",
            "landscape_diff",
            receipts.diff().receipt_id(),
            receipts.diff().receipt_digest(),
            10,
            receipts.diff().diff_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "landscape_diff_receipt",
            "landscape_diff_receipt",
            receipts.diff().receipt_id(),
            receipts.diff().receipt_digest(),
            10,
            receipts.diff().diff_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "landscape_diff_review_model",
            "landscape_diff_review_model",
            receipts.diff_review().receipt_id(),
            receipts.diff_review().receipt_digest(),
            11,
            receipts.diff_review().review_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "landscape_diff_review_receipt",
            "landscape_diff_review_receipt",
            receipts.diff_review().receipt_id(),
            receipts.diff_review().receipt_digest(),
            11,
            receipts.diff_review().review_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            "landscape_diff_review_ui_ir_projection",
            "elicit_ui_projection",
            receipts.diff_review_ui_ir().receipt_id(),
            receipts.diff_review_ui_ir().receipt_digest(),
            12,
            receipts.diff_review_ui_ir().trace_widget_ids().len(),
            stage_certificates,
        ),
    ];
    output_artifacts.extend(ui_ir_projection_outputs(
        Some("landscape_diff_review"),
        receipts.diff_review_ui_ir(),
        12,
        stage_certificates,
    ));
    output_artifacts.extend([output_artifact(
        "landscape_diff_review_ui_ir_receipt",
        "ui_ir_receipt",
        receipts.diff_review_ui_ir().receipt_id(),
        receipts.diff_review_ui_ir().receipt_digest(),
        12,
        receipts.diff_review_ui_ir().trace_widget_ids().len(),
        stage_certificates,
    )]);
    output_artifacts
}
