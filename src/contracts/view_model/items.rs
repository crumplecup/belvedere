//! Visible auditor navigation items.

use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;

use crate::contracts::{landscape::LandscapeGapKind, view_model::TraceRef};

mod assessment;
mod claim;

pub use assessment::{AssessmentQueueItem, AssessmentWorkItem};
pub(crate) use assessment::{AssessmentQueueItemBuilder, AssessmentWorkItemBuilder};
pub(crate) use claim::ProofChainItemBuilder;
pub use claim::{ClaimExplorerCounts, ClaimExplorerItem, ProofChainItem};

/// Auditor filter facet for narrowing the displayed defensive landscape.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
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
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct SourceNavigationItem {
    source_record_id: String,
    label: String,
    #[getter(skip)]
    trace: TraceRef,
}

impl SourceNavigationItem {
    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Backend coverage status for one claim and verifier.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
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
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct WitnessSurfaceItem {
    claim_id: String,
    verifier: String,
    destination_module: String,
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
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct ProvenanceRootItem {
    #[new(into)]
    claim_id: String,
    #[getter(skip)]
    trace: TraceRef,
}

impl ProvenanceRootItem {
    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Gap item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct GapItem {
    claim_id: String,
    #[getter(skip)]
    kind: LandscapeGapKind,
    detail: String,
    #[getter(skip)]
    high_priority: bool,
    #[getter(skip)]
    trace: TraceRef,
}

impl GapItem {
    /// Gap kind.
    #[must_use]
    pub const fn kind(&self) -> LandscapeGapKind {
        self.kind
    }

    /// Whether this item should be elevated in overview.
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
