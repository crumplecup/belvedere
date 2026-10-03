//! Claim explorer and proof-chain navigation items.

use derive_builder::Builder;
use derive_getters::Getters;
use tracing::instrument;

use crate::contracts::view_model::TraceRef;

/// Claim explorer row/card evidence.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct ClaimExplorerItem {
    claim_id: String,
    #[getter(skip)]
    root: bool,
    basis_ids: Vec<String>,
    verifiers: Vec<String>,
    filter_ids: Vec<String>,
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
        filter_ids: Vec<String>,
        counts: ClaimExplorerCounts,
        trace: TraceRef,
    ) -> Self {
        Self {
            claim_id: claim_id.into(),
            root,
            basis_ids,
            verifiers,
            filter_ids,
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
    review_state: String,
    next_action: String,
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
