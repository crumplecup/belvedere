//! Assessment-focused widget construction for the `elicit_ui` projection.

use elicit_ui::{AccessKitUiBackend, WidgetId};
use tracing::instrument;

use crate::contracts::{
    AssessmentHistoryItem, AssessmentQueueItem, AssessmentWorkItem, ResolutionPath, TraceWidgetRef,
};

use super::{
    UiIrError,
    widgets::{count_label, labeled_widget, list_label},
};

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn assessment_widget(
    backend: &AccessKitUiBackend,
    section_id: &'static str,
    item: &AssessmentQueueItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let recency = assessment_recency_label(item);
    let resolution = resolution_path_label(item.resolution_path());
    let weaknesses = weak_dimensions_label(item);
    labeled_widget(
        backend,
        section_id,
        "button",
        format!(
            "Assessment {} ({recency}) lane {} recommends {:?} for {}; resolution: {resolution}; {weaknesses}",
            item.assessment_id(),
            item.queue_lane(),
            item.recommendation(),
            item.proof_id()
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn assessment_history_widget(
    backend: &AccessKitUiBackend,
    section_id: &'static str,
    item: &AssessmentHistoryItem,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let assessments = list_label(item.assessment_ids(), "no assessments");
    let reviewers = list_label(item.reviewer_ids(), "no reviewers");
    let flow = list_label(item.recommendation_flow(), "no recommendations");
    let weaknesses = list_label(item.weak_dimensions(), "no weak dimensions");
    labeled_widget(
        backend,
        section_id,
        "button",
        format!(
            "Assessment history for {} current {} has {}; reviewers {}; recommendations {}; weak dimensions {}; resolution: {}; latest timestamp {}; comment: {}; {}, {}",
            item.proof_id(),
            item.current_assessment_id(),
            assessments,
            reviewers,
            flow,
            weaknesses,
            item.current_resolution_label(),
            item.latest_timestamp(),
            item.current_comment(),
            count_label(item.history_count(), "assessment", "assessments"),
            count_label(
                item.superseded_count(),
                "superseded assessment",
                "superseded assessments"
            )
        ),
        item.trace().clone(),
        trace_widgets,
    )
}

#[instrument(level = "debug", skip(backend, item, trace_widgets))]
pub(super) fn assessment_work_widget(
    backend: &AccessKitUiBackend,
    section_id: &'static str,
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
        section_id,
        "button",
        format!(
            "Assessment work {} lane {} for {} ({priority}); resolution: {}; reason: {}",
            item.action(),
            item.queue_lane(),
            item.claim_id(),
            item.resolution_label(),
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
    } else if let Some(current_id) = item.superseded_by() {
        format!("superseded by {current_id}")
    } else {
        "superseded".to_owned()
    }
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
