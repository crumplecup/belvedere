//! Landscape snapshot diff exchange implementation.

mod items;
mod receipt;
mod traces;

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    certify::{LandscapeDiffReceipt, LandscapeDiffReceiptBuilder},
    contracts::{
        BelvedereContract, LandscapeAssessment, LandscapeDiff, LandscapeDiffChangeKind,
        LandscapeDiffContract, LandscapeDiffInput, LandscapeDiffItem, LandscapeGap,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};
use items::{
    claim_change_summary, claim_summary, push_assessment_item, push_claim_item, push_gap_item,
};
use receipt::{change_subject_counts, diff_counts, subject_kind_counts};
use traces::{
    changed_assessment_source_record_ids, changed_claim_source_record_ids,
    changed_gap_source_record_ids, subject_source_record_ids_for_assessment,
    subject_source_record_ids_for_claim, subject_source_record_ids_for_claim_id,
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
        let mut items = Vec::new();
        diff_claims(&input, &source_record_ids, &mut items);
        diff_assessments(&input, &source_record_ids, &mut items);
        diff_gaps(&input, &source_record_ids, &mut items);
        items.sort_by(|left, right| left.item_id().cmp(right.item_id()));

        let counts = diff_counts(&items);
        let diff = LandscapeDiff::new(items, source_record_ids.clone());
        let receipt = LandscapeDiffReceiptBuilder::default()
            .contract(LandscapeDiffContract::NAME)
            .counts(counts)
            .subject_kind_counts(subject_kind_counts(diff.items()))
            .change_subject_counts(change_subject_counts(diff.items()))
            .source_record_ids(source_record_ids)
            .warnings(Vec::new())
            .build()
            .map_err(|error| BelvedereConstructionError::new("LandscapeDiffReceipt", error))?;
        Ok((diff, receipt))
    }
}

#[instrument(level = "debug", skip(input))]
fn merged_source_record_ids(input: &LandscapeDiffInput) -> Vec<String> {
    let ids: BTreeSet<String> = input
        .previous()
        .source_record_ids()
        .iter()
        .chain(input.current().source_record_ids())
        .cloned()
        .collect();
    ids.into_iter().collect()
}

#[instrument(level = "debug", skip(input, source_record_ids, items))]
fn diff_claims(
    input: &LandscapeDiffInput,
    source_record_ids: &[String],
    items: &mut Vec<LandscapeDiffItem>,
) {
    let previous = input.previous().claims();
    let current = input.current().claims();
    for claim_id in keyed_union(previous, current) {
        match (previous.get(claim_id), current.get(claim_id)) {
            (None, Some(claim)) => push_claim_item(
                items,
                LandscapeDiffChangeKind::Added,
                claim.id(),
                claim_summary("added", claim),
                subject_source_record_ids_for_claim(claim, source_record_ids),
            ),
            (Some(claim), None) => push_claim_item(
                items,
                LandscapeDiffChangeKind::Removed,
                claim.id(),
                claim_summary("removed", claim),
                subject_source_record_ids_for_claim(claim, source_record_ids),
            ),
            (Some(previous_claim), Some(current_claim)) if previous_claim != current_claim => {
                push_claim_item(
                    items,
                    LandscapeDiffChangeKind::Changed,
                    current_claim.id(),
                    claim_change_summary(previous_claim, current_claim),
                    changed_claim_source_record_ids(
                        previous_claim,
                        current_claim,
                        source_record_ids,
                    ),
                );
            }
            (Some(_), Some(_)) | (None, None) => {}
        }
    }
}

#[instrument(level = "debug", skip(input, source_record_ids, items))]
fn diff_assessments(
    input: &LandscapeDiffInput,
    source_record_ids: &[String],
    items: &mut Vec<LandscapeDiffItem>,
) {
    let previous = assessments_by_id(input.previous().assessments());
    let current = assessments_by_id(input.current().assessments());
    for assessment_id in keyed_union(&previous, &current) {
        match (previous.get(assessment_id), current.get(assessment_id)) {
            (None, Some(assessment)) => push_assessment_item(
                items,
                LandscapeDiffChangeKind::Added,
                assessment,
                "added assessment",
                subject_source_record_ids_for_assessment(assessment, source_record_ids),
            ),
            (Some(assessment), None) => push_assessment_item(
                items,
                LandscapeDiffChangeKind::Removed,
                assessment,
                "removed assessment",
                subject_source_record_ids_for_assessment(assessment, source_record_ids),
            ),
            (Some(previous_assessment), Some(current_assessment))
                if previous_assessment != current_assessment =>
            {
                push_assessment_item(
                    items,
                    LandscapeDiffChangeKind::Changed,
                    current_assessment,
                    "changed assessment",
                    changed_assessment_source_record_ids(
                        previous_assessment,
                        current_assessment,
                        source_record_ids,
                    ),
                );
            }
            (Some(_), Some(_)) | (None, None) => {}
        }
    }
}

#[instrument(level = "debug", skip(input, source_record_ids, items))]
fn diff_gaps(
    input: &LandscapeDiffInput,
    source_record_ids: &[String],
    items: &mut Vec<LandscapeDiffItem>,
) {
    let previous = gaps_by_key(input.previous().gaps());
    let current = gaps_by_key(input.current().gaps());
    for gap_key in keyed_union(&previous, &current) {
        match (previous.get(gap_key), current.get(gap_key)) {
            (None, Some(gap)) => push_gap_item(
                items,
                LandscapeDiffChangeKind::Added,
                gap_key,
                gap,
                "added gap",
                subject_source_record_ids_for_claim_id(gap.claim_id(), source_record_ids),
            ),
            (Some(gap), None) => push_gap_item(
                items,
                LandscapeDiffChangeKind::Removed,
                gap_key,
                gap,
                "removed gap",
                subject_source_record_ids_for_claim_id(gap.claim_id(), source_record_ids),
            ),
            (Some(previous_gap), Some(current_gap)) if previous_gap != current_gap => {
                push_gap_item(
                    items,
                    LandscapeDiffChangeKind::Changed,
                    gap_key,
                    current_gap,
                    "changed gap",
                    changed_gap_source_record_ids(previous_gap, current_gap, source_record_ids),
                );
            }
            (Some(_), Some(_)) | (None, None) => {}
        }
    }
}

#[instrument(level = "trace", skip(previous, current))]
fn keyed_union<'a, V>(
    previous: &'a BTreeMap<String, V>,
    current: &'a BTreeMap<String, V>,
) -> Vec<&'a str> {
    previous
        .keys()
        .chain(current.keys())
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[instrument(level = "debug", skip(assessments))]
fn assessments_by_id(
    assessments: &[LandscapeAssessment],
) -> BTreeMap<String, &LandscapeAssessment> {
    assessments
        .iter()
        .map(|assessment| (assessment.id().to_owned(), assessment))
        .collect()
}

#[instrument(level = "debug", skip(gaps))]
fn gaps_by_key(gaps: &[LandscapeGap]) -> BTreeMap<String, &LandscapeGap> {
    gaps.iter()
        .map(|gap| (format!("{}:{:?}", gap.claim_id(), gap.kind()), gap))
        .collect()
}
