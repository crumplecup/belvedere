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
                .label(format!("Source record {source_record_id}"))
                .trace(TraceRef::new(
                    format!("belvedere.source_record:{source_record_id}"),
                    vec![source_record_id.to_owned()],
                ))
                .build()
                .map_err(|error| BelvedereConstructionError::new("SourceNavigationItem", error))
        })
        .collect()
}
