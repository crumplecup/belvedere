//! Receipts for landscape diff review-model exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{BelvedereInvariantCheck, hex_digest, update_field, update_fields};

const LANDSCAPE_DIFF_REVIEW_RECEIPT_SCHEMA_ID: &str = "belvedere.receipts.landscape_diff_review.v2";
const LANDSCAPE_DIFF_REVIEW_RECEIPT_ID_PREFIX: &str = "belvedere.receipt.landscape_diff_review";

/// Audit receipt emitted by a landscape diff review-model exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct LandscapeDiffReviewReceipt {
    contract: String,
    #[getter(skip)]
    counts: LandscapeDiffReviewReceiptCounts,
    receipt_id: String,
    receipt_digest: String,
    change_kind_counts: BTreeMap<String, usize>,
    subject_kind_counts: BTreeMap<String, usize>,
    review_priority_counts: BTreeMap<String, usize>,
    review_action_counts: BTreeMap<String, usize>,
    trace_source_counts: BTreeMap<String, usize>,
    trace_source_record_ids: Vec<String>,
    source_diff_receipt_id: String,
    source_diff_receipt_digest: String,
    review_item_ids: Vec<String>,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
    #[builder(default)]
    transition_checks: Vec<BelvedereInvariantCheck>,
}

crate::certify::transition::impl_attested_receipt!(LandscapeDiffReviewReceipt);

impl LandscapeDiffReviewReceipt {
    /// Modeled diff review counts.
    #[must_use]
    pub const fn counts(&self) -> LandscapeDiffReviewReceiptCounts {
        self.counts
    }
}

/// Evidence used to identify a landscape diff review receipt.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(vis = "pub(crate)")]
pub(crate) struct LandscapeDiffReviewReceiptDigestEvidence<'evidence> {
    /// Exchange contract that emitted the receipt.
    contract: &'evidence str,
    /// Counts for emitted landscape diff review evidence.
    counts: LandscapeDiffReviewReceiptCounts,
    /// Counts grouped by diff change kind.
    change_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by changed subject kind.
    subject_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by auditor review priority.
    review_priority_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by auditor review action.
    review_action_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by trace source coverage.
    trace_source_counts: &'evidence BTreeMap<String, usize>,
    /// Source record IDs reached through review item traces.
    trace_source_record_ids: &'evidence [String],
    /// Source landscape diff receipt ID.
    source_diff_receipt_id: &'evidence str,
    /// Source landscape diff receipt digest.
    source_diff_receipt_digest: &'evidence str,
    /// Stable review item IDs emitted by the exchange.
    review_item_ids: &'evidence [String],
    /// Source record IDs represented by the source diff.
    source_record_ids: &'evidence [String],
    /// Recoverable warnings emitted by the exchange.
    warnings: &'evidence [String],
}

/// Build the stable identifier and digest for a landscape diff review receipt.
#[instrument(level = "debug", skip(evidence))]
pub(crate) fn landscape_diff_review_receipt_identity(
    evidence: LandscapeDiffReviewReceiptDigestEvidence<'_>,
) -> (String, String) {
    let receipt_digest = landscape_diff_review_receipt_digest(evidence);
    let receipt_id = format!("{LANDSCAPE_DIFF_REVIEW_RECEIPT_ID_PREFIX}:{receipt_digest}");
    (receipt_id, receipt_digest)
}

/// Counts emitted by a landscape diff review receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LandscapeDiffReviewReceiptCounts {
    total_items: usize,
    blocking_items: usize,
    review_items: usize,
    informational_items: usize,
}

impl LandscapeDiffReviewReceiptCounts {
    /// Build landscape diff review counts.
    #[must_use]
    pub const fn new(
        total_items: usize,
        blocking_items: usize,
        review_items: usize,
        informational_items: usize,
    ) -> Self {
        Self {
            total_items,
            blocking_items,
            review_items,
            informational_items,
        }
    }

    /// Number of reviewable diff items.
    #[must_use]
    pub const fn total_items(self) -> usize {
        self.total_items
    }

    /// Number of blocking review items.
    #[must_use]
    pub const fn blocking_items(self) -> usize {
        self.blocking_items
    }

    /// Number of focused review items.
    #[must_use]
    pub const fn review_items(self) -> usize {
        self.review_items
    }

    /// Number of informational review items.
    #[must_use]
    pub const fn informational_items(self) -> usize {
        self.informational_items
    }
}

#[instrument(level = "debug", skip(evidence))]
fn landscape_diff_review_receipt_digest(
    evidence: LandscapeDiffReviewReceiptDigestEvidence<'_>,
) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, LANDSCAPE_DIFF_REVIEW_RECEIPT_SCHEMA_ID);
    update_field(&mut hasher, evidence.contract);
    update_field(&mut hasher, &evidence.counts.total_items().to_string());
    update_field(&mut hasher, &evidence.counts.blocking_items().to_string());
    update_field(&mut hasher, &evidence.counts.review_items().to_string());
    update_field(
        &mut hasher,
        &evidence.counts.informational_items().to_string(),
    );
    update_counts(&mut hasher, evidence.change_kind_counts);
    update_counts(&mut hasher, evidence.subject_kind_counts);
    update_counts(&mut hasher, evidence.review_priority_counts);
    update_counts(&mut hasher, evidence.review_action_counts);
    update_counts(&mut hasher, evidence.trace_source_counts);
    update_fields(&mut hasher, evidence.trace_source_record_ids);
    update_field(&mut hasher, evidence.source_diff_receipt_id);
    update_field(&mut hasher, evidence.source_diff_receipt_digest);
    update_fields(&mut hasher, evidence.review_item_ids);
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
