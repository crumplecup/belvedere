//! Stage certificates for Belvedere view-model exchanges.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    certify::{
        BelvederePipelineStage, ViewModelReceipt, view_model_item_invariant_checks,
        view_model_source_navigation_invariant_checks, view_model_trace_invariant_checks,
    },
    contracts::AuditorNavigationContract,
};

use super::{BelvedereStageCertificate, StageCertificateEvidence};
use crate::certify::contract_metadata::{
    contract_guarantees, contract_requirements, contract_standards,
};

impl BelvedereStageCertificate {
    /// Certificate the auditor navigation view-model stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn auditor_navigation(receipt: &ViewModelReceipt) -> Self {
        let mut counts = BTreeMap::from([
            (
                "assessment_history_items".to_owned(),
                receipt.counts().assessment_history_items(),
            ),
            (
                "assessment_items".to_owned(),
                receipt.counts().assessment_items(),
            ),
            (
                "assessment_work_items".to_owned(),
                receipt.counts().assessment_work_items(),
            ),
            ("claim_cards".to_owned(), receipt.counts().claim_cards()),
            (
                "claim_detail_items".to_owned(),
                receipt.counts().claim_detail_items(),
            ),
            ("filter_facets".to_owned(), receipt.counts().filter_facets()),
            (
                "backend_coverage_items".to_owned(),
                receipt.counts().backend_coverage_items(),
            ),
            ("gap_items".to_owned(), receipt.counts().gap_items()),
            (
                "proof_chain_items".to_owned(),
                receipt.counts().proof_chain_items(),
            ),
            (
                "review_workflow_items".to_owned(),
                receipt.counts().review_workflow_items(),
            ),
            ("root_items".to_owned(), receipt.counts().root_items()),
            ("source_landscapes".to_owned(), 1),
            ("source_items".to_owned(), receipt.counts().source_items()),
            ("view_model_items".to_owned(), receipt.view_item_ids().len()),
            ("warnings".to_owned(), receipt.warnings().len()),
            ("witness_items".to_owned(), receipt.counts().witness_items()),
        ]);
        counts.extend(receipt.lane_counts().clone());
        counts.extend(receipt.facet_group_counts().clone());
        counts.extend(receipt.source_artifact_counts().clone());
        counts.extend(receipt.source_kind_counts().clone());
        counts.extend(receipt.source_locator_counts().clone());
        counts.extend(receipt.proof_chain_topology_counts().clone());
        counts.extend(receipt.provenance_root_kind_counts().clone());
        counts.extend(receipt.review_workflow_counts().clone());
        counts.extend(receipt.trace_kind_counts().clone());
        counts.extend(receipt.trace_source_counts().clone());
        counts.extend(receipt.item_kind_counts().clone());
        let mut additional_invariant_checks = view_model_trace_invariant_checks(
            receipt.trace_source_counts(),
            receipt.source_record_ids(),
        );
        additional_invariant_checks.extend(view_model_item_invariant_checks(
            &counts,
            receipt.item_kind_counts(),
            receipt.view_item_ids(),
        ));
        additional_invariant_checks.extend(view_model_source_navigation_invariant_checks(
            receipt.source_locator_counts(),
            receipt.source_record_ids(),
        ));
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::AuditorNavigation,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<AuditorNavigationContract>(),
            requires: contract_requirements::<AuditorNavigationContract>(),
            ensures: contract_guarantees::<AuditorNavigationContract>(),
            input_ids: vec![receipt.source_landscape_receipt_id().to_owned()],
            input_digests: vec![receipt.source_landscape_receipt_digest().to_owned()],
            output_ids: receipt.view_item_ids().clone(),
            counts,
            additional_invariant_checks,
            warning_count: receipt.warnings().len(),
        })
    }
}
