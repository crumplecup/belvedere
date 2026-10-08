//! Receipt construction for `elicit_ui` projection exchanges.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    certify::{UiIrReceipt, UiIrReceiptBuilder, UiIrReceiptCounts},
    contracts::{
        AuditorNavigationModel, BelvedereContract, ElicitUiProjectionContract, TraceWidgetRef,
        UiIrSection, UiStandardMint, UiViewportVerification,
    },
    exchanges::UiIrError,
};

#[instrument(
    level = "debug",
    skip(
        model,
        sections,
        trace_widgets,
        tree,
        viewport_verifications,
        minted_standards
    )
)]
pub(super) fn build_receipt(
    model: &AuditorNavigationModel,
    sections: &[UiIrSection],
    trace_widgets: &[TraceWidgetRef],
    tree: &elicit_ui::VerifiedTree,
    viewport_verifications: &[UiViewportVerification],
    minted_standards: &[UiStandardMint],
) -> Result<UiIrReceipt, UiIrError> {
    UiIrReceiptBuilder::default()
        .contract(ElicitUiProjectionContract::NAME)
        .counts(UiIrReceiptCounts::new(
            sections.len(),
            trace_widgets.len(),
            trace_widgets
                .iter()
                .map(|widget| widget.source_record_ids().len())
                .sum(),
            tree.nodes().len(),
        ))
        .source_record_ids(model.source_record_ids().to_vec())
        .section_ids(
            sections
                .iter()
                .map(|section| section.id().to_owned())
                .collect::<Vec<_>>(),
        )
        .section_root_widget_ids(
            sections
                .iter()
                .map(UiIrSection::root_widget_id)
                .collect::<Vec<_>>(),
        )
        .trace_widget_ids(
            trace_widgets
                .iter()
                .map(TraceWidgetRef::widget_id)
                .collect::<Vec<_>>(),
        )
        .trace_widget_ref_ids(
            trace_widgets
                .iter()
                .map(|widget| widget.trace_ref_id().to_owned())
                .collect::<Vec<_>>(),
        )
        .role_counts(role_counts(trace_widgets))
        .section_widget_counts(section_widget_counts(sections))
        .section_population_counts(section_population_counts(sections))
        .section_trace_counts(section_trace_counts(sections, trace_widgets))
        .viewport_ids(viewport_ids(viewport_verifications))
        .viewport_verification_counts(viewport_verification_counts(viewport_verifications))
        .trace_source_counts(trace_source_counts(trace_widgets))
        .trace_source_kind_counts(trace_source_kind_counts(trace_widgets))
        .minted_prop_counts(minted_prop_counts(minted_standards))
        .proof_token_counts(proof_token_counts(viewport_verifications))
        .minted_standards(
            minted_standards
                .iter()
                .map(|standard| standard.standard().to_owned())
                .collect::<Vec<_>>(),
        )
        .build()
        .map_err(|error| UiIrError::construction("UiIrReceipt", error))
}

#[instrument(level = "debug", skip(viewport_verifications))]
fn proof_token_counts(
    viewport_verifications: &[UiViewportVerification],
) -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("ui_ir_proof_token_ir_sourced".to_owned(), 1),
        ("ui_ir_proof_token_wcag_verified".to_owned(), 1),
        (
            "ui_ir_proof_token_viewport_wcag_verified".to_owned(),
            viewport_verifications.len(),
        ),
    ])
}

#[instrument(level = "debug", skip(minted_standards))]
fn minted_prop_counts(minted_standards: &[UiStandardMint]) -> BTreeMap<String, usize> {
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
fn role_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for widget in trace_widgets {
        *counts.entry(format!("role_{}", widget.role())).or_default() += 1;
    }
    counts
}

#[instrument(level = "debug", skip(sections))]
fn section_widget_counts(sections: &[UiIrSection]) -> BTreeMap<String, usize> {
    sections
        .iter()
        .map(|section| (section.id().to_owned(), section.widget_count()))
        .collect()
}

#[instrument(level = "debug", skip(sections))]
fn section_population_counts(sections: &[UiIrSection]) -> BTreeMap<String, usize> {
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
fn section_trace_counts(
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
fn viewport_ids(viewport_verifications: &[UiViewportVerification]) -> Vec<String> {
    viewport_verifications
        .iter()
        .map(|verification| verification.viewport_id())
        .collect()
}

#[instrument(level = "debug", skip(viewport_verifications))]
fn viewport_verification_counts(
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
fn trace_source_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
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
fn trace_source_kind_counts(trace_widgets: &[TraceWidgetRef]) -> BTreeMap<String, usize> {
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
