//! Landscape diff change detection.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    contracts::{
        LandscapeAssessment, LandscapeDiffChangeKind, LandscapeDiffInput, LandscapeDiffItem,
        LandscapeGap,
    },
    exchanges::{
        diff::{
            items::{
                claim_change_summary, claim_summary, push_assessment_item, push_claim_item,
                push_gap_item,
            },
            traces::{
                changed_assessment_source_record_ids, changed_claim_source_record_ids,
                changed_gap_source_record_ids,
            },
        },
        source_records::{
            subject_source_record_ids_for_assessment, subject_source_record_ids_for_claim,
            subject_source_record_ids_for_gap,
        },
    },
};

#[instrument(level = "debug", skip(input))]
pub(super) fn merged_source_record_ids(input: &LandscapeDiffInput) -> Vec<String> {
    let ids: BTreeSet<String> = input
        .previous()
        .source_record_ids()
        .iter()
        .chain(input.current().source_record_ids())
        .cloned()
        .collect();
    ids.into_iter().collect()
}

#[instrument(level = "debug", skip(input, source_record_ids))]
pub(super) fn diff_items(
    input: &LandscapeDiffInput,
    source_record_ids: &[String],
) -> Vec<LandscapeDiffItem> {
    let mut items = Vec::new();
    diff_claims(input, source_record_ids, &mut items);
    diff_assessments(input, source_record_ids, &mut items);
    diff_gaps(input, source_record_ids, &mut items);
    items.sort_by(|left, right| left.item_id().cmp(right.item_id()));
    items
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
                subject_source_record_ids_for_gap(gap, source_record_ids),
            ),
            (Some(gap), None) => push_gap_item(
                items,
                LandscapeDiffChangeKind::Removed,
                gap_key,
                gap,
                "removed gap",
                subject_source_record_ids_for_gap(gap, source_record_ids),
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
