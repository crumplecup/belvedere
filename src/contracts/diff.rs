//! Landscape snapshot diff contracts and evidence types.

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

use crate::{
    certify::{LandscapeDiffReceipt, LandscapeReceipt},
    contracts::{
        BelvedereContract, LANDSCAPE_CONSTRUCTION_STANDARD_ID, LandscapeGraph,
        REPORT_REPRODUCIBILITY_STANDARD_ID, TraceRef,
    },
};

/// Two landscape snapshots accepted by the landscape diff exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct LandscapeDiffInput {
    #[getter(skip)]
    previous: LandscapeGraph,
    #[getter(skip)]
    current: LandscapeGraph,
    source_landscape_receipt_ids: Vec<String>,
    source_landscape_receipt_digests: Vec<String>,
}

impl LandscapeDiffInput {
    /// Create a diff input from an earlier and later landscape snapshot.
    #[must_use]
    #[instrument(level = "debug", skip(previous, current))]
    pub fn new(previous: LandscapeGraph, current: LandscapeGraph) -> Self {
        Self {
            previous,
            current,
            source_landscape_receipt_ids: Vec::new(),
            source_landscape_receipt_digests: Vec::new(),
        }
    }

    /// Create a diff input from landscapes and their modeling receipts.
    #[must_use]
    #[instrument(
        level = "debug",
        skip(previous, previous_receipt, current, current_receipt)
    )]
    pub fn from_landscape_receipts(
        previous: LandscapeGraph,
        previous_receipt: &LandscapeReceipt,
        current: LandscapeGraph,
        current_receipt: &LandscapeReceipt,
    ) -> Self {
        Self {
            previous,
            current,
            source_landscape_receipt_ids: vec![
                previous_receipt.receipt_id().to_owned(),
                current_receipt.receipt_id().to_owned(),
            ],
            source_landscape_receipt_digests: vec![
                previous_receipt.receipt_digest().to_owned(),
                current_receipt.receipt_digest().to_owned(),
            ],
        }
    }

    /// Earlier landscape snapshot.
    #[must_use]
    pub const fn previous(&self) -> &LandscapeGraph {
        &self.previous
    }

    /// Later landscape snapshot.
    #[must_use]
    pub const fn current(&self) -> &LandscapeGraph {
        &self.current
    }
}

/// UI-neutral diff between two landscape snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct LandscapeDiff {
    items: Vec<LandscapeDiffItem>,
    source_record_ids: Vec<String>,
}

/// One changed landscape item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct LandscapeDiffItem {
    #[new(into)]
    item_id: String,
    #[getter(skip)]
    change_kind: LandscapeDiffChangeKind,
    #[new(into)]
    subject_kind: String,
    #[new(into)]
    subject_id: String,
    #[new(into)]
    summary: String,
    #[getter(skip)]
    trace: TraceRef,
}

impl LandscapeDiffItem {
    /// Kind of snapshot change.
    #[must_use]
    pub const fn change_kind(&self) -> LandscapeDiffChangeKind {
        self.change_kind
    }

    /// Source trace for the changed subject.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Snapshot change category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandscapeDiffChangeKind {
    /// Item exists only in the current snapshot.
    Added,
    /// Item exists only in the previous snapshot.
    Removed,
    /// Item exists in both snapshots but its modeled evidence changed.
    Changed,
}

/// Landscape snapshot diff contract.
pub struct LandscapeDiffContract;

impl BelvedereContract for LandscapeDiffContract {
    type Input = LandscapeDiffInput;
    type Output = LandscapeDiff;
    type Receipt = LandscapeDiffReceipt;

    const NAME: &'static str = "belvedere.diff.landscape";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "both landscape snapshots were produced by the landscape modeling exchange",
            "source record identities remain available for both snapshots",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "claim additions, removals, and modeled evidence changes are surfaced deterministically",
            "assessment additions, removals, and reviewer evidence changes are surfaced deterministically",
            "gap additions, removals, and detail changes are surfaced deterministically",
            "each diff item preserves the source identities available from the compared snapshots",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[
            LANDSCAPE_CONSTRUCTION_STANDARD_ID,
            REPORT_REPRODUCIBILITY_STANDARD_ID,
        ]
    }
}
