//! Receipts for landscape diff exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;

/// Audit receipt emitted by a landscape diff exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct LandscapeDiffReceipt {
    contract: String,
    #[getter(skip)]
    counts: LandscapeDiffReceiptCounts,
    subject_kind_counts: BTreeMap<String, usize>,
    change_subject_counts: BTreeMap<String, usize>,
    trace_source_kind_counts: BTreeMap<String, usize>,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
}

impl LandscapeDiffReceipt {
    /// Modeled diff counts.
    #[must_use]
    pub const fn counts(&self) -> LandscapeDiffReceiptCounts {
        self.counts
    }
}

/// Counts emitted by a landscape diff receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandscapeDiffReceiptCounts {
    added_items: usize,
    removed_items: usize,
    changed_items: usize,
}

impl LandscapeDiffReceiptCounts {
    /// Build landscape diff counts.
    #[must_use]
    pub const fn new(added_items: usize, removed_items: usize, changed_items: usize) -> Self {
        Self {
            added_items,
            removed_items,
            changed_items,
        }
    }

    /// Number of added landscape items.
    #[must_use]
    pub const fn added_items(self) -> usize {
        self.added_items
    }

    /// Number of removed landscape items.
    #[must_use]
    pub const fn removed_items(self) -> usize {
        self.removed_items
    }

    /// Number of changed landscape items.
    #[must_use]
    pub const fn changed_items(self) -> usize {
        self.changed_items
    }

    /// Total number of diff items.
    #[must_use]
    pub const fn total_items(self) -> usize {
        self.added_items + self.removed_items + self.changed_items
    }
}
