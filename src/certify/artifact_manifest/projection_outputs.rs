//! Output artifact lists for auditor projection stages.

use tracing::instrument;

use crate::certify::{UiIrReceipt, ViewModelReceipt};

use super::{
    BelvedereManifestOutputArtifact, BelvedereManifestStageCertificate,
    indexed_artifacts::output_artifact,
};

#[instrument(level = "trace", skip(receipt, stage_certificates))]
pub(super) fn view_model_projection_outputs(
    role_prefix: Option<&str>,
    receipt: &ViewModelReceipt,
    source_stage_index: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    let counts = receipt.counts();
    [
        ("overview", "auditor_overview", 1),
        (
            "backend_coverage",
            "auditor_backend_coverage",
            counts.backend_coverage_items(),
        ),
        (
            "claim_explorer",
            "auditor_claim_explorer",
            counts.claim_cards(),
        ),
        (
            "claim_details",
            "auditor_claim_details",
            counts.claim_detail_items(),
        ),
        (
            "filter_facets",
            "auditor_filter_facets",
            counts.filter_facets(),
        ),
        (
            "source_navigation",
            "auditor_source_navigation",
            counts.source_items(),
        ),
        (
            "proof_chains",
            "auditor_proof_chains",
            counts.proof_chain_items(),
        ),
        (
            "review_workflow",
            "auditor_review_workflow",
            counts.review_workflow_items(),
        ),
        (
            "assessment_history",
            "auditor_assessment_history",
            counts.assessment_history_items(),
        ),
        (
            "assessment_queue",
            "auditor_assessment_queue",
            counts.assessment_items(),
        ),
        (
            "assessment_work",
            "auditor_assessment_work",
            counts.assessment_work_items(),
        ),
        (
            "witness_surface",
            "auditor_witness_surface",
            counts.witness_items(),
        ),
        (
            "provenance_roots",
            "auditor_provenance_roots",
            counts.root_items(),
        ),
        ("gaps", "auditor_gaps", counts.gap_items()),
    ]
    .into_iter()
    .map(|(role, artifact_kind, item_count)| {
        output_artifact(
            &projection_role(role_prefix, role),
            artifact_kind,
            receipt.receipt_id(),
            receipt.receipt_digest(),
            source_stage_index,
            item_count,
            stage_certificates,
        )
    })
    .collect()
}

#[instrument(level = "trace", skip(receipt, stage_certificates))]
pub(super) fn ui_ir_projection_outputs(
    role_prefix: Option<&str>,
    receipt: &UiIrReceipt,
    source_stage_index: usize,
    stage_certificates: &[BelvedereManifestStageCertificate],
) -> Vec<BelvedereManifestOutputArtifact> {
    [
        (
            "ui_ir_outputs",
            "elicit_ui_outputs",
            receipt.output_ids().len(),
        ),
        (
            "ui_ir_sections",
            "elicit_ui_sections",
            receipt.section_ids().len(),
        ),
        (
            "ui_ir_section_root_widgets",
            "elicit_ui_section_root_widgets",
            receipt.section_root_widget_ids().len(),
        ),
        (
            "ui_ir_trace_widgets",
            "elicit_ui_trace_widgets",
            receipt.trace_widget_ids().len(),
        ),
        (
            "ui_ir_trace_widget_refs",
            "elicit_ui_trace_widget_refs",
            receipt.trace_widget_ref_ids().len(),
        ),
        (
            "ui_ir_viewports",
            "elicit_ui_viewports",
            receipt.viewport_ids().len(),
        ),
        (
            "ui_ir_minted_standards",
            "elicit_ui_minted_standards",
            receipt.minted_standards().len(),
        ),
    ]
    .into_iter()
    .map(|(role, artifact_kind, item_count)| {
        output_artifact(
            &projection_role(role_prefix, role),
            artifact_kind,
            receipt.receipt_id(),
            receipt.receipt_digest(),
            source_stage_index,
            item_count,
            stage_certificates,
        )
    })
    .collect()
}

#[instrument(level = "trace", skip(role_prefix, role))]
fn projection_role(role_prefix: Option<&str>, role: &str) -> String {
    role_prefix.map_or_else(|| role.to_owned(), |prefix| format!("{prefix}_{role}"))
}
