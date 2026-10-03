//! Source record navigation projections.

use derive_new::new;
use tracing::instrument;

use crate::{
    contracts::{SourceNavigationItem, SourceNavigationItemBuilder, TraceRef},
    error::BelvedereConstructionError,
};

#[instrument(level = "debug", skip(source_record_ids))]
pub(super) fn project_source_navigation(
    source_record_ids: &[String],
) -> Result<Vec<SourceNavigationItem>, BelvedereConstructionError> {
    source_record_ids
        .iter()
        .map(|source_record_id| {
            let source = source_record(source_record_id);
            SourceNavigationItemBuilder::default()
                .source_record_id(source_record_id)
                .source_kind(source.kind)
                .target(source.target)
                .label(source.label)
                .trace(TraceRef::new(
                    format!("belvedere.source_record:{source_record_id}"),
                    vec![source_record_id.to_owned()],
                ))
                .build()
                .map_err(|error| BelvedereConstructionError::new("SourceNavigationItem", error))
        })
        .collect()
}

#[instrument(level = "trace", skip(source_record_id))]
fn source_record(source_record_id: &str) -> SourceRecordNavigation {
    if let Some(rest) = source_record_id.strip_prefix("evidence_link:") {
        SourceRecordNavigation::new("evidence_link", rest, format!("Evidence link {rest}"))
    } else if let Some(rest) = source_record_id.strip_prefix("proof_record:") {
        SourceRecordNavigation::new("proof_record", rest, format!("Proof record {rest}"))
    } else if let Some(rest) = source_record_id.strip_prefix("contract_record:") {
        SourceRecordNavigation::new("contract_record", rest, format!("Contract record {rest}"))
    } else if let Some(rest) = source_record_id.strip_prefix("witness_export:") {
        SourceRecordNavigation::new("witness_export", rest, format!("Witness export {rest}"))
    } else if let Some(rest) = source_record_id.strip_prefix("kani_proof:") {
        SourceRecordNavigation::new("kani_proof", rest, format!("Kani proof {rest}"))
    } else if let Some(rest) = source_record_id.strip_prefix("assessment:") {
        SourceRecordNavigation::new("assessment", rest, format!("Assessment {rest}"))
    } else {
        SourceRecordNavigation::new(
            "source_record",
            source_record_id,
            format!("Source record {source_record_id}"),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
struct SourceRecordNavigation {
    #[new(into)]
    kind: String,
    #[new(into)]
    target: String,
    #[new(into)]
    label: String,
}
