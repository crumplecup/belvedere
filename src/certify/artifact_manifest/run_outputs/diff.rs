//! Diff-run output artifact lists.

mod artifacts;
mod snapshots;
mod stage_certificates;

use tracing::instrument;

use crate::certify::BelvedereRunCertificate;

use super::super::{
    BelvedereDiffManifestReceipts, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate, indexed_artifacts::run_certificate_output,
    projection_outputs::view_model_projection_outputs,
};
use artifacts::landscape_diff_outputs;
use snapshots::{current_snapshot_outputs, previous_snapshot_outputs};
use stage_certificates::diff_stage_certificate_outputs;

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
    output_artifacts.extend(landscape_diff_outputs(receipts, stage_certificates));
    output_artifacts
}
