//! Review workflow items for auditor task navigation.

use derive_builder::Builder;
use derive_getters::Getters;

use crate::contracts::view_model::TraceRef;

/// Auditor-facing priority for a review workflow task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReviewWorkflowPriority {
    /// Task blocks certification review until resolved.
    Blocking,
    /// Task should be elevated before confidence can increase.
    Elevated,
    /// Task is ready for normal auditor review.
    Ready,
}

impl ReviewWorkflowPriority {
    /// Stable auditor-facing priority label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::Elevated => "elevated",
            Self::Ready => "ready",
        }
    }
}

/// Workflow item that turns modeled review signals into an auditor task.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ReviewWorkflowItem {
    task_id: String,
    claim_id: String,
    lane: String,
    action: String,
    rationale: String,
    #[getter(skip)]
    priority: ReviewWorkflowPriority,
    #[getter(skip)]
    source_count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl ReviewWorkflowItem {
    /// Task priority.
    #[must_use]
    pub const fn priority(&self) -> ReviewWorkflowPriority {
        self.priority
    }

    /// Number of source records linked to this review task.
    #[must_use]
    pub const fn source_count(&self) -> usize {
        self.source_count
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}
