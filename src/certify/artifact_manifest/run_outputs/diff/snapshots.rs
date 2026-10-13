//! Snapshot output artifacts for diff runs.

use tracing::instrument;

use crate::certify::BelvedereSnapshotManifestReceipts;

use super::super::super::{
    BelvedereManifestOutputArtifact, BelvedereManifestStageCertificate,
    indexed_artifacts::output_artifact,
    projection_outputs::{ui_ir_projection_outputs, view_model_projection_outputs},
};

#[instrument(level = "trace", skip(snapshot, stage_certificates))]
pub(super) fn previous_snapshot_outputs(
    snapshot: BelvedereSnapshotManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    snapshot_outputs("previous", snapshot, 0, stage_certificates)
}

#[instrument(level = "trace", skip(snapshot, stage_certificates))]
pub(super) fn current_snapshot_outputs(
    snapshot: BelvedereSnapshotManifestReceipts<'_>,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    snapshot_outputs("current", snapshot, 5, stage_certificates)
}

#[instrument(level = "trace", skip(prefix, snapshot, stage_certificates))]
fn snapshot_outputs(
    prefix: &str,
    snapshot: BelvedereSnapshotManifestReceipts<'_>,
    stage_offset: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let mut output_artifacts = vec![
        output_artifact(
            &format!("{prefix}_registry_receipt"),
            "ingestion_receipt",
            snapshot.registry().receipt_id(),
            snapshot.registry().receipt_digest(),
            stage_offset,
            snapshot.registry().source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            &format!("{prefix}_assessment_receipt"),
            "ingestion_receipt",
            snapshot.assessment().receipt_id(),
            snapshot.assessment().receipt_digest(),
            stage_offset + 1,
            snapshot.assessment().source_record_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            &format!("{prefix}_landscape_graph"),
            "landscape_graph",
            snapshot.landscape().receipt_id(),
            snapshot.landscape().receipt_digest(),
            stage_offset + 2,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            &format!("{prefix}_landscape_receipt"),
            "landscape_receipt",
            snapshot.landscape().receipt_id(),
            snapshot.landscape().receipt_digest(),
            stage_offset + 2,
            snapshot.landscape().landscape_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            &format!("{prefix}_view_model"),
            "auditor_navigation_model",
            snapshot.view_model().receipt_id(),
            snapshot.view_model().receipt_digest(),
            stage_offset + 3,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
    ];
    output_artifacts.extend(view_model_projection_outputs(
        Some(prefix),
        snapshot.view_model(),
        stage_offset + 3,
        stage_certificates,
    ));
    output_artifacts.extend([
        output_artifact(
            &format!("{prefix}_view_model_receipt"),
            "view_model_receipt",
            snapshot.view_model().receipt_id(),
            snapshot.view_model().receipt_digest(),
            stage_offset + 3,
            snapshot.view_model().view_item_ids().len(),
            stage_certificates,
        ),
        output_artifact(
            &format!("{prefix}_ui_ir_projection"),
            "elicit_ui_projection",
            snapshot.ui_ir().receipt_id(),
            snapshot.ui_ir().receipt_digest(),
            stage_offset + 4,
            snapshot.ui_ir().trace_widget_ids().len(),
            stage_certificates,
        ),
    ]);
    output_artifacts.extend(ui_ir_projection_outputs(
        Some(prefix),
        snapshot.ui_ir(),
        stage_offset + 4,
        stage_certificates,
    ));
    output_artifacts.extend([output_artifact(
        &format!("{prefix}_ui_ir_receipt"),
        "ui_ir_receipt",
        snapshot.ui_ir().receipt_id(),
        snapshot.ui_ir().receipt_digest(),
        stage_offset + 4,
        snapshot.ui_ir().trace_widget_ids().len(),
        stage_certificates,
    )]);
    output_artifacts
}
