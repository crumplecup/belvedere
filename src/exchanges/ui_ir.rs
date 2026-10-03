//! `elicit_ui` projection exchange implementations.

use std::collections::BTreeMap;

use elicit_ui::{AccessKitUiBackend, Viewport};
use elicitation::Established;
use tracing::instrument;

use crate::{
    certify::{UiIrReceipt, UiIrReceiptBuilder, UiIrReceiptCounts},
    contracts::{
        BelvedereContract, ElicitUiIrSourcedStandard, ElicitUiProjectionContract,
        ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
        ElicitUiWcagVerifiedStandard, TraceRef, TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_QUEUE,
        UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS,
        UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW, UI_IR_SECTION_PROOF_CHAINS,
        UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_SOURCE_NAVIGATION,
        UI_IR_SECTION_WITNESS_SURFACE, UiIrInput, UiIrProjection, UiIrProjectionBuilder,
        UiIrProofBundle, UiStandardMint,
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
        let mut sections = Vec::new();

        let overview = labeled_widget(
            &backend,
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
            &mut trace_widgets,
        )?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_OVERVIEW,
            "overview",
            vec![overview],
        )?);

        let filter_widgets = model
            .filter_facets()
            .iter()
            .map(|item| filter_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_FILTERS,
            "filters",
            filter_widgets,
        )?);

        let backend_widgets = model
            .backend_coverage()
            .iter()
            .map(|item| backend_coverage_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_BACKEND_COVERAGE,
            "backend coverage",
            backend_widgets,
        )?);

        let claim_widgets = model
            .claim_explorer()
            .iter()
            .map(|item| claim_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_CLAIM_EXPLORER,
            "claim explorer",
            claim_widgets,
        )?);

        let proof_chain_widgets = model
            .proof_chains()
            .iter()
            .map(|item| proof_chain_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_PROOF_CHAINS,
            "proof chains",
            proof_chain_widgets,
        )?);

        let assessment_widgets = model
            .assessment_queue()
            .iter()
            .map(|item| assessment_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        let assessment_work_widgets = model
            .assessment_work()
            .iter()
            .map(|item| assessment_work_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        let assessment_widgets = assessment_widgets
            .into_iter()
            .chain(assessment_work_widgets)
            .collect::<Vec<_>>();
        sections.push(section(
            &backend,
            UI_IR_SECTION_ASSESSMENT_QUEUE,
            "assessment queue",
            assessment_widgets,
        )?);

        let witness_widgets = model
            .witness_surface()
            .iter()
            .map(|item| witness_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_WITNESS_SURFACE,
            "witness surface",
            witness_widgets,
        )?);

        let root_widgets = model
            .provenance_roots()
            .iter()
            .map(|item| provenance_root_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_PROVENANCE_ROOTS,
            "provenance roots",
            root_widgets,
        )?);

        let gap_widgets = model
            .gaps()
            .iter()
            .map(|item| gap_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_GAP_QUEUE,
            "gap queue",
            gap_widgets,
        )?);

        let source_widgets = model
            .source_navigation()
            .iter()
            .map(|item| source_navigation_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(
            &backend,
            UI_IR_SECTION_SOURCE_NAVIGATION,
            "source navigation",
            source_widgets,
        )?);

        let tree = backend.to_verified_tree(Viewport::new(1440, 960));
        let proofs = UiIrProofBundle::new(
            Established::<elicit_ui::IrSourced>::assert(),
            Established::<elicit_ui::WcagVerified>::prove(&tree),
        );
        let minted_standards = minted_standards();
        let receipt = UiIrReceiptBuilder::default()
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
            .role_counts(role_counts(&trace_widgets))
            .minted_standards(
                minted_standards
                    .iter()
                    .map(|standard| standard.standard().to_owned())
                    .collect::<Vec<_>>(),
            )
            .build()
            .map_err(|error| UiIrError::construction("UiIrReceipt", error))?;
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

#[instrument(level = "debug")]
fn minted_standards() -> Vec<UiStandardMint> {
    vec![
        UiStandardMint::from_standard::<ElicitUiIrSourcedStandard>(),
        UiStandardMint::from_standard::<ElicitUiWcagVerifiedStandard>(),
        UiStandardMint::from_standard::<ElicitUiRenderCompleteStandard>(),
        UiStandardMint::from_standard::<ElicitUiRolePreservedStandard>(),
    ]
}

#[instrument(level = "debug", skip(trace_widgets))]
fn role_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for widget in trace_widgets {
        *counts.entry(format!("role_{}", widget.role())).or_default() += 1;
    }
    counts
}
