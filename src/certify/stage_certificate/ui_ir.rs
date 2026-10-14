//! Stage certificates for UI IR projection exchanges.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    certify::{
        BelvedereCertificationError, BelvederePipelineStage, UiIrReceipt,
        ui_ir_output_invariant_checks, ui_ir_source_locator_invariant_checks,
        ui_ir_trace_invariant_checks,
    },
    contracts::{
        BelvedereContract, ElicitUiDiffReviewProjectionContract, ElicitUiProjectionContract,
    },
};

use super::{BelvedereStageCertificate, StageCertificateEvidence};
use crate::certify::contract_metadata::{
    contract_guarantees, contract_requirements, contract_standards,
};

impl BelvedereStageCertificate {
    /// Certificate the `elicit_ui` projection stage.
    ///
    /// # Errors
    ///
    /// Refuses a success certificate when a required check fails.
    #[instrument(level = "debug", skip(receipt))]
    pub fn ui_ir_projection(receipt: &UiIrReceipt) -> Result<Self, BelvedereCertificationError> {
        Self::ui_ir_projection_for::<ElicitUiProjectionContract>(
            BelvederePipelineStage::UiIrProjection,
            receipt,
        )
    }

    /// Certificate the landscape diff review `elicit_ui` projection stage.
    ///
    /// # Errors
    ///
    /// Refuses a success certificate when a required check fails.
    #[instrument(level = "debug", skip(receipt))]
    pub fn diff_review_ui_ir_projection(
        receipt: &UiIrReceipt,
    ) -> Result<Self, BelvedereCertificationError> {
        Self::ui_ir_projection_for::<ElicitUiDiffReviewProjectionContract>(
            BelvederePipelineStage::LandscapeDiffReviewUiIrProjection,
            receipt,
        )
    }

    #[instrument(level = "debug", skip(stage, receipt))]
    fn ui_ir_projection_for<C: BelvedereContract>(
        stage: BelvederePipelineStage,
        receipt: &UiIrReceipt,
    ) -> Result<Self, BelvedereCertificationError> {
        let mut counts = BTreeMap::from([
            (
                "minted_standards".to_owned(),
                receipt.minted_standards().len(),
            ),
            ("sections".to_owned(), receipt.counts().section_count()),
            ("trace_refs".to_owned(), receipt.counts().trace_ref_count()),
            ("widgets".to_owned(), receipt.counts().widget_count()),
            ("tree_nodes".to_owned(), receipt.counts().node_count()),
        ]);
        counts.extend(receipt.role_counts().clone());
        counts.extend(receipt.section_widget_counts().clone());
        counts.extend(receipt.section_population_counts().clone());
        counts.extend(receipt.section_trace_counts().clone());
        counts.extend(receipt.viewport_verification_counts().clone());
        counts.extend(receipt.trace_source_counts().clone());
        counts.extend(receipt.trace_source_kind_counts().clone());
        counts.extend(receipt.source_locator_counts().clone());
        counts.extend(receipt.minted_prop_counts().clone());
        counts.extend(receipt.proof_token_counts().clone());
        counts.extend(receipt.output_kind_counts().clone());
        counts.insert("source_view_models".to_owned(), 1);
        let mut additional_invariant_checks = ui_ir_trace_invariant_checks(
            receipt.counts().widget_count(),
            receipt.counts().trace_ref_count(),
            receipt.section_widget_counts(),
            receipt.section_trace_counts(),
            receipt.trace_source_counts(),
            receipt.source_record_ids(),
        );
        additional_invariant_checks.extend(ui_ir_output_invariant_checks(
            &counts,
            receipt.output_kind_counts(),
            receipt.output_ids(),
        ));
        additional_invariant_checks.extend(ui_ir_source_locator_invariant_checks(
            receipt.source_locator_counts(),
        ));
        Self::new(StageCertificateEvidence {
            stage,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<C>(),
            requires: contract_requirements::<C>(),
            ensures: contract_guarantees::<C>(),
            input_ids: vec![receipt.source_view_model_receipt_id().to_owned()],
            input_digests: vec![receipt.source_view_model_receipt_digest().to_owned()],
            output_ids: receipt.output_ids().clone(),
            counts,
            additional_invariant_checks,
            transition_checks: receipt.transition_checks().to_vec(),
            expected_transition_names: C::ASSERTION_NAMES.iter().map(ToString::to_string).collect(),
            expected_precondition_names: C::PRECONDITION_NAMES
                .iter()
                .map(ToString::to_string)
                .collect(),
            warning_count: 0,
        })
    }
}
