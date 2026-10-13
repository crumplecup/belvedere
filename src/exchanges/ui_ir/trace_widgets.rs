//! Trace-widget evidence construction for `elicit_ui` projections.

use elicit_ui::{
    AccessKitUiBackend, ContrastDescriptor, LabelDescriptor, SrgbColor, WcagContrastFactory,
    WcagLabelFactory, WidgetId,
};
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::{
    certify::{hex_digest, update_field, update_fields},
    contracts::{TraceRef, TraceWidgetRef, TraceWidgetRefBuilder},
};

use super::{UiIrError, accessibility::establish_interactive_proofs};

const TRACE_WIDGET_SCHEMA_ID: &str = "belvedere.ui_ir.trace_widget.v1";

#[instrument(level = "debug", skip(backend, item_id, trace, trace_widgets))]
pub(in crate::exchanges::ui_ir) fn labeled_widget(
    backend: &AccessKitUiBackend,
    section_id: &'static str,
    role: &str,
    label: String,
    item_id: impl AsRef<str>,
    trace: TraceRef,
    trace_widgets: &mut Vec<TraceWidgetRef>,
) -> Result<WidgetId, UiIrError> {
    let (element, _proof) = backend
        .build_labeled_element(LabelDescriptor {
            name: label.clone(),
            role: role.to_owned(),
            labelled_by: None,
        })
        .map_err(UiIrError::elicit_ui)?;
    if matches!(
        role,
        "button" | "link" | "checkbox" | "radio" | "text-input" | "search-input"
    ) {
        establish_interactive_proofs(backend, element.id)?;
    }
    backend
        .build_contrast_minimum(ContrastDescriptor {
            foreground: SrgbColor::from_u8(0x11, 0x18, 0x27),
            background: SrgbColor::from_u8(0xf8, 0xfa, 0xfc),
            widget: Some(element.id),
        })
        .map_err(UiIrError::elicit_ui)?;
    let claim_id = trace.claim_id().to_owned();
    let item_id = item_id.as_ref().to_owned();
    let trace_ref_id = trace_widget_ref_id(section_id, role, &item_id, trace_widgets);
    let source_record_ids = trace.source_record_ids().to_vec();
    let trace_widget_digest = trace_widget_digest(TraceWidgetDigestEvidence {
        trace_ref_id: &trace_ref_id,
        item_id: &item_id,
        widget_id: element.id.0,
        section_id,
        role,
        label: &label,
        claim_id: &claim_id,
        source_record_ids: &source_record_ids,
    });
    let trace_widget = TraceWidgetRefBuilder::default()
        .trace_ref_id(trace_ref_id)
        .trace_widget_digest(trace_widget_digest)
        .item_id(item_id)
        .widget_id(element.id.0)
        .section_id(section_id)
        .role(role)
        .label(label)
        .claim_id(claim_id)
        .source_record_ids(source_record_ids)
        .build()
        .map_err(|error| UiIrError::construction("TraceWidgetRef", error))?;
    trace_widgets.push(trace_widget);
    Ok(element.id)
}

struct TraceWidgetDigestEvidence<'evidence> {
    trace_ref_id: &'evidence str,
    item_id: &'evidence str,
    widget_id: u64,
    section_id: &'evidence str,
    role: &'evidence str,
    label: &'evidence str,
    claim_id: &'evidence str,
    source_record_ids: &'evidence [String],
}

#[instrument(level = "trace", skip(evidence))]
fn trace_widget_digest(evidence: TraceWidgetDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, TRACE_WIDGET_SCHEMA_ID);
    update_field(&mut hasher, evidence.trace_ref_id);
    update_field(&mut hasher, evidence.item_id);
    update_field(&mut hasher, &evidence.widget_id.to_string());
    update_field(&mut hasher, evidence.section_id);
    update_field(&mut hasher, evidence.role);
    update_field(&mut hasher, evidence.label);
    update_field(&mut hasher, evidence.claim_id);
    update_fields(&mut hasher, evidence.source_record_ids);
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(trace_widgets))]
fn trace_widget_ref_id(
    section_id: &str,
    role: &str,
    item_id: &str,
    trace_widgets: &[TraceWidgetRef],
) -> String {
    let ordinal = trace_widgets
        .iter()
        .filter(|widget| {
            widget.section_id() == section_id
                && widget.role() == role
                && widget.item_id() == item_id
        })
        .count();
    let base = format!("elicit_ui.trace_widget_ref:{section_id}:{role}:{item_id}");
    if ordinal == 0 {
        base
    } else {
        format!("{base}:{ordinal}")
    }
}
