//! `elicit_ui` IR invariant checks.

use std::collections::BTreeMap;

use tracing::instrument;

use super::{BelvedereInvariantCheck, count_value, unique_string_count};

#[instrument(
    level = "trace",
    skip(
        section_widget_counts,
        section_trace_counts,
        trace_source_counts,
        source_record_ids
    )
)]
pub(crate) fn ui_ir_trace_invariant_checks(
    trace_widget_count: usize,
    trace_ref_count: usize,
    section_widget_counts: &BTreeMap<String, usize>,
    section_trace_counts: &BTreeMap<String, usize>,
    trace_source_counts: &BTreeMap<String, usize>,
    source_record_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let unassigned_trace_widgets = count_value(
        section_trace_counts,
        "ui_ir_section_unassigned_trace_widgets",
    );
    let untraceable_sections = section_widget_counts
        .iter()
        .filter(|(section_id, widget_count)| {
            let source_widgets = section_trace_counts
                .get(&format!("ui_ir_section_trace_widgets:{section_id}"))
                .copied()
                .unwrap_or_default();
            (**widget_count > 0 && source_widgets == 0)
                || (**widget_count == 0 && source_widgets > 0)
        })
        .map(|(section_id, _)| section_id.as_str())
        .collect::<Vec<_>>();
    let section_source_links_total = count_value(
        section_trace_counts,
        "ui_ir_section_trace_source_links_total",
    );
    let widgets_with_sources = count_value(trace_source_counts, "trace_widgets_with_sources");
    let source_links = count_value(trace_source_counts, "trace_source_links");
    let unique_source_records = count_value(trace_source_counts, "trace_unique_source_records");
    let input_source_records = unique_string_count(source_record_ids);
    vec![
        BelvedereInvariantCheck::new(
            "ui_ir_trace_widgets_sourced".to_owned(),
            trace_widget_count > 0 && widgets_with_sources == trace_widget_count,
            format!(
                "{widgets_with_sources} of {trace_widget_count} trace widgets carry source links"
            ),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_trace_refs_preserved".to_owned(),
            source_links == trace_ref_count,
            format!("{source_links} source links recorded for {trace_ref_count} trace references"),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_trace_widgets_sectioned".to_owned(),
            unassigned_trace_widgets == 0 && section_source_links_total == trace_ref_count,
            format!(
                "{unassigned_trace_widgets} unassigned trace widgets; {section_source_links_total} section source links recorded for {trace_ref_count} trace references"
            ),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_sections_traceable".to_owned(),
            trace_widget_count > 0 && untraceable_sections.is_empty(),
            format!("sections with mismatched source traces: {untraceable_sections:?}"),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_input_sources_reachable".to_owned(),
            input_source_records > 0 && unique_source_records == input_source_records,
            format!(
                "{unique_source_records} of {input_source_records} input source records reachable from UI IR traces"
            ),
        ),
    ]
}

#[instrument(level = "trace", skip(source_locator_counts))]
pub(crate) fn ui_ir_source_locator_invariant_checks(
    source_locator_counts: &BTreeMap<String, usize>,
) -> Vec<BelvedereInvariantCheck> {
    let navigation_items = count_value(source_locator_counts, "ui_ir_source_navigation_items");
    let located_items = count_value(source_locator_counts, "ui_ir_source_locator_items");
    let located_widgets = count_value(source_locator_counts, "ui_ir_source_locator_widgets");
    vec![BelvedereInvariantCheck::new(
        "ui_ir_source_locators_preserved".to_owned(),
        navigation_items > 0
            && located_items == navigation_items
            && located_widgets == navigation_items,
        format!(
            "{located_widgets} locator widgets emitted for {located_items} located source navigation items"
        ),
    )]
}

#[instrument(level = "trace", skip(counts, output_kind_counts, output_ids))]
pub(crate) fn ui_ir_output_invariant_checks(
    counts: &BTreeMap<String, usize>,
    output_kind_counts: &BTreeMap<String, usize>,
    output_ids: &[String],
) -> Vec<BelvedereInvariantCheck> {
    let output_kind_total = output_kind_counts.values().sum::<usize>();
    let minted_standards = count_value(output_kind_counts, "ui_ir_output_kind_minted_standard");
    let sections = count_value(output_kind_counts, "ui_ir_output_kind_section");
    let viewports = count_value(output_kind_counts, "ui_ir_output_kind_viewport");
    let section_roots = count_value(output_kind_counts, "ui_ir_output_kind_section_root_widget");
    let trace_widgets = count_value(output_kind_counts, "ui_ir_output_kind_trace_widget");
    let trace_refs = count_value(output_kind_counts, "ui_ir_output_kind_trace_widget_ref");
    vec![
        BelvedereInvariantCheck::new(
            "ui_ir_output_ids_classified".to_owned(),
            output_kind_total == output_ids.len(),
            format!(
                "{output_kind_total} classified UI IR output IDs recorded for {} outputs",
                output_ids.len()
            ),
        ),
        BelvedereInvariantCheck::new(
            "ui_ir_core_counts_match_outputs".to_owned(),
            minted_standards == count_value(counts, "minted_standards")
                && sections == count_value(counts, "sections")
                && viewports == count_value(counts, "ui_ir_viewports_verified_total")
                && section_roots == count_value(counts, "sections")
                && trace_widgets == count_value(counts, "widgets")
                && trace_refs == count_value(counts, "widgets"),
            format!(
                "minted={minted_standards}, sections={sections}, viewports={viewports}, section_roots={section_roots}, trace_widgets={trace_widgets}, trace_refs={trace_refs}"
            ),
        ),
    ]
}
