//! `elicit_ui` projection exchange implementations.

use std::collections::{BTreeMap, BTreeSet};

use elicit_ui::{AccessKitUiBackend, Viewport};
use elicitation::Established;
use tracing::instrument;

use crate::{
    certify::{UiIrReceipt, UiIrReceiptBuilder, UiIrReceiptCounts},
    contracts::{
        AuditorNavigationModel, BelvedereContract, ElicitUiIrSourcedStandard,
        ElicitUiProjectionContract, ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
        ElicitUiWcagVerifiedStandard, TraceRef, TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_QUEUE,
        UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS,
        UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW, UI_IR_SECTION_PROOF_CHAINS,
        UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_SOURCE_NAVIGATION,
        UI_IR_SECTION_WITNESS_SURFACE, UiIrInput, UiIrProjection, UiIrProjectionBuilder,
        UiIrProofBundle, UiIrSection, UiStandardMint,
    },
    exchanges::BelvedereExchange,
};

mod errors;
mod widgets;

pub use errors::UiIrError;

use widgets::{
    assessment_widget, assessment_work_widget, backend_coverage_widget, claim_widget,
    filter_widget, gap_widget, labeled_widget, proof_chain_widget, provenance_root_widget, section,
    source_navigation_widget, witness_widget,
};

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
        let tree = backend.to_verified_tree(Viewport::new(1440, 960));
        let proofs = UiIrProofBundle::new(
            Established::<elicit_ui::IrSourced>::assert(),
            Established::<elicit_ui::WcagVerified>::prove(&tree),
        );
        let minted_standards = minted_standards();
        let receipt = build_receipt(model, &sections, &trace_widgets, &tree, &minted_standards)?;
        let projection = UiIrProjectionBuilder::default()
            .tree(tree)
            .sections(sections)
            .trace_widgets(trace_widgets)
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
        proof_chains_section(backend, model, trace_widgets)?,
        assessment_queue_section(backend, model, trace_widgets)?,
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
        .map(|item| filter_widget(backend, item, trace_widgets))
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
        .map(|item| backend_coverage_widget(backend, item, trace_widgets))
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
        .map(|item| claim_widget(backend, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_CLAIM_EXPLORER,
        "claim explorer",
        widgets,
    )
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
        .map(|item| proof_chain_widget(backend, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(backend, UI_IR_SECTION_PROOF_CHAINS, "proof chains", widgets)
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
        .map(|item| assessment_widget(backend, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    let assessment_work_widgets = model
        .assessment_work()
        .iter()
        .map(|item| assessment_work_widget(backend, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    let widgets = assessment_widgets
        .into_iter()
        .chain(assessment_work_widgets)
        .collect::<Vec<_>>();
    section(
        backend,
        UI_IR_SECTION_ASSESSMENT_QUEUE,
        "assessment queue",
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
        .map(|item| witness_widget(backend, item, trace_widgets))
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
        .map(|item| provenance_root_widget(backend, item, trace_widgets))
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
        .map(|item| gap_widget(backend, item, trace_widgets))
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
        .map(|item| source_navigation_widget(backend, item, trace_widgets))
        .collect::<Result<Vec<_>, _>>()?;
    section(
        backend,
        UI_IR_SECTION_SOURCE_NAVIGATION,
        "source navigation",
        widgets,
    )
}

#[instrument(
    level = "debug",
    skip(model, sections, trace_widgets, tree, minted_standards)
)]
fn build_receipt(
    model: &AuditorNavigationModel,
    sections: &[UiIrSection],
    trace_widgets: &[TraceWidgetRef],
    tree: &elicit_ui::VerifiedTree,
    minted_standards: &[UiStandardMint],
) -> Result<UiIrReceipt, UiIrError> {
    UiIrReceiptBuilder::default()
        .contract(ElicitUiProjectionContract::NAME)
        .counts(UiIrReceiptCounts::new(
            sections.len(),
            trace_widgets.len(),
            trace_widgets
                .iter()
                .map(|widget| widget.source_record_ids().len())
                .sum(),
            tree.nodes().len(),
        ))
        .source_record_ids(model.source_record_ids().to_vec())
        .section_ids(
            sections
                .iter()
                .map(|section| section.id().to_owned())
                .collect::<Vec<_>>(),
        )
        .section_root_widget_ids(
            sections
                .iter()
                .map(UiIrSection::root_widget_id)
                .collect::<Vec<_>>(),
        )
        .trace_widget_ids(
            trace_widgets
                .iter()
                .map(TraceWidgetRef::widget_id)
                .collect::<Vec<_>>(),
        )
        .role_counts(role_counts(trace_widgets))
        .section_widget_counts(section_widget_counts(sections))
        .trace_source_counts(trace_source_counts(trace_widgets))
        .minted_prop_counts(minted_prop_counts(minted_standards))
        .proof_token_counts(proof_token_counts())
        .minted_standards(
            minted_standards
                .iter()
                .map(|standard| standard.standard().to_owned())
                .collect::<Vec<_>>(),
        )
        .build()
        .map_err(|error| UiIrError::construction("UiIrReceipt", error))
}

#[instrument(level = "debug")]
fn minted_standards() -> Vec<UiStandardMint> {
    vec![
        UiStandardMint::from_standard::<ElicitUiIrSourcedStandard>(),
        UiStandardMint::from_standard::<ElicitUiWcagVerifiedStandard>(),
        UiStandardMint::from_standard::<ElicitUiRenderCompleteStandard>(),
        UiStandardMint::from_standard::<ElicitUiRolePreservedStandard>(),
    ]
}

#[instrument(level = "debug")]
fn proof_token_counts() -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("ui_ir_proof_token_ir_sourced".to_owned(), 1),
        ("ui_ir_proof_token_wcag_verified".to_owned(), 1),
    ])
}

#[instrument(level = "debug", skip(minted_standards))]
fn minted_prop_counts(minted_standards: &[UiStandardMint]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for mint in minted_standards {
        let prop = mint.prop().as_str();
        let prop_name = prop.strip_prefix("elicit_ui::").unwrap_or(prop);
        *counts
            .entry(format!("minted_elicit_ui_prop_{prop_name}"))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(trace_widgets))]
fn role_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for widget in trace_widgets {
        *counts.entry(format!("role_{}", widget.role())).or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(sections))]
fn section_widget_counts(sections: &[UiIrSection]) -> BTreeMap<String, usize> {
    sections
        .iter()
        .map(|section| (section.id().to_owned(), section.widget_count()))
        .collect()
}

#[instrument(level = "debug", skip(trace_widgets))]
fn trace_source_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
    let mut unique_source_records = BTreeSet::new();
    let mut widgets_with_sources = 0;
    let mut source_links = 0;
    for widget in trace_widgets {
        if !widget.source_record_ids().is_empty() {
            widgets_with_sources += 1;
        }
        source_links += widget.source_record_ids().len();
        unique_source_records.extend(widget.source_record_ids().iter().cloned());
    }
    BTreeMap::from([
        (
            "trace_widgets_with_sources".to_owned(),
            widgets_with_sources,
        ),
        (
            "trace_unique_source_records".to_owned(),
            unique_source_records.len(),
        ),
        ("trace_source_links".to_owned(), source_links),
    ])
}
