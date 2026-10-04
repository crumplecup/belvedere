//! Receipts for Belvedere landscape modeling exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;

/// Audit receipt emitted by a landscape modeling exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct LandscapeReceipt {
    contract: String,
    #[getter(skip)]
    counts: LandscapeReceiptCounts,
    gap_kind_counts: BTreeMap<String, usize>,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
}

impl LandscapeReceipt {
    /// Number of claims in the landscape.
    #[must_use]
    pub const fn claim_count(&self) -> usize {
        self.counts.claim_count()
    }

    /// Number of evidence edges in the landscape.
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.counts.edge_count()
    }

    /// Number of explicit root claims.
    #[must_use]
    pub const fn root_count(&self) -> usize {
        self.counts.root_count()
    }

    /// Number of proof summaries in the landscape.
    #[must_use]
    pub const fn proof_count(&self) -> usize {
        self.counts.proof_count()
    }

    /// Number of assessment events in the landscape.
    #[must_use]
    pub const fn assessment_count(&self) -> usize {
        self.counts.assessment_count()
    }

    /// Number of conservative review gaps.
    #[must_use]
    pub const fn gap_count(&self) -> usize {
        self.counts.gap_count()
    }

    /// All modeled landscape counts.
    #[must_use]
    pub const fn counts(&self) -> LandscapeReceiptCounts {
        self.counts
    }
}

/// Counts emitted by a landscape modeling receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandscapeReceiptCounts {
    claim_count: usize,
    edge_count: usize,
    root_count: usize,
    proof_count: usize,
    assessment_count: usize,
    gap_count: usize,
}

impl LandscapeReceiptCounts {
    /// Build modeled landscape counts.
    #[must_use]
    pub const fn new(
        claim_count: usize,
        edge_count: usize,
        root_count: usize,
        proof_count: usize,
        assessment_count: usize,
        gap_count: usize,
    ) -> Self {
        Self {
            claim_count,
            edge_count,
            root_count,
            proof_count,
            assessment_count,
            gap_count,
        }
    }

    /// Number of claims in the landscape.
    #[must_use]
    pub const fn claim_count(self) -> usize {
        self.claim_count
    }

    /// Number of evidence edges in the landscape.
    #[must_use]
    pub const fn edge_count(self) -> usize {
        self.edge_count
    }

    /// Number of explicit root claims.
    #[must_use]
    pub const fn root_count(self) -> usize {
        self.root_count
    }

    /// Number of proof coverage entries in the landscape.
    #[must_use]
    pub const fn proof_count(self) -> usize {
        self.proof_count
    }

    /// Number of assessment events in the landscape.
    #[must_use]
    pub const fn assessment_count(self) -> usize {
        self.assessment_count
    }

    /// Number of conservative review gaps.
    #[must_use]
    pub const fn gap_count(self) -> usize {
        self.gap_count
    }
}
