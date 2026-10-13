//! `elicit_ui` projection for landscape diff review evidence.

use std::collections::BTreeMap;

use elicit_ui::{AccessKitUiBackend, Viewport};
use elicitation::Established;
use tracing::instrument;

use crate::{
    certify::UiIrReceipt,
    contracts::{
        BelvedereContract, DIFF_REVIEW_UI_IR_SECTION_ITEMS,
        DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION, DIFF_REVIEW_UI_IR_SECTION_SUMMARY,
        DiffReviewUiIrInput, ElicitUiDiffReviewProjectionContract, LandscapeDiffChangeKind,
        LandscapeDiffReviewAction, LandscapeDiffReviewItem, LandscapeDiffReviewModel,
        LandscapeDiffReviewPriority, TraceRef, TraceWidgetRef, UiIrProjection,
        UiIrProjectionBuilder, UiIrProofBundle, UiIrSection,
    },
    exchanges::{BelvedereExchange, UiIrError},
};

use super::{
    UI_IR_PRIMARY_VIEWPORT, minted_standards,
    receipt::{SourceViewModelReceipt, UiIrReceiptEvidenceBuilder, build_receipt_from_evidence},
    trace_widgets::labeled_widget,
    verify_viewports,
    widgets::section,
};

/// Exchange from landscape diff review evidence into `elicit_ui` IR.
#[derive(Debug, Clone, Copy, Default)]
pub struct ElicitUiDiffReviewProjection;

impl ElicitUiDiffReviewProjection {
    /// Project landscape diff review evidence into an `elicit_ui` verified tree.
    ///
    /// # Errors
    ///
    /// Returns an error if `elicit_ui` rejects any label, layout, keyboard,
    /// focus, target, or contrast proof obligation.
    #[instrument(level = "debug", skip(self, input))]
    pub fn project(
        &self,
        input: DiffReviewUiIrInput,
    ) -> Result<(UiIrProjection, UiIrReceipt), UiIrError> {
        <Self as BelvedereExchange<ElicitUiDiffReviewProjectionContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<ElicitUiDiffReviewProjectionContract> for ElicitUiDiffReviewProjection {
    type Error = UiIrError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: DiffReviewUiIrInput,
    ) -> Result<(UiIrProjection, UiIrReceipt), Self::Error> {
        let model = input.model();
        let backend = AccessKitUiBackend::new();
        let mut trace_widgets = Vec::new();
        let sections = project_diff_review_sections(&backend, model, &mut trace_widgets)?;
        let tree = backend.to_verified_tree(Viewport::new(
            UI_IR_PRIMARY_VIEWPORT.0,
            UI_IR_PRIMARY_VIEWPORT.1,
        ));
        let proofs = UiIrProofBundle::new(
            Established::<elicit_ui::IrSourced>::assert(),
            Established::<elicit_ui::WcagVerified>::prove(&tree),
        );
        let viewport_verifications = verify_viewports(&backend);
        let minted_standards = minted_standards();
        let receipt_evidence = UiIrReceiptEvidenceBuilder::default()
            .contract(ElicitUiDiffReviewProjectionContract::NAME.to_owned())
            .source_view_model_receipt(SourceViewModelReceipt::new(
                input.source_view_model_receipt_id(),
                input.source_view_model_receipt_digest(),
            ))
            .source_record_ids(model.source_record_ids().as_slice())
            .source_locator_counts(diff_review_source_locator_counts(model, &trace_widgets))
            .sections(sections.as_slice())
            .trace_widgets(trace_widgets.as_slice())
            .tree(&tree)
            .viewport_verifications(viewport_verifications.as_slice())
            .minted_standards(minted_standards.as_slice())
            .build()
            .map_err(|error| UiIrError::construction("UiIrReceiptEvidence", error))?;
        let receipt = build_receipt_from_evidence(receipt_evidence)?;
        let projection = UiIrProjectionBuilder::default()
            .tree(tree)
            .sections(sections)
            .trace_widgets(trace_widgets)
            .viewport_verifications(viewport_verifications)
            .minted_standards(minted_standards)
            .proofs(proofs)
            .build()
            .map_err(|error| UiIrError::construction("UiIrProjection", error))?;

        Ok((projection, receipt))
    }
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn project_diff_review_sections(
    backend: &AccessKitUiBackend,
    model: &LandscapeDiffReviewModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<Vec<UiIrSection>, UiIrError> {
    Ok(vec![
        diff_review_summary_section(backend, model, trace_widgets)?,
        diff_review_items_section(backend, model, trace_widgets)?,
        diff_review_source_navigation_section(backend, model, trace_widgets)?,
    ])
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn diff_review_summary_section(
    backend: &AccessKitUiBackend,
    model: &LandscapeDiffReviewModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let summary = model.summary();
    let widget = labeled_widget(
        backend,
        DIFF_REVIEW_UI_IR_SECTION_SUMMARY,
        "status",
        format!(
            "Landscape diff review: {} changed items; {} added, {} removed, {} changed; {} blocking, {} review, {} informational",
            summary.total_items(),
            summary.added_items(),
            summary.removed_items(),
            summary.changed_items(),
            summary.blocking_items(),
            summary.review_items(),
            summary.informational_items()
        ),
        "belvedere::landscape_diff_review",
        TraceRef::new(
            "belvedere::landscape_diff_review",
            model.source_record_ids().to_vec(),
        ),
        trace_widgets,
    )?;
    section(
        backend,
        DIFF_REVIEW_UI_IR_SECTION_SUMMARY,
        "diff review summary",
        vec![widget],
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn diff_review_items_section(
    backend: &AccessKitUiBackend,
    model: &LandscapeDiffReviewModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .items()
        .iter()
        .map(|item| diff_review_item_widget(backend, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        DIFF_REVIEW_UI_IR_SECTION_ITEMS,
        "diff review items",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn diff_review_item_widget(
    backend: &AccessKitUiBackend,
    item: &LandscapeDiffReviewItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<elicit_ui::WidgetId, UiIrError> {
    labeled_widget(
        backend,
        DIFF_REVIEW_UI_IR_SECTION_ITEMS,
        "button",
        format!(
            "{} {} {} has {} priority; action {}; source diff item {}; summary: {}",
            change_kind_label(item.change_kind()),
            item.subject_kind(),
            item.subject_id(),
            review_priority_label(item.review_priority()),
            review_action_label(item.review_action()),
            item.diff_item_id(),
            item.summary()
        ),
        item.diff_item_id(),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn diff_review_source_navigation_section(
    backend: &AccessKitUiBackend,
    model: &LandscapeDiffReviewModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .source_record_ids()
        .iter()
        .map(|source_record_id| {
            labeled_widget(
                backend,
                DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION,
                "link",
                format!("Source record {source_record_id} contributes to landscape diff review"),
                source_record_id,
                TraceRef::new(
                    source_record_id.to_owned(),
                    vec![source_record_id.to_owned()],
                ),
                trace_widgets,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION,
        "diff review source navigation",
        widgets,
    )
}

#[instrument(level = "debug", skip(model, trace_widgets))]
fn diff_review_source_locator_counts(
    model: &LandscapeDiffReviewModel,
    trace_widgets: &[TraceWidgetRef],
) -> BTreeMap<String, usize> {
    let navigation_items = model.source_record_ids().len();
    let located_widgets = trace_widgets
        .iter()
        .filter(|widget| widget.section_id() == DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION)
        .count();
    BTreeMap::from([
        ("ui_ir_source_navigation_items".to_owned(), navigation_items),
        ("ui_ir_source_locator_items".to_owned(), navigation_items),
        ("ui_ir_source_locator_widgets".to_owned(), located_widgets),
    ])
}

const fn change_kind_label(change_kind: LandscapeDiffChangeKind) -> &'static str {
    match change_kind {
        LandscapeDiffChangeKind::Added => "Added",
        LandscapeDiffChangeKind::Removed => "Removed",
        LandscapeDiffChangeKind::Changed => "Changed",
    }
}

const fn review_priority_label(review_priority: LandscapeDiffReviewPriority) -> &'static str {
    match review_priority {
        LandscapeDiffReviewPriority::Blocking => "blocking",
        LandscapeDiffReviewPriority::Review => "review",
        LandscapeDiffReviewPriority::Informational => "informational",
    }
}

const fn review_action_label(review_action: LandscapeDiffReviewAction) -> &'static str {
    match review_action {
        LandscapeDiffReviewAction::ReviewNewClaim => "review new claim",
        LandscapeDiffReviewAction::CompareClaimEvidence => "compare claim evidence",
        LandscapeDiffReviewAction::ConfirmClaimRemoval => "confirm claim removal",
        LandscapeDiffReviewAction::ReviewNewAssessment => "review new assessment",
        LandscapeDiffReviewAction::CompareAssessmentEvidence => "compare assessment evidence",
        LandscapeDiffReviewAction::ConfirmAssessmentRemoval => "confirm assessment removal",
        LandscapeDiffReviewAction::TriageNewGap => "triage new gap",
        LandscapeDiffReviewAction::ReassessChangedGap => "reassess changed gap",
        LandscapeDiffReviewAction::VerifyResolvedGap => "verify resolved gap",
    }
}
