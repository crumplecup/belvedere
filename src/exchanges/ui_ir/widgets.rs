//! Traceable widget construction for the `elicit_ui` projection exchange.

use elicit_ui::{
    AccessKitUiBackend, ContainerId, ContrastDescriptor, LabelDescriptor, SrgbColor,
    UiLayoutManager, WcagContrastFactory, WcagLabelFactory, WidgetId,
};
use tracing::instrument;

use crate::contracts::{
    AuditorFilterFacet, BackendCoverageItem, ClaimDetailItem, ClaimExplorerItem, GapItem,
    ProofChainItem, ProvenanceRootItem, ReviewWorkflowItem, SourceNavigationItem, TraceRef,
    TraceWidgetRef, TraceWidgetRefBuilder, UiIrSection, WitnessSurfaceItem,
};

use super::{UiIrError, accessibility::establish_interactive_proofs};

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
    let filters = list_label(item.filter_ids(), "no filters");
    labeled_widget(
        backend,
        "button",
        format!(
            "Claim {} is a {role}; {}, {}, {}, {}, {}; filters {filters}",
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
pub(super) fn count_label(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn claim_detail_widget(
    backend: &AccessKitUiBackend,
    item: &ClaimDetailItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let role = claim_detail_role_label(item);
    let bases = list_label(item.basis_ids(), "no direct bases");
    let roots = list_label(item.root_ids(), "no modeled roots");
    let verifiers = list_label(item.verifiers(), "no verifier proofs");
    let contracts = list_label(item.contract_fragments(), "no contract fragments");
    let proof_statuses = list_label(item.proof_statuses(), "no proof execution statuses");
    let assessments = list_label(item.assessment_ids(), "no assessments");
    let gaps = list_label(item.gap_summaries(), "no gaps");
    labeled_widget(
        backend,
        "button",
        format!(
            "Claim detail {} is a {role}; direct bases {bases}; resolved roots {roots}; verifier proofs {verifiers}; proof statuses {proof_statuses}; contracts {contracts}; assessments {assessments}; gaps {gaps}; review state {}; next action: {}; {}",
            item.claim_id(),
            item.review_state(),
            item.next_action(),
            count_label(
                item.source_record_count(),
                "source record",
                "source records"
            )
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "trace", skip(item))]
fn claim_detail_role_label(item: &ClaimDetailItem) -> &'static str {
    if item.root() {
        "provenance root"
    } else {
        "derived claim"
    }
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
    let proof_statuses = list_label(item.proof_statuses(), "no proof execution statuses");
    let backend_coverage = chain_backend_coverage_label(item);
    let topology = item.topology();
    labeled_widget(
        backend,
        "button",
        format!(
            "Proof chain {} is {}; next action: {}; reaches roots {roots} through direct bases {bases}; verifier proofs {verifiers}; proof statuses {proof_statuses}; chain backend coverage {backend_coverage}; topology {}, {}, {}, {}; subject {}",
            item.claim_id(),
            item.review_state(),
            item.next_action(),
            count_label(
                topology.transitive_claim_count(),
                "linked claim",
                "linked claims"
            ),
            count_label(topology.max_depth(), "level deep", "levels deep"),
            count_label(
                topology.branching_claim_count(),
                "branching claim",
                "branching claims"
            ),
            count_label(topology.chain_gap_count(), "chain gap", "chain gaps"),
            count_label(item.gap_count(), "gap", "gaps")
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "trace", skip(item))]
fn chain_backend_coverage_label(item: &ProofChainItem) -> String {
    item.backend_coverage()
        .iter()
        .map(|coverage| {
            let state = if coverage.complete() {
                "complete"
            } else {
                "incomplete"
            };
            format!(
                "{} {state} ({} covered, {} missing)",
                coverage.verifier(),
                coverage.covered_claim_count(),
                coverage.missing_claim_count()
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[instrument(level = "trace", skip(values))]
pub(super) fn list_label(values: &[String], empty: &str) -> String {
    if values.is_empty() {
        empty.to_owned()
    } else {
        values.join(", ")
    }
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn review_workflow_widget(
    backend: &AccessKitUiBackend,
    item: &ReviewWorkflowItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    labeled_widget(
        backend,
        "button",
        format!(
            "Review task {} for {} lane {} priority {}; action: {}; rationale: {}; {}",
            item.task_id(),
            item.claim_id(),
            item.lane(),
            item.priority().label(),
            item.action(),
            item.rationale(),
            count_label(item.source_count(), "source record", "source records")
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
    let verifiers = list_label(item.verifiers(), "no verifier proofs");
    let assessments = list_label(item.assessment_ids(), "no assessments");
    let gaps = list_label(item.gap_summaries(), "no gaps");
    labeled_widget(
        backend,
        "link",
        format!(
            "Provenance root {} is {}; verifier proofs {verifiers}; assessments {assessments}; gaps {gaps}; next action: {}; {}, {}, {}, {}",
            item.claim_id(),
            item.root_kind(),
            item.next_action(),
            count_label(item.verifier_count(), "verifier proof", "verifier proofs"),
            count_label(item.assessment_count(), "assessment", "assessments"),
            count_label(item.gap_count(), "gap", "gaps"),
            count_label(
                item.source_record_count(),
                "source record",
                "source records"
            )
        ),
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
            "Gap {:?} for {} in {} lane is {} priority; next action: {}; detail: {}",
            item.kind(),
            item.claim_id(),
            item.queue_lane(),
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
            "{} opens {} {} target {}",
            item.label(),
            item.artifact_kind(),
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
        .role(role)
        .label(label)
        .claim_id(trace.claim_id())
        .source_record_ids(trace.source_record_ids().to_vec())
        .build()
        .map_err(|error| UiIrError::construction("TraceWidgetRef", error))?;
    trace_widgets.push(trace_widget);
    Ok(element.id)
}

#[instrument(level = "debug", skip(backend, content))]
pub(super) fn section(
    backend: &AccessKitUiBackend,
    id: &'static str,
    name: &'static str,
    content: Vec<WidgetId>,
) -> Result<UiIrSection, UiIrError> {
    let widget_count = content.len();
    let container = backend
        .container_panel(name, content)
        .map_err(UiIrError::elicit_ui)?;
    backend
        .add_child(ROOT_CONTAINER, WidgetId(container.0))
        .map_err(UiIrError::elicit_ui)?;
    Ok(UiIrSection::new(id, name, container.0, widget_count))
}
