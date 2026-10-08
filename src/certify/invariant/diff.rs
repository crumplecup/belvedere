//! Landscape diff invariant checks.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use super::{BelvedereInvariantCheck, count_value};

#[instrument(
    level = "trace",
    skip(trace_source_counts, source_record_ids, trace_source_record_ids)
)]
pub(crate) fn diff_trace_invariant_checks(
    diff_item_count: usize,
    trace_source_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
    trace_source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let traceable_items = count_value(trace_source_counts, "diff_traceable_items");
    let items_with_sources = count_value(trace_source_counts, "diff_trace_items_with_sources");
    let source_links = count_value(trace_source_counts, "diff_trace_source_links");
    let unique_source_records =
        count_value(trace_source_counts, "diff_trace_unique_source_records");
    let input_source_records = source_record_ids.iter().collect::<BTreeSet<_>>();
    let trace_sources_are_admissible = trace_source_record_ids
        .iter()
        .all(|source_record_id| input_source_records.contains(source_record_id));
    vec![
        BelvedereInvariantCheck::new(
            "landscape_diff_items_sourced".to_owned(),
            diff_item_count > 0
                && traceable_items == diff_item_count
                && items_with_sources == diff_item_count,
            format!("{items_with_sources} of {diff_item_count} diff items carry source links"),
        ),
        BelvedereInvariantCheck::new(
            "landscape_diff_trace_links_preserved".to_owned(),
            source_links >= items_with_sources,
            format!(
                "{source_links} source links recorded across {items_with_sources} sourced diff items"
            ),
        ),
        BelvedereInvariantCheck::new(
            "landscape_diff_trace_sources_admissible".to_owned(),
            trace_sources_are_admissible && unique_source_records == trace_source_record_ids.len(),
            format!(
                "{} unique diff trace sources checked against {} compared snapshot source records",
                trace_source_record_ids.len(),
                input_source_records.len()
            ),
        ),
    ]
}

#[instrument(level = "trace", skip(counts, item_kind_counts, diff_item_ids))]
pub(crate) fn diff_item_invariant_checks(
    counts: &BTreeMap<String, usize>,
    item_kind_counts: &BTreeMap<String, usize>,
    diff_item_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let item_kind_total = item_kind_counts.values().sum::<usize>();
    let added_items = count_value(item_kind_counts, "diff_item_kind_added");
    let changed_items = count_value(item_kind_counts, "diff_item_kind_changed");
    let removed_items = count_value(item_kind_counts, "diff_item_kind_removed");
    vec![
        BelvedereInvariantCheck::new(
            "landscape_diff_item_ids_classified".to_owned(),
            item_kind_total == diff_item_ids.len(),
            format!(
                "{item_kind_total} classified diff item IDs recorded for {} outputs",
                diff_item_ids.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "landscape_diff_core_counts_match_items".to_owned(),
            added_items == count_value(counts, "added_items")
                && changed_items == count_value(counts, "changed_items")
                && removed_items == count_value(counts, "removed_items"),
            format!("added={added_items}, changed={changed_items}, removed={removed_items}"),
        ),
    ]
}
