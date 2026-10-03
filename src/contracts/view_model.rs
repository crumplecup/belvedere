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
    AssessmentQueueItem, AssessmentWorkItem, AuditorFilterFacet, BackendCoverageItem,
    ClaimExplorerCounts, ClaimExplorerItem, GapItem, GapSeverity, ProofChainBackendCoverage,
    ProofChainItem, ProofChainTopology, ProvenanceRootItem, SourceNavigationItem,
    WitnessSurfaceItem,
};
pub(crate) use items::{
    AssessmentQueueItemBuilder, AssessmentWorkItemBuilder, AuditorFilterFacetBuilder,
    BackendCoverageItemBuilder, GapItemBuilder, ProofChainBackendCoverageBuilder,
    ProofChainItemBuilder, SourceNavigationItemBuilder, WitnessSurfaceItemBuilder,
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
    backend_coverage: Vec<BackendCoverageItem>,
    claim_explorer: Vec<ClaimExplorerItem>,
    proof_chains: Vec<ProofChainItem>,
    assessment_queue: Vec<AssessmentQueueItem>,
    assessment_work: Vec<AssessmentWorkItem>,
    witness_surface: Vec<WitnessSurfaceItem>,
    provenance_roots: Vec<ProvenanceRootItem>,
    gaps: Vec<GapItem>,
    source_navigation: Vec<SourceNavigationItem>,
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
    assessment_recency: AssessmentRecencySummary,
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
        assessment_recency: AssessmentRecencySummary,
        gap_count: usize,
        high_priority_gap_count: usize,
    ) -> Self {
        Self {
            claim_count,
            root_count,
            proof_coverage_count,
            assessment_recency,
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
        self.assessment_recency.total()
    }

    /// Number of assessments that are current for their proof.
    #[must_use]
    pub const fn current_assessment_count(&self) -> usize {
        self.assessment_recency.current_count()
    }

    /// Number of assessments that have been superseded by newer reviews.
    #[must_use]
    pub const fn superseded_assessment_count(&self) -> usize {
        self.assessment_recency.superseded_count()
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

/// Assessment recency totals surfaced in the overview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssessmentRecencySummary {
    current_count: usize,
    superseded_count: usize,
}

impl AssessmentRecencySummary {
    /// Create assessment recency totals.
    #[must_use]
    pub const fn new(current_count: usize, superseded_count: usize) -> Self {
        Self {
            current_count,
            superseded_count,
        }
    }

    /// Number of current assessments.
    #[must_use]
    pub const fn current_count(self) -> usize {
        self.current_count
    }

    /// Number of superseded assessments.
    #[must_use]
    pub const fn superseded_count(self) -> usize {
        self.superseded_count
    }

    /// Total number of assessments.
    #[must_use]
    pub const fn total(self) -> usize {
        self.current_count + self.superseded_count
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
            "filter facets expose deterministic claim-family, verifier, recommendation, support, package, gap, and priority axes with trace IDs",
            "backend coverage entries expose Kani, Creusot, and Verus status per claim",
            "claim explorer entries retain bases, verifier coverage, filter memberships, contracts, assessments, gaps, and trace IDs",
            "proof-chain entries retain direct bases, resolved roots, subject verifier coverage, chain backend completeness, topology metrics, review state, next action, gaps, and trace IDs",
            "assessment queue entries preserve recommendation lanes, resolution, recency vocabulary, and supersession links",
            "assessment work entries expose queue lanes, missing reviews, current reviewer actions, resolution labels, and trace IDs",
            "witness surface entries preserve support kind and support counts",
            "provenance root entries preserve explicit root identities, root kinds, coverage counts, assessment counts, and gap counts",
            "gap entries preserve conservative gap kinds, severity, action, and trace IDs",
            "source navigation entries preserve ingested source record identities, source kinds, and navigation targets",
        ]
    }
}
