//! `elicit_ui` projection exchange implementations.

use std::{error::Error as StdError, panic::Location};

use elicit_ui::{
    AccessKitUiBackend, ContainerId, ContrastDescriptor, FocusDescriptor, KeyboardDescriptor,
    LabelDescriptor, SrgbColor, TargetDescriptor, UiLayoutManager, Viewport, WcagContrastFactory,
    WcagFocusFactory, WcagKeyboardFactory, WcagLabelFactory, WcagTargetFactory, WidgetId,
};
use elicitation::Established;
use thiserror::Error;
use tracing::instrument;

use crate::{
    certify::{UiIrReceipt, UiIrReceiptBuilder, UiIrReceiptCounts},
    contracts::{
        AssessmentQueueItem, BelvedereContract, ClaimExplorerItem, ElicitUiIrSourcedStandard,
        ElicitUiProjectionContract, ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
        ElicitUiWcagVerifiedStandard, GapItem, ProofChainItem, ProvenanceRootItem, TraceRef,
        TraceWidgetRef, TraceWidgetRefBuilder, UiIrInput, UiIrProjection, UiIrProjectionBuilder,
        UiIrProofBundle, UiIrSection, UiStandardMint, WitnessSurfaceItem,
    },
    error::BelvedereConstructionError,
    exchanges::BelvedereExchange,
};

const ROOT_CONTAINER: ContainerId = ContainerId(0);

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
                "Overview: {} claims, {} proofs, {} gaps",
                model.overview().claim_count(),
                model.overview().proof_coverage_count(),
                model.overview().gap_count()
            ),
            TraceRef::new("belvedere::overview", model.source_record_ids().to_vec()),
            &mut trace_widgets,
        )?;
        sections.push(section(&backend, "overview", vec![overview])?);

        let claim_widgets = model
            .claim_explorer()
            .iter()
            .map(|item| claim_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(&backend, "claim explorer", claim_widgets)?);

        let proof_chain_widgets = model
            .proof_chains()
            .iter()
            .map(|item| proof_chain_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(&backend, "proof chains", proof_chain_widgets)?);

        let assessment_widgets = model
            .assessment_queue()
            .iter()
            .map(|item| assessment_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(&backend, "assessment queue", assessment_widgets)?);

        let witness_widgets = model
            .witness_surface()
            .iter()
            .map(|item| witness_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(&backend, "witness surface", witness_widgets)?);

        let root_widgets = model
            .provenance_roots()
            .iter()
            .map(|item| provenance_root_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(&backend, "provenance roots", root_widgets)?);

        let gap_widgets = model
            .gaps()
            .iter()
            .map(|item| gap_widget(&backend, item, &mut trace_widgets))
            .collect::<Result<Vec<_>, _>>()?;
        sections.push(section(&backend, "gap queue", gap_widgets)?);

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

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn claim_widget(
    backend: &AccessKitUiBackend,
    item: &ClaimExplorerItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "button",
        format!(
            "Claim {} with {} verifier proofs and {} gaps",
            item.claim_id(),
            item.verifiers().len(),
            item.gap_count()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn proof_chain_widget(
    backend: &AccessKitUiBackend,
    item: &ProofChainItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "button",
        format!(
            "Proof chain {} reaches {} roots through {} direct bases with {} verifier proofs and {} gaps",
            item.claim_id(),
            item.root_ids().len(),
            item.basis_ids().len(),
            item.verifiers().len(),
            item.gap_count()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn assessment_widget(
    backend: &AccessKitUiBackend,
    item: &AssessmentQueueItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "button",
        format!(
            "Assessment {} recommends {:?} for {}",
            item.assessment_id(),
            item.recommendation(),
            item.proof_id()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn witness_widget(
    backend: &AccessKitUiBackend,
    item: &WitnessSurfaceItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "button",
        format!(
            "Witness {} for {} has {} checked, {} trusted, {} opaque supports",
            item.verifier(),
            item.claim_id(),
            item.checked(),
            item.trusted(),
            item.opaque()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn provenance_root_widget(
    backend: &AccessKitUiBackend,
    item: &ProvenanceRootItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "link",
        format!("Provenance root {}", item.claim_id()),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
fn gap_widget(
    backend: &AccessKitUiBackend,
    item: &GapItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "button",
        format!(
            "Gap {:?} for {}: {}",
            item.kind(),
            item.claim_id(),
            item.detail()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, trace, trace_widgets))]
fn labeled_widget(
    backend: &AccessKitUiBackend,
    role: &str,
    label: String,
    trace: TraceRef,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let (element, _proof) = backend
        .build_labeled_element(LabelDescriptor {
            name: label.clone(),
            role: role.to_owned(),
            labelled_by: None,
        })
        .map_err(UiIrError::elicit_ui)?;
    if matches!(
        role,
        "button" | "link" | "checkbox" | "radio" | "text-input" | "search-input"
    ) {
        establish_interactive_proofs(backend, element.id)?;
    }
    backend
        .build_contrast_minimum(ContrastDescriptor {
            foreground: SrgbColor::from_u8(0x11, 0x18, 0x27),
            background: SrgbColor::from_u8(0xf8, 0xfa, 0xfc),
            widget: Some(element.id),
        })
        .map_err(UiIrError::elicit_ui)?;
    let trace_widget = TraceWidgetRefBuilder::default()
        .widget_id(element.id.0)
        .label(label)
        .claim_id(trace.claim_id())
        .source_record_ids(trace.source_record_ids().to_vec())
        .build()
        .map_err(|error| UiIrError::construction("TraceWidgetRef", error))?;
    trace_widgets.push(trace_widget);
    Ok(element.id)
}

#[instrument(level = "debug", skip(backend, widget))]
fn establish_interactive_proofs(
    backend: &AccessKitUiBackend,
    widget: WidgetId,
) -> Result<(), UiIrError> {
    backend
        .build_keyboard_accessible(KeyboardDescriptor {
            widget,
            tab_index: 0,
            shortcut: None,
        })
        .map_err(UiIrError::elicit_ui)?;
    backend
        .build_target_minimum(TargetDescriptor {
            widget,
            width_px: 44.0,
            height_px: 44.0,
            adjacent_spacing_px: 8.0,
        })
        .map_err(UiIrError::elicit_ui)?;
    backend
        .build_focus_visible(FocusDescriptor {
            widget,
            indicator_area_px: 88.0,
            indicator_contrast: 3.0,
        })
        .map_err(UiIrError::elicit_ui)?;
    Ok(())
}

#[instrument(level = "debug", skip(backend, content))]
fn section(
    backend: &AccessKitUiBackend,
    name: &'static str,
    content: Vec<WidgetId>,
) -> Result<UiIrSection, UiIrError> {
    let container = backend
        .container_panel(name, content)
        .map_err(UiIrError::elicit_ui)?;
    backend
        .add_child(ROOT_CONTAINER, WidgetId(container.0))
        .map_err(UiIrError::elicit_ui)?;
    Ok(UiIrSection::new(name, container.0))
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

/// Errors raised while projecting into `elicit_ui`.
#[derive(Debug, Error)]
#[error("UI IR projection failed at {file}:{line}: {kind}")]
pub struct UiIrError {
    #[source]
    kind: Box<UiIrErrorKind>,
    file: String,
    line: u32,
}

impl UiIrError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn elicit_ui(source: elicit_ui::UiError) -> Self {
        Self::new(UiIrErrorKind::ElicitUi(ElicitUiProjectionError::new(
            source,
        )))
    }

    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn construction<E>(target: &'static str, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::new(UiIrErrorKind::Construction(
            BelvedereConstructionError::new(target, source),
        ))
    }

    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: UiIrErrorKind) -> Self {
        let location = Location::caller();
        Self {
            kind: Box::new(kind),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
enum UiIrErrorKind {
    #[error(transparent)]
    ElicitUi(ElicitUiProjectionError),
    #[error(transparent)]
    Construction(BelvedereConstructionError),
}

#[derive(Debug, Error)]
#[error("elicit_ui rejected UI IR projection at {file}:{line}: {source}")]
struct ElicitUiProjectionError {
    source: elicit_ui::UiError,
    file: String,
    line: u32,
}

impl ElicitUiProjectionError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: elicit_ui::UiError) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}
