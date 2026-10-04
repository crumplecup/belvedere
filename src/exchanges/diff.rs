//! Landscape snapshot diff exchange implementation.

mod changes;
mod items;
mod receipt;
mod traces;

use tracing::instrument;

use crate::{
    certify::{LandscapeDiffReceipt, LandscapeDiffReceiptBuilder},
    contracts::{BelvedereContract, LandscapeDiff, LandscapeDiffContract, LandscapeDiffInput},
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};
use changes::{diff_items, merged_source_record_ids};
use receipt::{change_subject_counts, diff_counts, subject_kind_counts, trace_source_kind_counts};

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
        let receipt = LandscapeDiffReceiptBuilder::default()
            .contract(LandscapeDiffContract::NAME)
            .counts(counts)
            .subject_kind_counts(subject_kind_counts(diff.items()))
            .change_subject_counts(change_subject_counts(diff.items()))
            .trace_source_kind_counts(trace_source_kind_counts(diff.items()))
            .source_record_ids(source_record_ids)
            .warnings(Vec::new())
            .build()
            .map_err(|error| BelvedereConstructionError::new("LandscapeDiffReceipt", error))?;
        Ok((diff, receipt))
    }
}
