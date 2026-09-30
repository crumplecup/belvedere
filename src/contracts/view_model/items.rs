//! Visible auditor navigation items.

use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use tracing::instrument;

use crate::contracts::{
    ingestion::{Recommendation, ResolutionPath},
    landscape::LandscapeGapKind,
    view_model::TraceRef,
};

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

/// Claim explorer row/card evidence.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct ClaimExplorerItem {
    claim_id: String,
    #[getter(skip)]
    root: bool,
    basis_ids: Vec<String>,
    verifiers: Vec<String>,
    #[getter(skip)]
    contract_count: usize,
    #[getter(skip)]
    assessment_count: usize,
    #[getter(skip)]
    gap_count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl ClaimExplorerItem {
    /// Create a claim explorer item.
    #[must_use]
    #[instrument(level = "debug", skip(claim_id, counts, trace))]
    pub fn new(
        claim_id: impl Into<String>,
        root: bool,
        basis_ids: Vec<String>,
        verifiers: Vec<String>,
        counts: ClaimExplorerCounts,
        trace: TraceRef,
    ) -> Self {
        Self {
            claim_id: claim_id.into(),
            root,
            basis_ids,
            verifiers,
            contract_count: counts.contract_count(),
            assessment_count: counts.assessment_count(),
            gap_count: counts.gap_count(),
            trace,
        }
    }

    /// Whether this claim is a provenance root.
    #[must_use]
    pub const fn root(&self) -> bool {
        self.root
    }

    /// Number of contract fragments.
    #[must_use]
    pub const fn contract_count(&self) -> usize {
        self.contract_count
    }

    /// Number of assessments.
    #[must_use]
    pub const fn assessment_count(&self) -> usize {
        self.assessment_count
    }

    /// Number of gaps.
    #[must_use]
    pub const fn gap_count(&self) -> usize {
        self.gap_count
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Counts used when creating a claim explorer item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimExplorerCounts {
    contract_count: usize,
    assessment_count: usize,
    gap_count: usize,
}

impl ClaimExplorerCounts {
    /// Create claim explorer counts.
    #[must_use]
    pub const fn new(contract_count: usize, assessment_count: usize, gap_count: usize) -> Self {
        Self {
            contract_count,
            assessment_count,
            gap_count,
        }
    }

    /// Number of contract fragments.
    #[must_use]
    pub const fn contract_count(self) -> usize {
        self.contract_count
    }

    /// Number of assessments.
    #[must_use]
    pub const fn assessment_count(self) -> usize {
        self.assessment_count
    }

    /// Number of gaps.
    #[must_use]
    pub const fn gap_count(self) -> usize {
        self.gap_count
    }
}

/// Proof-chain navigation item for one claim.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ProofChainItem {
    claim_id: String,
    basis_ids: Vec<String>,
    root_ids: Vec<String>,
    verifiers: Vec<String>,
    #[getter(skip)]
    gap_count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl ProofChainItem {
    /// Number of gaps currently attached to the chain subject.
    #[must_use]
    pub const fn gap_count(&self) -> usize {
        self.gap_count
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

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
