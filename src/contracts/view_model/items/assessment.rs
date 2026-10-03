//! Assessment queue and reviewer work items.

use derive_builder::Builder;
use derive_getters::Getters;

use crate::contracts::{Recommendation, ResolutionPath, view_model::TraceRef};

/// Assessment queue item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct AssessmentQueueItem {
    assessment_id: String,
    proof_id: String,
    #[getter(skip)]
    current: bool,
    #[getter(skip)]
    superseded_count: usize,
    #[getter(skip)]
    recommendation: Recommendation,
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
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
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
