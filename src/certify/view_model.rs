//! Receipts for auditor navigation view-model exchanges.

use derive_builder::Builder;
use derive_getters::Getters;

/// Audit receipt emitted by a view-model exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ViewModelReceipt {
    contract: String,
    #[getter(skip)]
    counts: ViewModelReceiptCounts,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
}

impl ViewModelReceipt {
    /// Modeled view counts.
    #[must_use]
    pub const fn counts(&self) -> ViewModelReceiptCounts {
        self.counts
    }
}

/// Counts emitted by an auditor navigation receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewModelReceiptCounts {
    claim_cards: usize,
    filter_facets: usize,
    proof_chain_items: usize,
    assessment_items: usize,
    witness_items: usize,
    root_items: usize,
    gap_items: usize,
}

impl ViewModelReceiptCounts {
    /// Build view-model counts.
    #[must_use]
    pub const fn new(
        claim_cards: usize,
        filter_facets: usize,
        proof_chain_items: usize,
        assessment_items: usize,
        witness_items: usize,
        root_items: usize,
        gap_items: usize,
    ) -> Self {
        Self {
            claim_cards,
            filter_facets,
            proof_chain_items,
            assessment_items,
            witness_items,
            root_items,
            gap_items,
        }
    }

    /// Number of claim explorer cards.
    #[must_use]
    pub const fn claim_cards(self) -> usize {
        self.claim_cards
    }

    /// Number of auditor filter facets.
    #[must_use]
    pub const fn filter_facets(self) -> usize {
        self.filter_facets
    }

    /// Number of proof-chain navigation items.
    #[must_use]
    pub const fn proof_chain_items(self) -> usize {
        self.proof_chain_items
    }

    /// Number of assessment queue items.
    #[must_use]
    pub const fn assessment_items(self) -> usize {
        self.assessment_items
    }

    /// Number of witness surface items.
    #[must_use]
    pub const fn witness_items(self) -> usize {
        self.witness_items
    }

    /// Number of provenance root items.
    #[must_use]
    pub const fn root_items(self) -> usize {
        self.root_items
    }

    /// Number of gap items.
    #[must_use]
    pub const fn gap_items(self) -> usize {
        self.gap_items
    }
}
