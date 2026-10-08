//! Stable output IDs emitted by the auditor navigation view model.

use tracing::instrument;

use crate::contracts::{AuditorNavigationModel, BackendCoverageItem, WitnessSurfaceItem};

#[instrument(level = "debug", skip(model))]
pub(super) fn view_item_ids(model: &AuditorNavigationModel) -> Vec<String> {
    let mut ids = vec!["view_model.overview".to_owned()];
    ids.extend(
        model
            .filter_facets()
            .iter()
            .map(|item| format!("view_model.filter_facet:{}", item.facet_id())),
    );
    ids.extend(model.backend_coverage().iter().map(backend_coverage_id));
    ids.extend(
        model
            .claim_explorer()
            .iter()
            .map(|item| format!("view_model.claim:{}", item.claim_id())),
    );
    ids.extend(
        model
            .claim_details()
            .iter()
            .map(|item| format!("view_model.claim_detail:{}", item.claim_id())),
    );
    ids.extend(
        model
            .proof_chains()
            .iter()
            .map(|item| format!("view_model.proof_chain:{}", item.claim_id())),
    );
    ids.extend(
        model
            .review_workflow()
            .iter()
            .map(|item| format!("view_model.review_task:{}", item.task_id())),
    );
    ids.extend(
        model
            .assessment_history()
            .iter()
            .map(|item| format!("view_model.assessment_history:{}", item.proof_id())),
    );
    ids.extend(
        model
            .assessment_queue()
            .iter()
            .map(|item| format!("view_model.assessment_queue:{}", item.assessment_id())),
    );
    ids.extend(model.assessment_work().iter().map(|item| {
        format!(
            "view_model.assessment_work:{}:{}",
            item.claim_id(),
            item.queue_lane()
        )
    }));
    ids.extend(model.witness_surface().iter().map(witness_id));
    ids.extend(
        model
            .provenance_roots()
            .iter()
            .map(|item| format!("view_model.provenance_root:{}", item.claim_id())),
    );
    ids.extend(model.gaps().iter().map(|item| item.gap_id().to_owned()));
    ids.extend(
        model
            .source_navigation()
            .iter()
            .map(|item| format!("view_model.source:{}", item.source_record_id())),
    );
    ids
}

#[instrument(level = "trace", skip(item))]
fn backend_coverage_id(item: &BackendCoverageItem) -> String {
    format!(
        "view_model.backend_coverage:{}:{}",
        item.claim_id(),
        item.verifier()
    )
}

#[instrument(level = "trace", skip(item))]
fn witness_id(item: &WitnessSurfaceItem) -> String {
    format!(
        "view_model.witness:{}:{}:{}",
        item.claim_id(),
        item.verifier(),
        item.destination_module()
    )
}
