//! Source record navigation projections.

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
            SourceNavigationItemBuilder::default()
                .source_record_id(source_record_id)
                .label(source_record_label(source_record_id))
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
fn source_record_label(source_record_id: &str) -> String {
    if let Some(rest) = source_record_id.strip_prefix("evidence_link:") {
        format!("Evidence link {rest}")
    } else if let Some(rest) = source_record_id.strip_prefix("proof_record:") {
        format!("Proof record {rest}")
    } else if let Some(rest) = source_record_id.strip_prefix("contract_record:") {
        format!("Contract record {rest}")
    } else if let Some(rest) = source_record_id.strip_prefix("witness_export:") {
        format!("Witness export {rest}")
    } else if let Some(rest) = source_record_id.strip_prefix("kani_proof:") {
        format!("Kani proof {rest}")
    } else if let Some(rest) = source_record_id.strip_prefix("assessment:") {
        format!("Assessment {rest}")
    } else {
        format!("Source record {source_record_id}")
    }
}
