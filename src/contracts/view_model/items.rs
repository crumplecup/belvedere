//! Visible auditor navigation items.

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;

use crate::contracts::{ArtifactKind, view_model::TraceRef};

mod assessment;
mod claim;
mod gap;
mod workflow;

pub use assessment::{AssessmentHistoryItem, AssessmentQueueItem, AssessmentWorkItem};
pub(crate) use assessment::{
    AssessmentHistoryItemBuilder, AssessmentQueueItemBuilder, AssessmentWorkItemBuilder,
};
pub use claim::{
    ClaimDetailItem, ClaimExplorerCounts, ClaimExplorerItem, ProofChainBackendCoverage,
    ProofChainItem, ProofChainTopology,
};
pub(crate) use claim::{
    ClaimDetailItemBuilder, ProofChainBackendCoverageBuilder, ProofChainItemBuilder,
};
pub(crate) use gap::GapItemBuilder;
pub use gap::{GapItem, GapSeverity};
pub(crate) use workflow::ReviewWorkflowItemBuilder;
pub use workflow::{ReviewWorkflowItem, ReviewWorkflowPriority};

/// Auditor filter facet for narrowing the displayed defensive landscape.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct AuditorFilterFacet {
    facet_id: String,
    group: String,
    value: String,
    label: String,
    #[getter(skip)]
    count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl AuditorFilterFacet {
    /// Number of visible items selected by this facet.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Source record navigation item for jumping back to ingested evidence.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct SourceNavigationItem {
    source_record_id: String,
    #[getter(skip)]
    artifact_kind: ArtifactKind,
    source_kind: String,
    target: String,
    source_locator: String,
    label: String,
    #[getter(skip)]
    trace: TraceRef,
}

impl SourceNavigationItem {
    /// Ingested artifact family that produced this source record.
    #[must_use]
    pub const fn artifact_kind(&self) -> ArtifactKind {
        self.artifact_kind
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Backend coverage status for one claim and verifier.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct BackendCoverageItem {
    claim_id: String,
    verifier: String,
    #[getter(skip)]
    covered: bool,
    #[getter(skip)]
    trace: TraceRef,
}

impl BackendCoverageItem {
    /// Whether this verifier has modeled proof coverage for the claim.
    #[must_use]
    pub const fn covered(&self) -> bool {
        self.covered
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Witness surface item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct WitnessSurfaceItem {
    claim_id: String,
    verifier: String,
    destination_module: String,
    support_kind: String,
    #[getter(skip)]
    trivial: usize,
    #[getter(skip)]
    checked: usize,
    #[getter(skip)]
    trusted: usize,
    #[getter(skip)]
    opaque: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl WitnessSurfaceItem {
    /// Trivial support count.
    #[must_use]
    pub const fn trivial(&self) -> usize {
        self.trivial
    }

    /// Checked support count.
    #[must_use]
    pub const fn checked(&self) -> usize {
        self.checked
    }

    /// Trusted support count.
    #[must_use]
    pub const fn trusted(&self) -> usize {
        self.trusted
    }

    /// Opaque support count.
    #[must_use]
    pub const fn opaque(&self) -> usize {
        self.opaque
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Provenance root item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ProvenanceRootItem {
    claim_id: String,
    root_kind: String,
    verifiers: Vec<String>,
    assessment_ids: Vec<String>,
    gap_summaries: Vec<String>,
    next_action: String,
    #[getter(skip)]
    verifier_count: usize,
    #[getter(skip)]
    assessment_count: usize,
    #[getter(skip)]
    gap_count: usize,
    #[getter(skip)]
    source_record_count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl ProvenanceRootItem {
    /// Number of verifier proofs attached to this root.
    #[must_use]
    pub const fn verifier_count(&self) -> usize {
        self.verifier_count
    }

    /// Number of assessments attached to this root.
    #[must_use]
    pub const fn assessment_count(&self) -> usize {
        self.assessment_count
    }

    /// Number of modeled gaps attached to this root.
    #[must_use]
    pub const fn gap_count(&self) -> usize {
        self.gap_count
    }

    /// Number of source records available through the trace.
    #[must_use]
    pub const fn source_record_count(&self) -> usize {
        self.source_record_count
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}
