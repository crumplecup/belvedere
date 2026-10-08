//! Receipt count aggregation for landscape diffs.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    certify::LandscapeDiffReceiptCounts,
    contracts::{LandscapeDiffChangeKind, LandscapeDiffItem},
};

#[instrument(level = "debug", skip(items))]
pub(super) fn diff_counts(items: &[LandscapeDiffItem]) -> LandscapeDiffReceiptCounts {
    let added_items = items
        .iter()
        .filter(|item| item.change_kind() == LandscapeDiffChangeKind::Added)
        .count();
    let removed_items = items
        .iter()
        .filter(|item| item.change_kind() == LandscapeDiffChangeKind::Removed)
        .count();
    let changed_items = items
        .iter()
        .filter(|item| item.change_kind() == LandscapeDiffChangeKind::Changed)
        .count();
    LandscapeDiffReceiptCounts::new(added_items, removed_items, changed_items)
}

#[instrument(level = "debug", skip(items))]
pub(super) fn diff_item_ids(items: &[LandscapeDiffItem]) -> Vec<String> {
    items.iter().map(|item| item.item_id().to_owned()).collect()
}

#[instrument(level = "debug", skip(items))]
pub(super) fn subject_kind_counts(items: &[LandscapeDiffItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts
            .entry(format!("diff_subject_{}", item.subject_kind()))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
pub(super) fn change_subject_counts(items: &[LandscapeDiffItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts
            .entry(format!(
                "diff_change_{}_{}",
                change_kind_key(item.change_kind()),
                item.subject_kind()
            ))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
pub(super) fn trace_source_kind_counts(items: &[LandscapeDiffItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for source_record_id in items
        .iter()
        .flat_map(|item| item.trace().source_record_ids())
    {
        let kind = source_record_id
            .split_once(':')
            .map_or(source_record_id.as_str(), |(kind, _identity)| kind);
        *counts
            .entry(format!("diff_trace_source_kind_{kind}"))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(items))]
pub(super) fn trace_source_counts(items: &[LandscapeDiffItem]) -> BTreeMap<String, usize> {
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
        ("diff_traceable_items".to_owned(), items.len()),
        (
            "diff_trace_items_with_sources".to_owned(),
            items_with_sources,
        ),
        ("diff_trace_source_links".to_owned(), source_links),
        (
            "diff_trace_unique_source_records".to_owned(),
            source_records.len(),
        ),
    ])
}

#[instrument(level = "debug", skip(items))]
pub(super) fn trace_source_record_ids(items: &[LandscapeDiffItem]) -> Vec<String> {
    items
        .iter()
        .flat_map(|item| item.trace().source_record_ids().iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

const fn change_kind_key(change_kind: LandscapeDiffChangeKind) -> &'static str {
    match change_kind {
        LandscapeDiffChangeKind::Added => "added",
        LandscapeDiffChangeKind::Removed => "removed",
        LandscapeDiffChangeKind::Changed => "changed",
    }
}
