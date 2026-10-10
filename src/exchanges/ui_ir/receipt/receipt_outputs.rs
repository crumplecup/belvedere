//! Output identity helpers for UI IR receipts.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::contracts::{TraceWidgetRef, UiIrSection, UiStandardMint, UiViewportVerification};

#[instrument(
    level = "debug",
    skip(sections, trace_widgets, viewport_verifications, minted_standards)
)]
pub(super) fn output_ids(
    sections: &[UiIrSection],
    trace_widgets: &[TraceWidgetRef],
    viewport_verifications: &[UiViewportVerification],
    minted_standards: &[UiStandardMint],
) -> Vec<String> {
    let mut ids = minted_standards
        .iter()
        .map(|standard| standard.standard().to_owned())
        .collect::<Vec<_>>();
    ids.extend(sections.iter().map(|section| section.id().to_owned()));
    ids.extend(
        viewport_verifications
            .iter()
            .map(|viewport| format!("elicit_ui.viewport:{}", viewport.viewport_id())),
    );
    ids.extend(
        sections
            .iter()
            .map(|section| format!("elicit_ui.section_root_widget:{}", section.root_widget_id())),
    );
    ids.extend(
        trace_widgets
            .iter()
            .map(|widget| format!("elicit_ui.trace_widget:{}", widget.widget_id())),
    );
    ids.extend(
        trace_widgets
            .iter()
            .map(|widget| widget.trace_ref_id().to_owned()),
    );
    ids
}

#[instrument(level = "debug", skip(output_ids))]
pub(super) fn output_kind_counts(output_ids: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for output_id in output_ids {
        let kind = ui_ir_output_kind(output_id);
        *counts
            .entry(format!("ui_ir_output_kind_{kind}"))
            .or_default() += 1;
    }
    counts
}

#[instrument(level = "trace", skip(output_id))]
fn ui_ir_output_kind(output_id: &str) -> &'static str {
    if output_id.starts_with("belvedere.ui.section.") {
        "section"
    } else if output_id.starts_with("elicit_ui.viewport:") {
        "viewport"
    } else if output_id.starts_with("elicit_ui.section_root_widget:") {
        "section_root_widget"
    } else if output_id.starts_with("elicit_ui.trace_widget:") {
        "trace_widget"
    } else if output_id.starts_with("elicit_ui.trace_widget_ref:") {
        "trace_widget_ref"
    } else {
        "minted_standard"
    }
}
