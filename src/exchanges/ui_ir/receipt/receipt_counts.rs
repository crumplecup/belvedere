//! Count summaries for UI IR receipts.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::contracts::{
    AuditorNavigationModel, TraceWidgetRef, UI_IR_SECTION_SOURCE_NAVIGATION, UiIrSection,
    UiStandardMint, UiViewportVerification,
};

#[instrument(level = "debug", skip(minted_standards))]
pub(super) fn minted_prop_counts(minted_standards: &[UiStandardMint]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for mint in minted_standards {
        let prop = mint.prop().as_str();
        let prop_name = prop.strip_prefix("elicit_ui::").unwrap_or(prop);
        *counts
            .entry(format!("minted_elicit_ui_prop_{prop_name}"))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(trace_widgets))]
pub(super) fn role_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for widget in trace_widgets {
        *counts.entry(format!("role_{}", widget.role())).or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(sections))]
pub(super) fn section_widget_counts(sections: &[UiIrSection]) -> BTreeMap<String, usize> {
    sections
        .iter()
        .map(|section| (section.id().to_owned(), section.widget_count()))
        .collect()
}

#[instrument(level = "debug", skip(sections))]
pub(super) fn section_population_counts(sections: &[UiIrSection]) -> BTreeMap<String, usize> {
    let populated_sections = sections
        .iter()
        .filter(|section| section.widget_count() > 0)
        .count();
    BTreeMap::from([
        (
            "ui_ir_sections_empty".to_owned(),
            sections.len() - populated_sections,
        ),
        ("ui_ir_sections_populated".to_owned(), populated_sections),
        (
            "ui_ir_section_widgets_total".to_owned(),
            sections.iter().map(UiIrSection::widget_count).sum(),
        ),
    ])
}

#[instrument(level = "debug", skip(sections, trace_widgets))]
pub(super) fn section_trace_counts(
    sections: &[UiIrSection],
    trace_widgets: &[TraceWidgetRef],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let known_section_ids = sections
        .iter()
        .map(|section| section.id().to_owned())
        .collect::<BTreeSet<_>>();
    let mut source_widgets_by_section = BTreeMap::new();
    let mut source_links_by_section = BTreeMap::new();

    for section in sections {
        source_widgets_by_section.insert(section.id().to_owned(), 0);
        source_links_by_section.insert(section.id().to_owned(), 0);
    }

    let mut unassigned_trace_widgets = 0;
    for widget in trace_widgets {
        if !known_section_ids.contains(widget.section_id().as_str()) {
            unassigned_trace_widgets += 1;
            continue;
        }
        if !widget.source_record_ids().is_empty() {
            *source_widgets_by_section
                .entry(widget.section_id().to_owned())
                .or_default() += 1;
        }
        *source_links_by_section
            .entry(widget.section_id().to_owned())
            .or_default() += widget.source_record_ids().len();
    }

    let sections_with_source_widgets = source_widgets_by_section
        .values()
        .filter(|source_widgets| **source_widgets > 0)
        .count();
    let source_links_total = source_links_by_section.values().sum();

    for section in sections {
        counts.insert(
            format!("ui_ir_section_trace_widgets:{}", section.id()),
            *source_widgets_by_section.get(section.id()).unwrap_or(&0),
        );
        counts.insert(
            format!("ui_ir_section_trace_source_links:{}", section.id()),
            *source_links_by_section.get(section.id()).unwrap_or(&0),
        );
    }

    counts.insert(
        "ui_ir_sections_with_source_widgets".to_owned(),
        sections_with_source_widgets,
    );
    counts.insert(
        "ui_ir_sections_without_source_widgets".to_owned(),
        sections.len() - sections_with_source_widgets,
    );
    counts.insert(
        "ui_ir_section_trace_source_links_total".to_owned(),
        source_links_total,
    );
    counts.insert(
        "ui_ir_section_unassigned_trace_widgets".to_owned(),
        unassigned_trace_widgets,
    );
    counts
}

#[instrument(level = "debug", skip(viewport_verifications))]
pub(super) fn viewport_ids(viewport_verifications: &[UiViewportVerification]) -> Vec<String> {
    viewport_verifications
        .iter()
        .map(|verification| verification.viewport_id())
        .collect()
}

#[instrument(level = "debug", skip(viewport_verifications))]
pub(super) fn viewport_verification_counts(
    viewport_verifications: &[UiViewportVerification],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::from([(
        "ui_ir_viewports_verified_total".to_owned(),
        viewport_verifications.len(),
    )]);
    for verification in viewport_verifications {
        counts.insert(
            format!("ui_ir_viewport_nodes:{}", verification.viewport_id()),
            verification.node_count(),
        );
    }
    counts
}

#[instrument(level = "debug", skip(trace_widgets))]
pub(super) fn trace_source_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
    let mut unique_source_records = BTreeSet::new();
    let mut widgets_with_sources = 0;
    let mut source_links = 0;
    for widget in trace_widgets {
        if !widget.source_record_ids().is_empty() {
            widgets_with_sources += 1;
        }
        source_links += widget.source_record_ids().len();
        unique_source_records.extend(widget.source_record_ids().iter().cloned());
    }
    BTreeMap::from([
        (
            "trace_widgets_with_sources".to_owned(),
            widgets_with_sources,
        ),
        (
            "trace_unique_source_records".to_owned(),
            unique_source_records.len(),
        ),
        ("trace_source_links".to_owned(), source_links),
    ])
}

#[instrument(level = "debug", skip(trace_widgets))]
pub(super) fn trace_source_kind_counts(
    trace_widgets: &[TraceWidgetRef],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for source_record_id in trace_widgets
        .iter()
        .flat_map(|widget| widget.source_record_ids())
    {
        let kind = source_record_id
            .split_once(':')
            .map_or(source_record_id.as_str(), |(kind, _identity)| kind);
        *counts
            .entry(format!("trace_source_kind_{kind}"))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(model, trace_widgets))]
pub(super) fn source_locator_counts(
    model: &AuditorNavigationModel,
    trace_widgets: &[TraceWidgetRef],
) -> BTreeMap<String, usize> {
    let source_navigation_items = model.source_navigation().len();
    let located_items = model
        .source_navigation()
        .iter()
        .filter(|item| !item.source_locator().is_empty())
        .count();
    let located_source_record_ids = model
        .source_navigation()
        .iter()
        .filter(|item| !item.source_locator().is_empty())
        .map(|item| item.source_record_id().to_owned())
        .collect::<BTreeSet<_>>();
    let located_widgets = trace_widgets
        .iter()
        .filter(|widget| {
            widget.section_id() == UI_IR_SECTION_SOURCE_NAVIGATION
                && widget
                    .source_record_ids()
                    .iter()
                    .any(|source_record_id| located_source_record_ids.contains(source_record_id))
        })
        .count();
    BTreeMap::from([
        (
            "ui_ir_source_navigation_items".to_owned(),
            source_navigation_items,
        ),
        ("ui_ir_source_locator_items".to_owned(), located_items),
        ("ui_ir_source_locator_widgets".to_owned(), located_widgets),
    ])
}
