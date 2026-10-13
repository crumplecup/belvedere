//! Diff-run stage certificate output artifacts.

use tracing::instrument;

use super::super::super::{
    BelvedereDiffManifestReceipts, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate, indexed_artifacts::stage_certificate_output,
};

#[instrument(level = "trace", skip(receipts, stage_certificates))]
pub(super) fn diff_stage_certificate_outputs(
    receipts: BelvedereDiffManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts =
        snapshot_stage_certificate_outputs("previous", *receipts.previous(), 0, stage_certificates);
    output_artifacts.extend(snapshot_stage_certificate_outputs(
        "current",
        *receipts.current(),
        5,
        stage_certificates,
    ));
    output_artifacts.extend([
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
    ]);
    output_artifacts
}

#[instrument(level = "trace", skip(prefix, snapshot, stage_certificates))]
fn snapshot_stage_certificate_outputs(
    prefix: &str,
    snapshot: super::super::super::BelvedereSnapshotManifestReceipts<'_>,
    stage_offset: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    vec![
        stage_certificate_output(
            &format!("{prefix}_registry_ingestion_certificate"),
            "stage_certificate",
            stage_offset,
            snapshot.registry().source_record_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            &format!("{prefix}_assessment_ingestion_certificate"),
            "stage_certificate",
            stage_offset + 1,
            snapshot.assessment().source_record_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            &format!("{prefix}_landscape_modeling_certificate"),
            "stage_certificate",
            stage_offset + 2,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            &format!("{prefix}_auditor_navigation_certificate"),
            "stage_certificate",
            stage_offset + 3,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
        stage_certificate_output(
            &format!("{prefix}_ui_ir_projection_certificate"),
            "stage_certificate",
            stage_offset + 4,
            snapshot.ui_ir().output_ids().len(),
            stage_certificates,
        ),
    ]
}
