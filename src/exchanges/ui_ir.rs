//! `elicit_ui` projection exchange implementations.

use elicit_ui::{AccessKitUiBackend, Viewport};
use elicitation::Established;
use tracing::instrument;

use crate::{
    certify::UiIrReceipt,
    contracts::{
        AuditorNavigationModel, ElicitUiIrSourcedStandard, ElicitUiProjectionContract,
        ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
        ElicitUiWcagVerifiedStandard, TraceRef, TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_HISTORY,
        UI_IR_SECTION_ASSESSMENT_QUEUE, UI_IR_SECTION_ASSESSMENT_WORK,
        UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_DETAIL, UI_IR_SECTION_CLAIM_EXPLORER,
        UI_IR_SECTION_FILTERS, UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW,
        UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_REVIEW_WORKFLOW,
        UI_IR_SECTION_SOURCE_NAVIGATION, UI_IR_SECTION_WITNESS_SURFACE, UiIrInput, UiIrProjection,
        UiIrProjectionBuilder, UiIrProofBundle, UiIrSection, UiStandardMint,
        UiViewportVerification,
    },
    exchanges::BelvedereExchange,
};

mod accessibility;
mod assessment_widgets;
mod diff_review;
mod errors;
mod metadata;
mod receipt;
mod trace_widgets;
mod widgets;

pub use diff_review::ElicitUiDiffReviewProjection;
pub use errors::UiIrError;
pub use metadata::ElicitUiMetadataProjection;

use assessment_widgets::{assessment_history_widget, assessment_widget, assessment_work_widget};
use receipt::{SourceViewModelReceipt, build_receipt};
use trace_widgets::labeled_widget;
use widgets::{
    backend_coverage_widget, claim_detail_widget, claim_widget, filter_widget, gap_widget,
    proof_chain_widget, provenance_root_widget, review_workflow_widget, section,
    source_navigation_widget, witness_widget,
};

pub(in crate::exchanges::ui_ir) const UI_IR_PRIMARY_VIEWPORT: (u32, u32) = (1440, 960);
pub(in crate::exchanges::ui_ir) const UI_IR_VERIFICATION_VIEWPORTS: &[(u32, u32)] =
    &[(390, 844), (768, 1024), UI_IR_PRIMARY_VIEWPORT];

/// Exchange from auditor navigation evidence into `elicit_ui` IR.
#[derive(Debug, Clone, Copy, Default)]
pub struct ElicitUiProjection;

impl ElicitUiProjection {
    /// Project auditor navigation evidence into an `elicit_ui` verified tree.
    ///
    /// # Errors
    ///
    /// Returns an error if `elicit_ui` rejects any label, layout, keyboard,
    /// focus, target, or contrast proof obligation.
    #[instrument(level = "debug", skip(self, input))]
    pub fn project(&self, input: UiIrInput) -> Result<(UiIrProjection, UiIrReceipt), UiIrError> {
        <Self as BelvedereExchange<ElicitUiProjectionContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<ElicitUiProjectionContract> for ElicitUiProjection {
    type Error = UiIrError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(&self, input: UiIrInput) -> Result<(UiIrProjection, UiIrReceipt), Self::Error> {
        let model = input.model();
        let backend = AccessKitUiBackend::new();
        let mut trace_widgets = Vec::new();
        let sections = project_sections(&backend, model, &mut trace_widgets)?;
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
        let receipt = build_receipt(
            model,
            SourceViewModelReceipt::new(
                input.source_view_model_receipt_id(),
                input.source_view_model_receipt_digest(),
            ),
            &sections,
            &trace_widgets,
            &tree,
            &viewport_verifications,
            &minted_standards,
        )?;
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
fn project_sections(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<Vec<UiIrSection>, UiIrError> {
    Ok(vec![
        overview_section(backend, model, trace_widgets)?,
        filter_section(backend, model, trace_widgets)?,
        backend_coverage_section(backend, model, trace_widgets)?,
        claim_explorer_section(backend, model, trace_widgets)?,
        claim_detail_section(backend, model, trace_widgets)?,
        proof_chains_section(backend, model, trace_widgets)?,
        review_workflow_section(backend, model, trace_widgets)?,
        assessment_history_section(backend, model, trace_widgets)?,
        assessment_queue_section(backend, model, trace_widgets)?,
        assessment_work_section(backend, model, trace_widgets)?,
        witness_surface_section(backend, model, trace_widgets)?,
        provenance_roots_section(backend, model, trace_widgets)?,
        gap_queue_section(backend, model, trace_widgets)?,
        source_navigation_section(backend, model, trace_widgets)?,
    ])
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn overview_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let overview = labeled_widget(
        backend,
        UI_IR_SECTION_OVERVIEW,
        "region",
        format!(
            "Overview: {} claims, {} proofs, {} assessments ({} current, {} superseded), {} gaps, {} high-priority gaps",
            model.overview().claim_count(),
            model.overview().proof_coverage_count(),
            model.overview().assessment_count(),
            model.overview().current_assessment_count(),
            model.overview().superseded_assessment_count(),
            model.overview().gap_count(),
            model.overview().high_priority_gap_count()
        ),
        "belvedere::overview",
        TraceRef::new("belvedere::overview", model.source_record_ids().to_vec()),
        trace_widgets,
    )?;
    section(backend, UI_IR_SECTION_OVERVIEW, "overview", vec![overview])
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn filter_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .filter_facets()
        .iter()
        .map(|item| filter_widget(backend, UI_IR_SECTION_FILTERS, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(backend, UI_IR_SECTION_FILTERS, "filters", widgets)
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn backend_coverage_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .backend_coverage()
        .iter()
        .map(|item| {
            backend_coverage_widget(backend, UI_IR_SECTION_BACKEND_COVERAGE, item, trace_widgets)
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_BACKEND_COVERAGE,
        "backend coverage",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn claim_explorer_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .claim_explorer()
        .iter()
        .map(|item| claim_widget(backend, UI_IR_SECTION_CLAIM_EXPLORER, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_CLAIM_EXPLORER,
        "claim explorer",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn claim_detail_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .claim_details()
        .iter()
        .map(|item| claim_detail_widget(backend, UI_IR_SECTION_CLAIM_DETAIL, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(backend, UI_IR_SECTION_CLAIM_DETAIL, "claim detail", widgets)
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn proof_chains_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .proof_chains()
        .iter()
        .map(|item| proof_chain_widget(backend, UI_IR_SECTION_PROOF_CHAINS, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(backend, UI_IR_SECTION_PROOF_CHAINS, "proof chains", widgets)
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn review_workflow_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .review_workflow()
        .iter()
        .map(|item| {
            review_workflow_widget(backend, UI_IR_SECTION_REVIEW_WORKFLOW, item, trace_widgets)
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_REVIEW_WORKFLOW,
        "review workflow",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn assessment_history_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .assessment_history()
        .iter()
        .map(|item| {
            assessment_history_widget(
                backend,
                UI_IR_SECTION_ASSESSMENT_HISTORY,
                item,
                trace_widgets,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_ASSESSMENT_HISTORY,
        "assessment history",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn assessment_queue_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let assessment_widgets = model
        .assessment_queue()
        .iter()
        .map(|item| assessment_widget(backend, UI_IR_SECTION_ASSESSMENT_QUEUE, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_ASSESSMENT_QUEUE,
        "assessment queue",
        assessment_widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn assessment_work_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .assessment_work()
        .iter()
        .map(|item| {
            assessment_work_widget(backend, UI_IR_SECTION_ASSESSMENT_WORK, item, trace_widgets)
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_ASSESSMENT_WORK,
        "assessment work",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn witness_surface_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .witness_surface()
        .iter()
        .map(|item| witness_widget(backend, UI_IR_SECTION_WITNESS_SURFACE, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_WITNESS_SURFACE,
        "witness surface",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn provenance_roots_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .provenance_roots()
        .iter()
        .map(|item| {
            provenance_root_widget(backend, UI_IR_SECTION_PROVENANCE_ROOTS, item, trace_widgets)
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_PROVENANCE_ROOTS,
        "provenance roots",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn gap_queue_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .gaps()
        .iter()
        .map(|item| gap_widget(backend, UI_IR_SECTION_GAP_QUEUE, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(backend, UI_IR_SECTION_GAP_QUEUE, "gap queue", widgets)
}

#[instrument(level = "debug", skip(backend, model, trace_widgets))]
fn source_navigation_section(
    backend: &AccessKitUiBackend,
    model: &AuditorNavigationModel,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<UiIrSection, UiIrError> {
    let widgets = model
        .source_navigation()
        .iter()
        .map(|item| {
            source_navigation_widget(
                backend,
                UI_IR_SECTION_SOURCE_NAVIGATION,
                item,
                trace_widgets,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_SOURCE_NAVIGATION,
        "source navigation",
        widgets,
    )
}

#[instrument(level = "debug", skip(backend))]
pub(in crate::exchanges::ui_ir) fn verify_viewports(
    backend: &AccessKitUiBackend,
) -> Vec<UiViewportVerification> {
    UI_IR_VERIFICATION_VIEWPORTS
        .iter()
        .map(|&(width, height)| {
            let tree = backend.to_verified_tree(Viewport::new(width, height));
            let _wcag_verified = Established::<elicit_ui::WcagVerified>::prove(&tree);
            UiViewportVerification::new(width, height, tree.nodes().len())
        })
        .collect()
}

#[instrument(level = "debug")]
pub(in crate::exchanges::ui_ir) fn minted_standards() -> Vec<UiStandardMint> {
    vec![
        UiStandardMint::from_standard::<ElicitUiIrSourcedStandard>(),
        UiStandardMint::from_standard::<ElicitUiWcagVerifiedStandard>(),
        UiStandardMint::from_standard::<ElicitUiRenderCompleteStandard>(),
        UiStandardMint::from_standard::<ElicitUiRolePreservedStandard>(),
    ]
}
