//! Traceable widget construction for the `elicit_ui` projection exchange.

use elicit_ui::{
    AccessKitUiBackend, ContainerId, ContrastDescriptor, FocusDescriptor, KeyboardDescriptor,
    LabelDescriptor, SrgbColor, TargetDescriptor, UiLayoutManager, WcagContrastFactory,
    WcagFocusFactory, WcagKeyboardFactory, WcagLabelFactory, WcagTargetFactory, WidgetId,
};
use tracing::instrument;

use crate::contracts::{
    AssessmentQueueItem, AssessmentWorkItem, AuditorFilterFacet, BackendCoverageItem,
    ClaimExplorerItem, GapItem, ProofChainItem, ProvenanceRootItem, ResolutionPath,
    SourceNavigationItem, TraceRef, TraceWidgetRef, TraceWidgetRefBuilder, UiIrSection,
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
        format!("Filter {} selects {} items", item.label(), item.count()),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn backend_coverage_widget(
    backend: &AccessKitUiBackend,
    item: &BackendCoverageItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let status = if item.covered() { "covered" } else { "missing" };
    labeled_widget(
        backend,
        "status",
        format!(
            "Backend coverage {} for {} is {status}",
            item.verifier(),
            item.claim_id()
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
    let role = claim_role_label(item);
    labeled_widget(
        backend,
        "button",
        format!(
            "Claim {} is a {role}; {}, {}, {}, {}, {}",
            item.claim_id(),
            count_label(item.basis_ids().len(), "direct basis", "direct bases"),
            count_label(item.verifiers().len(), "verifier proof", "verifier proofs"),
            count_label(item.contract_count(), "contract", "contracts"),
            count_label(item.assessment_count(), "assessment", "assessments"),
            count_label(item.gap_count(), "gap", "gaps")
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "trace", skip(item))]
fn claim_role_label(item: &ClaimExplorerItem) -> &'static str {
    if item.root() {
        "provenance root"
    } else {
        "derived claim"
    }
}

#[instrument(level = "trace", skip(singular, plural))]
fn count_label(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn proof_chain_widget(
    backend: &AccessKitUiBackend,
    item: &ProofChainItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let roots = list_label(item.root_ids(), "no modeled roots");
    let bases = list_label(item.basis_ids(), "no direct bases");
    let verifiers = list_label(item.verifiers(), "no verifier proofs");
    labeled_widget(
        backend,
        "button",
        format!(
            "Proof chain {} reaches roots {roots} through direct bases {bases}; verifier proofs {verifiers}; {}",
            item.claim_id(),
            count_label(item.gap_count(), "gap", "gaps")
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "trace", skip(values))]
fn list_label(values: &[String], empty: &str) -> String {
    if values.is_empty() {
        empty.to_owned()
    } else {
        values.join(", ")
    }
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn assessment_widget(
    backend: &AccessKitUiBackend,
    item: &AssessmentQueueItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let recency = assessment_recency_label(item);
    let resolution = resolution_path_label(item.resolution_path());
    let weaknesses = weak_dimensions_label(item);
    labeled_widget(
        backend,
        "button",
        format!(
            "Assessment {} ({recency}) recommends {:?} for {}; resolution: {resolution}; {weaknesses}",
            item.assessment_id(),
            item.recommendation(),
            item.proof_id()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "trace", skip(item))]
fn weak_dimensions_label(item: &AssessmentQueueItem) -> String {
    if item.weak_dimensions().is_empty() {
        "no weak dimensions".to_owned()
    } else {
        format!("weak dimensions {}", item.weak_dimensions().join(", "))
    }
}

#[instrument(level = "debug", skip(path))]
fn resolution_path_label(path: Option<ResolutionPath>) -> &'static str {
    match path {
        Some(ResolutionPath::KeepCurrentProof) => "keep current proof",
        Some(ResolutionPath::StrengthenCurrentProof) => "strengthen current proof",
        Some(ResolutionPath::ReplaceWithProofSpecificModel) => "replace with proof-specific model",
        Some(ResolutionPath::ReplaceWithAccommodationModel) => "replace with accommodation model",
        Some(ResolutionPath::RetireClaim) => "retire claim",
        None => "no resolution path",
    }
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn assessment_work_widget(
    backend: &AccessKitUiBackend,
    item: &AssessmentWorkItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let priority = if item.high_priority() {
        "high priority"
    } else {
        "normal priority"
    };
    labeled_widget(
        backend,
        "button",
        format!(
            "Assessment work {} for {} ({priority}): {}",
            item.action(),
            item.claim_id(),
            item.reason()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "trace", skip(item))]
fn assessment_recency_label(item: &AssessmentQueueItem) -> String {
    if item.current() && item.superseded_count() == 0 {
        "current".to_owned()
    } else if item.current() && item.superseded_count() == 1 {
        "current, supersedes 1 older assessment".to_owned()
    } else if item.current() {
        format!(
            "current, supersedes {} older assessments",
            item.superseded_count()
        )
    } else {
        "superseded".to_owned()
    }
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
            "Witness {} {} for {} has {} trivial, {} checked, {} trusted, {} opaque supports",
            item.verifier(),
            item.support_kind(),
            item.claim_id(),
            item.trivial(),
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
            "Gap {:?} for {} is {} priority; next action: {}; detail: {}",
            item.kind(),
            item.claim_id(),
            item.severity().label(),
            item.action(),
            item.detail()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn source_navigation_widget(
    backend: &AccessKitUiBackend,
    item: &SourceNavigationItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "link",
        format!(
            "{} opens {} target {}",
            item.label(),
            item.source_kind(),
            item.target()
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
