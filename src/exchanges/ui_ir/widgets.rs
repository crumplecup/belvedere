//! Traceable widget construction for the `elicit_ui` projection exchange.

use elicit_ui::{
    AccessKitUiBackend, ContainerId, ContrastDescriptor, FocusDescriptor, KeyboardDescriptor,
    LabelDescriptor, SrgbColor, TargetDescriptor, UiLayoutManager, WcagContrastFactory,
    WcagFocusFactory, WcagKeyboardFactory, WcagLabelFactory, WcagTargetFactory, WidgetId,
};
use tracing::instrument;

use crate::contracts::{
    AssessmentQueueItem, AuditorFilterFacet, ClaimExplorerItem, GapItem, ProofChainItem,
    ProvenanceRootItem, TraceRef, TraceWidgetRef, TraceWidgetRefBuilder, UiIrSection,
    WitnessSurfaceItem,
};

use super::UiIrError;

const ROOT_CONTAINER: ContainerId = ContainerId(0);

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn filter_widget(
    backend: &AccessKitUiBackend,
    item: &AuditorFilterFacet,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "checkbox",
        format!(
            "Filter {} {} selects {} items",
            item.group(),
            item.value(),
            item.count()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn claim_widget(
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
pub(super) fn proof_chain_widget(
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
pub(super) fn assessment_widget(
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
pub(super) fn witness_widget(
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
pub(super) fn provenance_root_widget(
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
pub(super) fn gap_widget(
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
pub(super) fn labeled_widget(
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
pub(super) fn section(
    backend: &AccessKitUiBackend,
    id: &'static str,
    name: &'static str,
    content: Vec<WidgetId>,
) -> Result<UiIrSection, UiIrError> {
    let container = backend
        .container_panel(name, content)
        .map_err(UiIrError::elicit_ui)?;
    backend
        .add_child(ROOT_CONTAINER, WidgetId(container.0))
        .map_err(UiIrError::elicit_ui)?;
    Ok(UiIrSection::new(id, name, container.0))
}
