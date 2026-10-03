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
    backend_coverage: Vec<ProofChainBackendCoverage>,
    #[getter(skip)]
    topology: ProofChainTopology,
    review_state: String,
    next_action: String,
    #[getter(skip)]
    gap_count: usize,
    #[getter(skip)]
    trace: TraceRef,
}

impl ProofChainItem {
    /// Proof-chain topology evidence.
    #[must_use]
    pub const fn topology(&self) -> &ProofChainTopology {
        &self.topology
    }

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

/// Per-backend completeness evidence for a transitive proof chain.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ProofChainBackendCoverage {
    verifier: String,
    #[getter(skip)]
    complete: bool,
    #[getter(skip)]
    covered_claim_count: usize,
    #[getter(skip)]
    missing_claim_count: usize,
}

impl ProofChainBackendCoverage {
    /// Whether every claim in the chain has proof coverage for this backend.
    #[must_use]
    pub const fn complete(&self) -> bool {
        self.complete
    }

    /// Number of claims in the chain covered by this backend.
    #[must_use]
    pub const fn covered_claim_count(&self) -> usize {
        self.covered_claim_count
    }

    /// Number of claims in the chain missing this backend.
    #[must_use]
    pub const fn missing_claim_count(&self) -> usize {
        self.missing_claim_count
    }
}

/// Topology summary for a claim's transitive proof chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProofChainTopology {
    transitive_claim_count: usize,
    max_depth: usize,
    branching_claim_count: usize,
    chain_gap_count: usize,
}

impl ProofChainTopology {
    /// Create proof-chain topology evidence.
    #[must_use]
    pub const fn new(
        transitive_claim_count: usize,
        max_depth: usize,
        branching_claim_count: usize,
        chain_gap_count: usize,
    ) -> Self {
        Self {
            transitive_claim_count,
            max_depth,
            branching_claim_count,
            chain_gap_count,
        }
    }

    /// Number of distinct claims reached by following bases, including the subject.
    #[must_use]
    pub const fn transitive_claim_count(self) -> usize {
        self.transitive_claim_count
    }

    /// Longest basis distance from subject to terminal root.
    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// Number of reached claims with more than one modeled direct basis.
    #[must_use]
    pub const fn branching_claim_count(self) -> usize {
        self.branching_claim_count
    }

    /// Number of gaps attached anywhere in the reached chain.
    #[must_use]
    pub const fn chain_gap_count(self) -> usize {
        self.chain_gap_count
    }
}
