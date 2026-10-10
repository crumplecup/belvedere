//! Receipt construction for `elicit_ui` projection exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_new::new;
use tracing::instrument;

use crate::{
    certify::{
        UiIrReceipt, UiIrReceiptBuilder, UiIrReceiptCounts, UiIrReceiptDigestEvidenceBuilder,
        ui_ir_receipt_identity,
    },
    contracts::{
        AuditorNavigationModel, BelvedereContract, ElicitUiProjectionContract, TraceWidgetRef,
        UiIrSection, UiStandardMint, UiViewportVerification,
    },
    exchanges::UiIrError,
};

mod proof_tokens;
mod receipt_counts;
mod receipt_outputs;

use proof_tokens::proof_token_counts;
use receipt_counts::{
    minted_prop_counts, role_counts, section_population_counts, section_trace_counts,
    section_widget_counts, source_locator_counts, trace_source_counts, trace_source_kind_counts,
    viewport_ids, viewport_verification_counts,
};
use receipt_outputs::{output_ids, output_kind_counts};

#[instrument(
    level = "debug",
    skip(
        model,
        source_view_model_receipt,
        sections,
        trace_widgets,
        tree,
        viewport_verifications,
        minted_standards
    )
)]
pub(super) fn build_receipt(
    model: &AuditorNavigationModel,
    source_view_model_receipt: SourceViewModelReceipt<'_>,
    sections: &[UiIrSection],
    trace_widgets: &[TraceWidgetRef],
    tree: &elicit_ui::VerifiedTree,
    viewport_verifications: &[UiViewportVerification],
    minted_standards: &[UiStandardMint],
) -> Result<UiIrReceipt, UiIrError> {
    let evidence = UiIrReceiptEvidenceBuilder::default()
        .contract(ElicitUiProjectionContract::NAME.to_owned())
        .source_view_model_receipt(source_view_model_receipt)
        .source_record_ids(model.source_record_ids().as_slice())
        .source_locator_counts(source_locator_counts(model, trace_widgets))
        .sections(sections)
        .trace_widgets(trace_widgets)
        .tree(tree)
        .viewport_verifications(viewport_verifications)
        .minted_standards(minted_standards)
        .build()
        .map_err(|error| UiIrError::construction("UiIrReceiptEvidence", error))?;
    build_receipt_from_evidence(evidence)
}

#[instrument(level = "debug", skip(evidence))]
pub(in crate::exchanges::ui_ir) fn build_receipt_from_evidence(
    evidence: UiIrReceiptEvidence<'_>,
) -> Result<UiIrReceipt, UiIrError> {
    let receipt_output_ids = output_ids(
        evidence.sections,
        evidence.trace_widgets,
        evidence.viewport_verifications,
        evidence.minted_standards,
    );
    let receipt_output_kind_counts = output_kind_counts(&receipt_output_ids);
    let receipt_counts = UiIrReceiptCounts::new(
        evidence.sections.len(),
        evidence.trace_widgets.len(),
        evidence
            .trace_widgets
            .iter()
            .map(|widget| widget.source_record_ids().len())
            .sum(),
        evidence.tree.nodes().len(),
    );
    let receipt_source_record_ids = evidence.source_record_ids.to_vec();
    let receipt_section_ids = evidence
        .sections
        .iter()
        .map(|section| section.id().to_owned())
        .collect::<Vec<_>>();
    let receipt_section_root_widget_ids = evidence
        .sections
        .iter()
        .map(UiIrSection::root_widget_id)
        .collect::<Vec<_>>();
    let receipt_trace_widget_ids = evidence
        .trace_widgets
        .iter()
        .map(TraceWidgetRef::widget_id)
        .collect::<Vec<_>>();
    let receipt_trace_widget_ref_ids = evidence
        .trace_widgets
        .iter()
        .map(|widget| widget.trace_ref_id().to_owned())
        .collect::<Vec<_>>();
    let receipt_trace_widget_digests = evidence
        .trace_widgets
        .iter()
        .map(|widget| widget.trace_widget_digest().to_owned())
        .collect::<Vec<_>>();
    let receipt_role_counts = role_counts(evidence.trace_widgets);
    let receipt_section_widget_counts = section_widget_counts(evidence.sections);
    let receipt_section_population_counts = section_population_counts(evidence.sections);
    let receipt_section_trace_counts =
        section_trace_counts(evidence.sections, evidence.trace_widgets);
    let receipt_viewport_ids = viewport_ids(evidence.viewport_verifications);
    let receipt_viewport_verification_counts =
        viewport_verification_counts(evidence.viewport_verifications);
    let receipt_trace_source_counts = trace_source_counts(evidence.trace_widgets);
    let receipt_trace_source_kind_counts = trace_source_kind_counts(evidence.trace_widgets);
    let receipt_source_locator_counts = evidence.source_locator_counts;
    let receipt_minted_prop_counts = minted_prop_counts(evidence.minted_standards);
    let receipt_proof_token_counts = proof_token_counts(evidence.viewport_verifications);
    let receipt_minted_standards = evidence
        .minted_standards
        .iter()
        .map(|standard| standard.standard().to_owned())
        .collect::<Vec<_>>();
    let digest_evidence = UiIrReceiptDigestEvidenceBuilder::default()
        .contract(evidence.contract.as_str())
        .counts(receipt_counts)
        .source_view_model_receipt_id(evidence.source_view_model_receipt.receipt_id)
        .source_view_model_receipt_digest(evidence.source_view_model_receipt.receipt_digest)
        .source_record_ids(&receipt_source_record_ids)
        .section_ids(&receipt_section_ids)
        .section_root_widget_ids(&receipt_section_root_widget_ids)
        .trace_widget_ids(&receipt_trace_widget_ids)
        .trace_widget_ref_ids(&receipt_trace_widget_ref_ids)
        .trace_widget_digests(&receipt_trace_widget_digests)
        .output_ids(&receipt_output_ids)
        .output_kind_counts(&receipt_output_kind_counts)
        .role_counts(&receipt_role_counts)
        .section_widget_counts(&receipt_section_widget_counts)
        .section_population_counts(&receipt_section_population_counts)
        .section_trace_counts(&receipt_section_trace_counts)
        .viewport_ids(&receipt_viewport_ids)
        .viewport_verification_counts(&receipt_viewport_verification_counts)
        .trace_source_counts(&receipt_trace_source_counts)
        .trace_source_kind_counts(&receipt_trace_source_kind_counts)
        .source_locator_counts(&receipt_source_locator_counts)
        .minted_prop_counts(&receipt_minted_prop_counts)
        .proof_token_counts(&receipt_proof_token_counts)
        .minted_standards(&receipt_minted_standards)
        .build()
        .map_err(|error| UiIrError::construction("UiIrReceiptDigestEvidence", error))?;
    let (receipt_id, receipt_digest) = ui_ir_receipt_identity(digest_evidence);
    UiIrReceiptBuilder::default()
        .contract(evidence.contract)
        .counts(receipt_counts)
        .receipt_id(receipt_id)
        .receipt_digest(receipt_digest)
        .source_view_model_receipt_id(evidence.source_view_model_receipt.receipt_id)
        .source_view_model_receipt_digest(evidence.source_view_model_receipt.receipt_digest)
        .source_record_ids(receipt_source_record_ids)
        .section_ids(receipt_section_ids)
        .section_root_widget_ids(receipt_section_root_widget_ids)
        .trace_widget_ids(receipt_trace_widget_ids)
        .trace_widget_ref_ids(receipt_trace_widget_ref_ids)
        .trace_widget_digests(receipt_trace_widget_digests)
        .output_ids(receipt_output_ids)
        .output_kind_counts(receipt_output_kind_counts)
        .role_counts(receipt_role_counts)
        .section_widget_counts(receipt_section_widget_counts)
        .section_population_counts(receipt_section_population_counts)
        .section_trace_counts(receipt_section_trace_counts)
        .viewport_ids(receipt_viewport_ids)
        .viewport_verification_counts(receipt_viewport_verification_counts)
        .trace_source_counts(receipt_trace_source_counts)
        .trace_source_kind_counts(receipt_trace_source_kind_counts)
        .source_locator_counts(receipt_source_locator_counts)
        .minted_prop_counts(receipt_minted_prop_counts)
        .proof_token_counts(receipt_proof_token_counts)
        .minted_standards(receipt_minted_standards)
        .build()
        .map_err(|error| UiIrError::construction("UiIrReceipt", error))
}

#[derive(Debug, Clone, Copy, new)]
pub(in crate::exchanges::ui_ir) struct SourceViewModelReceipt<'receipt> {
    receipt_id: &'receipt str,
    receipt_digest: &'receipt str,
}

#[derive(Debug, Builder)]
#[builder(vis = "pub(crate)")]
pub(in crate::exchanges::ui_ir) struct UiIrReceiptEvidence<'evidence> {
    contract: String,
    source_view_model_receipt: SourceViewModelReceipt<'evidence>,
    source_record_ids: &'evidence [String],
    source_locator_counts: BTreeMap<String, usize>,
    sections: &'evidence [UiIrSection],
    trace_widgets: &'evidence [TraceWidgetRef],
    tree: &'evidence elicit_ui::VerifiedTree,
    viewport_verifications: &'evidence [UiViewportVerification],
    minted_standards: &'evidence [UiStandardMint],
}
