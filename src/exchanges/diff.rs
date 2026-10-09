//! Landscape snapshot diff exchange implementation.

mod changes;
mod items;
mod receipt;
mod traces;

use tracing::instrument;

use crate::{
    certify::{
        LandscapeDiffReceipt, LandscapeDiffReceiptBuilder,
        LandscapeDiffReceiptDigestEvidenceBuilder, landscape_diff_receipt_identity,
    },
    contracts::{BelvedereContract, LandscapeDiff, LandscapeDiffContract, LandscapeDiffInput},
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};
use changes::{diff_items, merged_source_record_ids};
use receipt::{
    change_subject_counts, diff_counts, diff_item_ids, item_kind_counts, subject_kind_counts,
    trace_source_counts, trace_source_kind_counts, trace_source_record_ids,
};

/// Exchange from two landscape snapshots into deterministic diff evidence.
#[derive(Debug, Clone, Copy, Default)]
pub struct LandscapeDiffing;

impl LandscapeDiffing {
    /// Diff two landscape snapshots.
    ///
    /// # Errors
    ///
    /// Returns an error if crate-internal evidence builders reject the diff
    /// receipt. Snapshot differences themselves are represented as diff items
    /// rather than exchange failures.
    #[instrument(level = "debug", skip(self, input))]
    pub fn diff(
        &self,
        input: LandscapeDiffInput,
    ) -> Result<(LandscapeDiff, LandscapeDiffReceipt), BelvedereError> {
        <Self as BelvedereExchange<LandscapeDiffContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<LandscapeDiffContract> for LandscapeDiffing {
    type Error = BelvedereError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: LandscapeDiffInput,
    ) -> Result<(LandscapeDiff, LandscapeDiffReceipt), Self::Error> {
        let source_record_ids = merged_source_record_ids(&input);
        let items = diff_items(&input, &source_record_ids);

        let counts = diff_counts(&items);
        let diff = LandscapeDiff::new(items, source_record_ids.clone());
        let receipt_item_kind_counts = item_kind_counts(diff.items());
        let receipt_subject_kind_counts = subject_kind_counts(diff.items());
        let receipt_change_subject_counts = change_subject_counts(diff.items());
        let receipt_trace_source_counts = trace_source_counts(diff.items());
        let receipt_trace_source_kind_counts = trace_source_kind_counts(diff.items());
        let receipt_trace_source_record_ids = trace_source_record_ids(diff.items());
        let receipt_diff_item_ids = diff_item_ids(diff.items());
        let source_landscape_receipt_ids = input.source_landscape_receipt_ids().to_vec();
        let source_landscape_receipt_digests = input.source_landscape_receipt_digests().to_vec();
        let receipt_warnings = Vec::new();
        let identity_evidence = LandscapeDiffReceiptDigestEvidenceBuilder::default()
            .contract(LandscapeDiffContract::NAME)
            .counts(counts)
            .item_kind_counts(&receipt_item_kind_counts)
            .subject_kind_counts(&receipt_subject_kind_counts)
            .change_subject_counts(&receipt_change_subject_counts)
            .trace_source_counts(&receipt_trace_source_counts)
            .trace_source_kind_counts(&receipt_trace_source_kind_counts)
            .trace_source_record_ids(&receipt_trace_source_record_ids)
            .diff_item_ids(&receipt_diff_item_ids)
            .source_landscape_receipt_ids(&source_landscape_receipt_ids)
            .source_landscape_receipt_digests(&source_landscape_receipt_digests)
            .source_record_ids(&source_record_ids)
            .warnings(&receipt_warnings)
            .build()
            .map_err(|error| {
                BelvedereConstructionError::new("LandscapeDiffReceiptDigestEvidence", error)
            })?;
        let (receipt_id, receipt_digest) = landscape_diff_receipt_identity(identity_evidence);
        let receipt = LandscapeDiffReceiptBuilder::default()
            .contract(LandscapeDiffContract::NAME)
            .counts(counts)
            .receipt_id(receipt_id)
            .receipt_digest(receipt_digest)
            .item_kind_counts(receipt_item_kind_counts)
            .subject_kind_counts(receipt_subject_kind_counts)
            .change_subject_counts(receipt_change_subject_counts)
            .trace_source_counts(receipt_trace_source_counts)
            .trace_source_kind_counts(receipt_trace_source_kind_counts)
            .trace_source_record_ids(receipt_trace_source_record_ids)
            .diff_item_ids(receipt_diff_item_ids)
            .source_landscape_receipt_ids(source_landscape_receipt_ids)
            .source_landscape_receipt_digests(source_landscape_receipt_digests)
            .source_record_ids(source_record_ids)
            .warnings(receipt_warnings)
            .build()
            .map_err(|error| BelvedereConstructionError::new("LandscapeDiffReceipt", error))?;
        Ok((diff, receipt))
    }
}
