//! Diff-specific output artifacts.

use tracing::instrument;

use super::super::super::{
    BelvedereDiffManifestReceipts, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate, indexed_artifacts::output_artifact,
    projection_outputs::ui_ir_projection_outputs,
};

#[instrument(level = "trace", skip(receipts, stage_certificates))]
pub(super) fn landscape_diff_outputs(
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
