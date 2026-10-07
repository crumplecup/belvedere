//! Assessment queue and reviewer work items.

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

use crate::contracts::{Recommendation, ResolutionPath, view_model::TraceRef};

/// Assessment history grouped by proof ID.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct AssessmentHistoryItem {
    proof_id: String,
    current_assessment_id: String,
    assessment_ids: Vec<String>,
    reviewer_ids: Vec<String>,
    recommendation_flow: Vec<String>,
    weak_dimensions: Vec<String>,
    current_resolution_label: String,
    current_comment: String,
    #[getter(skip)]
    latest_timestamp: u64,
    #[getter(skip)]
    history_count: usize,
    #[getter(skip)]
    superseded_count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl AssessmentHistoryItem {
    /// Latest assessment timestamp in the history group.
    #[must_use]
    pub const fn latest_timestamp(&self) -> u64 {
        self.latest_timestamp
    }

    /// Number of assessments in this proof's review history.
    #[must_use]
    pub const fn history_count(&self) -> usize {
        self.history_count
    }

    /// Number of prior assessments superseded by the current assessment.
    #[must_use]
    pub const fn superseded_count(&self) -> usize {
        self.superseded_count
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Assessment queue item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct AssessmentQueueItem {
    assessment_id: String,
    proof_id: String,
    #[getter(skip)]
    current: bool,
    #[getter(skip)]
    superseded_count: usize,
    #[getter(skip)]
    superseded_by: Option<String>,
    #[getter(skip)]
    recommendation: Recommendation,
    queue_lane: String,
    #[getter(skip)]
    resolution_path: Option<ResolutionPath>,
    weak_dimensions: Vec<String>,
    #[getter(skip)]
    trace: TraceRef,
}

impl AssessmentQueueItem {
    /// Whether this is the latest known assessment for the proof.
    #[must_use]
    pub const fn current(&self) -> bool {
        self.current
    }

    /// Number of older assessment events superseded by this queue item.
    #[must_use]
    pub const fn superseded_count(&self) -> usize {
        self.superseded_count
    }

    /// Assessment ID that supersedes this item, if this item is no longer current.
    #[must_use]
    #[instrument(level = "trace", skip(self))]
    pub fn superseded_by(&self) -> Option<&str> {
        self.superseded_by.as_deref()
    }

    /// Recommendation.
    #[must_use]
    pub const fn recommendation(&self) -> Recommendation {
        self.recommendation
    }

    /// Resolution path.
    #[must_use]
    pub const fn resolution_path(&self) -> Option<ResolutionPath> {
        self.resolution_path
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Assessment work item for proofs or claims needing reviewer action.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct AssessmentWorkItem {
    claim_id: String,
    queue_lane: String,
    action: String,
    resolution_label: String,
    reason: String,
    #[getter(skip)]
    high_priority: bool,
    #[getter(skip)]
    trace: TraceRef,
}

impl AssessmentWorkItem {
    /// Whether this item should be elevated for auditor attention.
    #[must_use]
    pub const fn high_priority(&self) -> bool {
        self.high_priority
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}
