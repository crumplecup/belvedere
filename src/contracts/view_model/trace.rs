//! Source trace references retained by visible view-model items.

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;

/// Source trace retained by every visible view-model item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct TraceRef {
    #[new(into)]
    claim_id: String,
    source_record_ids: Vec<String>,
}
