//! Landscape diff review-model exchange implementation.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    certify::{
        LandscapeDiffReviewReceipt, LandscapeDiffReviewReceiptBuilder,
        LandscapeDiffReviewReceiptCounts, LandscapeDiffReviewReceiptDigestEvidenceBuilder,
        landscape_diff_review_receipt_identity,
    },
    contracts::{
        BelvedereContract, LandscapeDiffChangeKind, LandscapeDiffReviewAction,
        LandscapeDiffReviewContract, LandscapeDiffReviewInput, LandscapeDiffReviewItem,
        LandscapeDiffReviewModel, LandscapeDiffReviewPriority, LandscapeDiffReviewSummary,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};

/// Exchange from a landscape diff into UI-neutral auditor review evidence.
#[derive(Debug, Clone, Copy, Default)]
pub struct LandscapeDiffReviewProjection;

impl LandscapeDiffReviewProjection {
    /// Project a landscape diff into auditor review evidence.
    ///
    /// # Errors
    ///
    /// Returns an error if crate-internal evidence builders reject the projected
    /// review model or receipt.
    #[instrument(level = "debug", skip(self, input))]
    pub fn project(
        &self,
        input: LandscapeDiffReviewInput,
    ) -> Result<(LandscapeDiffReviewModel, LandscapeDiffReviewReceipt), BelvedereError> {
        <Self as BelvedereExchange<LandscapeDiffReviewContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<LandscapeDiffReviewContract> for LandscapeDiffReviewProjection {
    type Error = BelvedereError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: LandscapeDiffReviewInput,
    ) -> Result<(LandscapeDiffReviewModel, LandscapeDiffReviewReceipt), Self::Error> {
        let items = input
            .diff()
            .items()
            .iter()
            .map(|item| {
                LandscapeDiffReviewItem::from_diff_item(review_item_id(item.item_id()), item)
            })
            .collect::<Vec<_>>();
        let summary = review_summary(&items);
        let source_record_ids = input.diff().source_record_ids().to_vec();
        let model = LandscapeDiffReviewModel::new(summary, items, source_record_ids.clone());
        let receipt_counts = LandscapeDiffReviewReceiptCounts::new(
            model.items().len(),
            summary.blocking_items(),
            summary.review_items(),
            summary.informational_items(),
        );
        let receipt_change_kind_counts = change_kind_counts(model.items());
        let receipt_subject_kind_counts = subject_kind_counts(model.items());
        let receipt_review_priority_counts = review_priority_counts(model.items());
        let receipt_review_action_counts = review_action_counts(model.items());
        let receipt_trace_source_counts = trace_source_counts(model.items());
        let receipt_trace_source_record_ids = trace_source_record_ids(model.items());
        let receipt_review_item_ids = review_item_ids(model.items());
        let receipt_warnings = Vec::new();
        let digest_evidence = LandscapeDiffReviewReceiptDigestEvidenceBuilder::default()
            .contract(LandscapeDiffReviewContract::NAME)
            .counts(receipt_counts)
            .change_kind_counts(&receipt_change_kind_counts)
            .subject_kind_counts(&receipt_subject_kind_counts)
            .review_priority_counts(&receipt_review_priority_counts)
            .review_action_counts(&receipt_review_action_counts)
            .trace_source_counts(&receipt_trace_source_counts)
            .trace_source_record_ids(&receipt_trace_source_record_ids)
            .source_diff_receipt_id(input.source_diff_receipt_id())
            .source_diff_receipt_digest(input.source_diff_receipt_digest())
            .review_item_ids(&receipt_review_item_ids)
            .source_record_ids(&source_record_ids)
            .warnings(&receipt_warnings)
            .build()
            .map_err(|error| {
                BelvedereConstructionError::new("LandscapeDiffReviewReceiptDigestEvidence", error)
            })?;
        let (receipt_id, receipt_digest) = landscape_diff_review_receipt_identity(digest_evidence);
        let receipt = LandscapeDiffReviewReceiptBuilder::default()
            .contract(LandscapeDiffReviewContract::NAME)
            .counts(receipt_counts)
            .receipt_id(receipt_id)
            .receipt_digest(receipt_digest)
            .change_kind_counts(receipt_change_kind_counts)
            .subject_kind_counts(receipt_subject_kind_counts)
            .review_priority_counts(receipt_review_priority_counts)
            .review_action_counts(receipt_review_action_counts)
            .trace_source_counts(receipt_trace_source_counts)
            .trace_source_record_ids(receipt_trace_source_record_ids)
            .source_diff_receipt_id(input.source_diff_receipt_id().to_owned())
            .source_diff_receipt_digest(input.source_diff_receipt_digest().to_owned())
            .review_item_ids(receipt_review_item_ids)
            .source_record_ids(source_record_ids)
            .warnings(receipt_warnings)
            .build()
            .map_err(|error| {
                BelvedereConstructionError::new("LandscapeDiffReviewReceipt", error)
            })?;
        Ok((model, receipt))
    }
}

#[instrument(level = "debug", skip(items))]
fn review_summary(items: &[LandscapeDiffReviewItem]) -> LandscapeDiffReviewSummary {
    LandscapeDiffReviewSummary::new(
        items
            .iter()
            .filter(|item| item.change_kind() == LandscapeDiffChangeKind::Added)
            .count(),
        items
            .iter()
            .filter(|item| item.change_kind() == LandscapeDiffChangeKind::Removed)
            .count(),
        items
            .iter()
            .filter(|item| item.change_kind() == LandscapeDiffChangeKind::Changed)
            .count(),
        items
            .iter()
            .filter(|item| item.review_priority() == LandscapeDiffReviewPriority::Blocking)
            .count(),
        items
            .iter()
            .filter(|item| item.review_priority() == LandscapeDiffReviewPriority::Review)
            .count(),
        items
            .iter()
            .filter(|item| item.review_priority() == LandscapeDiffReviewPriority::Informational)
            .count(),
    )
}

#[instrument(level = "debug", skip(items))]
fn review_item_ids(items: &[LandscapeDiffReviewItem]) -> Vec<String> {
    items.iter().map(|item| item.item_id().to_owned()).collect()
}

#[instrument(level = "debug", skip(items))]
fn change_kind_counts(items: &[LandscapeDiffReviewItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts
            .entry(format!(
                "diff_review_change_kind_{}",
                change_kind_key(item.change_kind())
            ))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
fn subject_kind_counts(items: &[LandscapeDiffReviewItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts
            .entry(format!("diff_review_subject_{}", item.subject_kind()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
fn review_priority_counts(items: &[LandscapeDiffReviewItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts
            .entry(format!(
                "diff_review_priority_{}",
                review_priority_key(item.review_priority())
            ))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
fn review_action_counts(items: &[LandscapeDiffReviewItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts
            .entry(format!(
                "diff_review_action_{}",
                review_action_key(item.review_action())
            ))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
fn trace_source_counts(items: &[LandscapeDiffReviewItem]) -> BTreeMap<String, usize> {
    let mut source_records = BTreeSet::new();
    let mut items_with_sources = 0;
    let mut source_links = 0;
    for item in items {
        if !item.trace().source_record_ids().is_empty() {
            items_with_sources += 1;
        }
        source_links += item.trace().source_record_ids().len();
        source_records.extend(item.trace().source_record_ids().iter().cloned());
    }
    BTreeMap::from([
        ("diff_review_traceable_items".to_owned(), items.len()),
        (
            "diff_review_trace_items_with_sources".to_owned(),
            items_with_sources,
        ),
        ("diff_review_trace_source_links".to_owned(), source_links),
        (
            "diff_review_trace_unique_source_records".to_owned(),
            source_records.len(),
        ),
    ])
}

#[instrument(level = "debug", skip(items))]
fn trace_source_record_ids(items: &[LandscapeDiffReviewItem]) -> Vec<String> {
    items
        .iter()
        .flat_map(|item| item.trace().source_record_ids().iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[instrument(level = "trace", skip(diff_item_id))]
fn review_item_id(diff_item_id: &str) -> String {
    format!("belvedere.diff_review.item:{diff_item_id}")
}

const fn change_kind_key(change_kind: LandscapeDiffChangeKind) -> &'static str {
    match change_kind {
        LandscapeDiffChangeKind::Added => "added",
        LandscapeDiffChangeKind::Removed => "removed",
        LandscapeDiffChangeKind::Changed => "changed",
    }
}

const fn review_priority_key(review_priority: LandscapeDiffReviewPriority) -> &'static str {
    match review_priority {
        LandscapeDiffReviewPriority::Blocking => "blocking",
        LandscapeDiffReviewPriority::Review => "review",
        LandscapeDiffReviewPriority::Informational => "informational",
    }
}

const fn review_action_key(review_action: LandscapeDiffReviewAction) -> &'static str {
    match review_action {
        LandscapeDiffReviewAction::ReviewNewClaim => "review_new_claim",
        LandscapeDiffReviewAction::CompareClaimEvidence => "compare_claim_evidence",
        LandscapeDiffReviewAction::ConfirmClaimRemoval => "confirm_claim_removal",
        LandscapeDiffReviewAction::ReviewNewAssessment => "review_new_assessment",
        LandscapeDiffReviewAction::CompareAssessmentEvidence => "compare_assessment_evidence",
        LandscapeDiffReviewAction::ConfirmAssessmentRemoval => "confirm_assessment_removal",
        LandscapeDiffReviewAction::TriageNewGap => "triage_new_gap",
        LandscapeDiffReviewAction::ReassessChangedGap => "reassess_changed_gap",
        LandscapeDiffReviewAction::VerifyResolvedGap => "verify_resolved_gap",
    }
}
