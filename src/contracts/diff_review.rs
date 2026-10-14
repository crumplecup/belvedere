//! Auditor review view-model contracts for landscape diffs.

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

use crate::{
    certify::{
        BelvedereCertificationError, BelvedereStageCertificate, LandscapeDiffReceipt,
        LandscapeDiffReviewReceipt,
    },
    contracts::{
        BelvedereContract, LandscapeDiff, LandscapeDiffChangeKind, LandscapeDiffItem,
        LandscapeDiffReviewAction, LandscapeDiffReviewPriority, TraceRef,
        VIEW_MODEL_FIDELITY_STANDARD_ID, require_transition,
    },
};

/// Landscape diff evidence accepted by the diff review-model exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct LandscapeDiffReviewInput {
    #[getter(skip)]
    diff: LandscapeDiff,
    source_diff_receipt_id: String,
    source_diff_receipt_digest: String,
}

impl LandscapeDiffReviewInput {
    /// Create review-model input from a landscape diff and its receipt.
    #[must_use]
    #[instrument(level = "debug", skip(diff, receipt))]
    pub fn from_diff(diff: LandscapeDiff, receipt: &LandscapeDiffReceipt) -> Self {
        Self::new(
            diff,
            receipt.receipt_id().to_owned(),
            receipt.receipt_digest().to_owned(),
        )
    }

    /// Landscape diff evidence to project.
    #[must_use]
    pub const fn diff(&self) -> &LandscapeDiff {
        &self.diff
    }
}

/// UI-neutral auditor review model for a landscape diff.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct LandscapeDiffReviewModel {
    #[getter(skip)]
    summary: LandscapeDiffReviewSummary,
    items: Vec<LandscapeDiffReviewItem>,
    source_record_ids: Vec<String>,
}

impl LandscapeDiffReviewModel {
    /// Diff review totals and queue signals.
    #[must_use]
    pub const fn summary(&self) -> &LandscapeDiffReviewSummary {
        &self.summary
    }
}

/// Summary totals for a diff review model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, new)]
pub struct LandscapeDiffReviewSummary {
    added_items: usize,
    removed_items: usize,
    changed_items: usize,
    blocking_items: usize,
    review_items: usize,
    informational_items: usize,
}

impl LandscapeDiffReviewSummary {
    /// Number of added diff items.
    #[must_use]
    pub const fn added_items(self) -> usize {
        self.added_items
    }

    /// Number of removed diff items.
    #[must_use]
    pub const fn removed_items(self) -> usize {
        self.removed_items
    }

    /// Number of changed diff items.
    #[must_use]
    pub const fn changed_items(self) -> usize {
        self.changed_items
    }

    /// Number of review items that should block acceptance until inspected.
    #[must_use]
    pub const fn blocking_items(self) -> usize {
        self.blocking_items
    }

    /// Number of review items that deserve focused inspection.
    #[must_use]
    pub const fn review_items(self) -> usize {
        self.review_items
    }

    /// Number of review items that provide non-blocking context.
    #[must_use]
    pub const fn informational_items(self) -> usize {
        self.informational_items
    }

    /// Total number of diff review items.
    #[must_use]
    pub const fn total_items(self) -> usize {
        self.added_items + self.removed_items + self.changed_items
    }
}

/// One auditor-reviewable changed landscape item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct LandscapeDiffReviewItem {
    item_id: String,
    diff_item_id: String,
    #[getter(skip)]
    change_kind: LandscapeDiffChangeKind,
    #[getter(skip)]
    review_priority: LandscapeDiffReviewPriority,
    #[getter(skip)]
    review_action: LandscapeDiffReviewAction,
    subject_kind: String,
    subject_id: String,
    summary: String,
    #[getter(skip)]
    trace: TraceRef,
}

impl LandscapeDiffReviewItem {
    /// Create a review item from a source diff item while preserving its
    /// auditor guidance and provenance trace.
    #[must_use]
    #[instrument(level = "debug", skip(item_id, diff_item))]
    pub fn from_diff_item(item_id: impl Into<String>, diff_item: &LandscapeDiffItem) -> Self {
        Self {
            item_id: item_id.into(),
            diff_item_id: diff_item.item_id().to_owned(),
            change_kind: diff_item.change_kind(),
            review_priority: diff_item.review_priority(),
            review_action: diff_item.review_action(),
            subject_kind: diff_item.subject_kind().to_owned(),
            subject_id: diff_item.subject_id().to_owned(),
            summary: diff_item.summary().to_owned(),
            trace: diff_item.trace().clone(),
        }
    }

    /// Kind of snapshot change this review item represents.
    #[must_use]
    pub const fn change_kind(&self) -> LandscapeDiffChangeKind {
        self.change_kind
    }

    /// Auditor review priority for this item.
    #[must_use]
    pub const fn review_priority(&self) -> LandscapeDiffReviewPriority {
        self.review_priority
    }

    /// Recommended auditor action for this item.
    #[must_use]
    pub const fn review_action(&self) -> LandscapeDiffReviewAction {
        self.review_action
    }

    /// Source trace for the reviewed subject.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Landscape diff review-model contract.
pub struct LandscapeDiffReviewContract;

impl BelvedereContract for LandscapeDiffReviewContract {
    type Input = LandscapeDiffReviewInput;
    type Output = LandscapeDiffReviewModel;
    type Receipt = LandscapeDiffReviewReceipt;

    const NAME: &'static str = "belvedere.view_model.landscape_diff_review";
    const ASSERTION_NAMES: &'static [&'static str] = &[
        "review_items_preserve_diff",
        "review_sources_preserved",
        "review_receipt_input_binding",
        "review_receipt_output_binding",
    ];

    #[instrument(level = "trace", skip(input, output, receipt))]
    fn verify_transition(
        input: &Self::Input,
        output: &Self::Output,
        receipt: &Self::Receipt,
    ) -> Result<Vec<crate::certify::BelvedereInvariantCheck>, BelvedereCertificationError> {
        let items_preserved = input.diff().items().len() == output.items().len()
            && input
                .diff()
                .items()
                .iter()
                .zip(output.items())
                .all(|(source, item)| {
                    item.diff_item_id() == source.item_id()
                        && item.change_kind() == source.change_kind()
                        && item.review_priority() == source.review_priority()
                        && item.review_action() == source.review_action()
                        && item.subject_kind() == source.subject_kind()
                        && item.subject_id() == source.subject_id()
                        && item.summary() == source.summary()
                        && item.trace() == source.trace()
                });
        require_transition::<Self>([
            ("review_items_preserve_diff", items_preserved),
            (
                "review_sources_preserved",
                output.source_record_ids() == input.diff().source_record_ids(),
            ),
            (
                "review_receipt_input_binding",
                receipt.source_diff_receipt_id() == input.source_diff_receipt_id()
                    && receipt.source_diff_receipt_digest() == input.source_diff_receipt_digest(),
            ),
            (
                "review_receipt_output_binding",
                receipt.source_record_ids() == output.source_record_ids()
                    && receipt.counts().total_items() == output.items().len(),
            ),
        ])
    }

    #[instrument(level = "trace", skip(receipt))]
    fn verify_receipt(receipt: &Self::Receipt) -> Result<(), BelvedereCertificationError> {
        BelvedereStageCertificate::landscape_diff_review(receipt).map(|_| ())
    }

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "landscape diff was produced by the landscape diff exchange",
            "source diff receipt identity is available",
            "diff items carry deterministic review priorities, actions, and trace IDs",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "diff review summary totals are derived from diff item change kinds and priorities",
            "each review item retains the source diff item ID, subject identity, review priority, review action, and trace IDs",
            "review model receipts count priority queues, action categories, subject kinds, change kinds, and trace coverage",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[VIEW_MODEL_FIDELITY_STANDARD_ID]
    }
}
