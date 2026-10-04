//! Receipts for auditor navigation view-model exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;

/// Audit receipt emitted by a view-model exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ViewModelReceipt {
    contract: String,
    #[getter(skip)]
    counts: ViewModelReceiptCounts,
    lane_counts: BTreeMap<String, usize>,
    facet_group_counts: BTreeMap<String, usize>,
    source_artifact_counts: BTreeMap<String, usize>,
    source_kind_counts: BTreeMap<String, usize>,
    proof_chain_topology_counts: BTreeMap<String, usize>,
    provenance_root_kind_counts: BTreeMap<String, usize>,
    review_workflow_counts: BTreeMap<String, usize>,
    trace_kind_counts: BTreeMap<String, usize>,
    trace_source_counts: BTreeMap<String, usize>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ViewModelReceiptCounts {
    claim_cards: usize,
    filter_facets: usize,
    backend_coverage_items: usize,
    proof_chain_items: usize,
    review_workflow_items: usize,
    assessment_items: usize,
    assessment_work_items: usize,
    witness_items: usize,
    root_items: usize,
    gap_items: usize,
    source_items: usize,
}

impl ViewModelReceiptCounts {
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

    /// Number of backend coverage status entries.
    #[must_use]
    pub const fn backend_coverage_items(self) -> usize {
        self.backend_coverage_items
    }

    /// Number of proof-chain navigation items.
    #[must_use]
    pub const fn proof_chain_items(self) -> usize {
        self.proof_chain_items
    }

    /// Number of review workflow tasks.
    #[must_use]
    pub const fn review_workflow_items(self) -> usize {
        self.review_workflow_items
    }

    /// Number of assessment queue items.
    #[must_use]
    pub const fn assessment_items(self) -> usize {
        self.assessment_items
    }

    /// Number of assessment work items.
    #[must_use]
    pub const fn assessment_work_items(self) -> usize {
        self.assessment_work_items
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

    /// Number of source navigation items.
    #[must_use]
    pub const fn source_items(self) -> usize {
        self.source_items
    }
}
