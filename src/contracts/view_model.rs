//! UI-neutral auditor navigation view-model contracts.

use derive_builder::Builder;
use derive_getters::Getters;
use tracing::instrument;

use crate::{
    certify::ViewModelReceipt,
    contracts::{BelvedereContract, LandscapeGraph},
};

mod items;
mod trace;

pub use items::{
    AssessmentQueueItem, AuditorFilterFacet, ClaimExplorerCounts, ClaimExplorerItem, GapItem,
    ProofChainItem, ProvenanceRootItem, WitnessSurfaceItem,
};
pub(crate) use items::{
    AssessmentQueueItemBuilder, AuditorFilterFacetBuilder, GapItemBuilder, ProofChainItemBuilder,
    WitnessSurfaceItemBuilder,
};
pub use trace::TraceRef;

/// Landscape evidence accepted by the view-model exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModelInput {
    landscape: LandscapeGraph,
}

impl ViewModelInput {
    /// Create view-model input from a landscape graph.
    #[must_use]
    pub const fn new(landscape: LandscapeGraph) -> Self {
        Self { landscape }
    }

    /// Landscape evidence to project.
    #[must_use]
    pub const fn landscape(&self) -> &LandscapeGraph {
        &self.landscape
    }
}

/// UI-neutral navigation evidence for certification auditors.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct AuditorNavigationModel {
    #[getter(skip)]
    overview: OverviewView,
    filter_facets: Vec<AuditorFilterFacet>,
    claim_explorer: Vec<ClaimExplorerItem>,
    proof_chains: Vec<ProofChainItem>,
    assessment_queue: Vec<AssessmentQueueItem>,
    witness_surface: Vec<WitnessSurfaceItem>,
    provenance_roots: Vec<ProvenanceRootItem>,
    gaps: Vec<GapItem>,
    source_record_ids: Vec<String>,
}

impl AuditorNavigationModel {
    /// Overview screen evidence.
    #[must_use]
    pub const fn overview(&self) -> &OverviewView {
        &self.overview
    }
}

/// Overview totals and high-priority review signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewView {
    claim_count: usize,
    root_count: usize,
    proof_coverage_count: usize,
    assessment_count: usize,
    gap_count: usize,
    high_priority_gap_count: usize,
}

impl OverviewView {
    /// Create overview totals.
    #[must_use]
    pub const fn new(
        claim_count: usize,
        root_count: usize,
        proof_coverage_count: usize,
        assessment_count: usize,
        gap_count: usize,
        high_priority_gap_count: usize,
    ) -> Self {
        Self {
            claim_count,
            root_count,
            proof_coverage_count,
            assessment_count,
            gap_count,
            high_priority_gap_count,
        }
    }

    /// Number of claims.
    #[must_use]
    pub const fn claim_count(&self) -> usize {
        self.claim_count
    }

    /// Number of roots.
    #[must_use]
    pub const fn root_count(&self) -> usize {
        self.root_count
    }

    /// Number of verifier coverage entries.
    #[must_use]
    pub const fn proof_coverage_count(&self) -> usize {
        self.proof_coverage_count
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

    /// Number of high-priority gaps.
    #[must_use]
    pub const fn high_priority_gap_count(&self) -> usize {
        self.high_priority_gap_count
    }
}

/// Auditor navigation view-model contract.
pub struct AuditorNavigationContract;

impl BelvedereContract for AuditorNavigationContract {
    type Input = ViewModelInput;
    type Output = AuditorNavigationModel;
    type Receipt = ViewModelReceipt;

    const NAME: &'static str = "belvedere.view_model.auditor_navigation";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "landscape graph was produced by the landscape modeling exchange",
            "claims, assessments, witness surfaces, roots, and gaps are traceable",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "overview totals are derived from landscape counts",
            "filter facets expose deterministic auditor narrowing axes with trace IDs",
            "claim explorer entries retain bases, verifier coverage, contracts, assessments, gaps, and trace IDs",
            "proof-chain entries retain direct bases, resolved roots, verifier coverage, gaps, and trace IDs",
            "assessment queue entries preserve recommendation and resolution vocabulary",
            "witness surface entries preserve support counts",
            "provenance root entries preserve explicit root identities",
            "gap entries preserve conservative gap kinds and trace IDs",
        ]
    }
}
