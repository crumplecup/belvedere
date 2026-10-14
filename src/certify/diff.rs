//! Receipts for landscape diff exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{BelvedereInvariantCheck, hex_digest, update_field, update_fields};

const LANDSCAPE_DIFF_RECEIPT_SCHEMA_ID: &str = "belvedere.receipts.landscape_diff.v2";
const LANDSCAPE_DIFF_RECEIPT_ID_PREFIX: &str = "belvedere.receipt.landscape_diff";

/// Audit receipt emitted by a landscape diff exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct LandscapeDiffReceipt {
    contract: String,
    #[getter(skip)]
    counts: LandscapeDiffReceiptCounts,
    receipt_id: String,
    receipt_digest: String,
    item_kind_counts: BTreeMap<String, usize>,
    subject_kind_counts: BTreeMap<String, usize>,
    change_subject_counts: BTreeMap<String, usize>,
    review_priority_counts: BTreeMap<String, usize>,
    review_action_counts: BTreeMap<String, usize>,
    trace_source_counts: BTreeMap<String, usize>,
    trace_source_kind_counts: BTreeMap<String, usize>,
    trace_source_record_ids: Vec<String>,
    diff_item_ids: Vec<String>,
    source_landscape_receipt_ids: Vec<String>,
    source_landscape_receipt_digests: Vec<String>,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
    #[builder(default)]
    transition_checks: Vec<BelvedereInvariantCheck>,
}

crate::certify::transition::impl_attested_receipt!(LandscapeDiffReceipt);

impl LandscapeDiffReceipt {
    /// Modeled diff counts.
    #[must_use]
    pub const fn counts(&self) -> LandscapeDiffReceiptCounts {
        self.counts
    }
}

/// Evidence used to identify a landscape diff receipt.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(vis = "pub(crate)")]
pub(crate) struct LandscapeDiffReceiptDigestEvidence<'evidence> {
    /// Exchange contract that emitted the receipt.
    contract: &'evidence str,
    /// Counts for emitted landscape diff evidence.
    counts: LandscapeDiffReceiptCounts,
    /// Counts grouped by diff item kind.
    item_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by changed subject kind.
    subject_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by combined change and subject kind.
    change_subject_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by auditor review priority.
    review_priority_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by auditor review action.
    review_action_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by trace source coverage.
    trace_source_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by trace source record kind.
    trace_source_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Source record IDs reached through diff traces.
    trace_source_record_ids: &'evidence [String],
    /// Stable diff item IDs emitted by the exchange.
    diff_item_ids: &'evidence [String],
    /// Source landscape receipt IDs compared by the exchange.
    source_landscape_receipt_ids: &'evidence [String],
    /// Source landscape receipt digests compared by the exchange.
    source_landscape_receipt_digests: &'evidence [String],
    /// Source record IDs represented by the diff input snapshots.
    source_record_ids: &'evidence [String],
    /// Recoverable warnings emitted by the exchange.
    warnings: &'evidence [String],
}

/// Build the stable identifier and digest for a landscape diff receipt.
#[instrument(level = "debug", skip(evidence))]
pub(crate) fn landscape_diff_receipt_identity(
    evidence: LandscapeDiffReceiptDigestEvidence<'_>,
) -> (String, String) {
    let receipt_digest = landscape_diff_receipt_digest(evidence);
    let receipt_id = format!("{LANDSCAPE_DIFF_RECEIPT_ID_PREFIX}:{receipt_digest}");
    (receipt_id, receipt_digest)
}

/// Counts emitted by a landscape diff receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[instrument(level = "debug", skip(evidence))]
fn landscape_diff_receipt_digest(evidence: LandscapeDiffReceiptDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, LANDSCAPE_DIFF_RECEIPT_SCHEMA_ID);
    update_field(&mut hasher, evidence.contract);
    update_field(&mut hasher, &evidence.counts.added_items().to_string());
    update_field(&mut hasher, &evidence.counts.removed_items().to_string());
    update_field(&mut hasher, &evidence.counts.changed_items().to_string());
    update_counts(&mut hasher, evidence.item_kind_counts);
    update_counts(&mut hasher, evidence.subject_kind_counts);
    update_counts(&mut hasher, evidence.change_subject_counts);
    update_counts(&mut hasher, evidence.review_priority_counts);
    update_counts(&mut hasher, evidence.review_action_counts);
    update_counts(&mut hasher, evidence.trace_source_counts);
    update_counts(&mut hasher, evidence.trace_source_kind_counts);
    update_fields(&mut hasher, evidence.trace_source_record_ids);
    update_fields(&mut hasher, evidence.diff_item_ids);
    update_fields(&mut hasher, evidence.source_landscape_receipt_ids);
    update_fields(&mut hasher, evidence.source_landscape_receipt_digests);
    update_fields(&mut hasher, evidence.source_record_ids);
    update_fields(&mut hasher, evidence.warnings);
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(hasher, counts))]
fn update_counts(hasher: &mut Sha256, counts: &BTreeMap<String, usize>) {
    for (name, count) in counts {
        update_field(hasher, name);
        update_field(hasher, &count.to_string());
    }
}
