//! Receipts for UI IR projection exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;

/// Audit receipt emitted by an `elicit_ui` projection exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct UiIrReceipt {
    contract: String,
    #[getter(skip)]
    counts: UiIrReceiptCounts,
    source_record_ids: Vec<String>,
    section_ids: Vec<String>,
    role_counts: BTreeMap<String, usize>,
    section_widget_counts: BTreeMap<String, usize>,
    trace_source_counts: BTreeMap<String, usize>,
    minted_standards: Vec<String>,
}

impl UiIrReceipt {
    /// UI IR projection counts.
    #[must_use]
    pub const fn counts(&self) -> UiIrReceiptCounts {
        self.counts
    }
}

/// Counts emitted by a UI IR projection receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiIrReceiptCounts {
    section_count: usize,
    widget_count: usize,
    trace_ref_count: usize,
    node_count: usize,
}

impl UiIrReceiptCounts {
    /// Build UI IR projection counts.
    #[must_use]
    pub const fn new(
        section_count: usize,
        widget_count: usize,
        trace_ref_count: usize,
        node_count: usize,
    ) -> Self {
        Self {
            section_count,
            widget_count,
            trace_ref_count,
            node_count,
        }
    }

    /// Number of top-level auditor navigation sections.
    #[must_use]
    pub const fn section_count(self) -> usize {
        self.section_count
    }

    /// Number of traceable widgets emitted by the projection.
    #[must_use]
    pub const fn widget_count(self) -> usize {
        self.widget_count
    }

    /// Number of source trace references preserved.
    #[must_use]
    pub const fn trace_ref_count(self) -> usize {
        self.trace_ref_count
    }

    /// Number of nodes in the emitted `elicit_ui` tree.
    #[must_use]
    pub const fn node_count(self) -> usize {
        self.node_count
    }
}
