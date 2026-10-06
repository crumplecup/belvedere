//! Unsupported-field evidence shared by ingestion decoders.

use std::collections::BTreeMap;

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

/// A field preserved by ingestion but not yet interpreted by Belvedere.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new, Serialize)]
pub struct UnsupportedField {
    #[new(into)]
    record: String,
    #[new(into)]
    field: String,
}

#[instrument(level = "debug", skip(record, extra))]
pub(super) fn unsupported(
    record: impl Into<String>,
    extra: &BTreeMap<String, serde_json::Value>,
) -> Vec<UnsupportedField> {
    let record = record.into();
    extra
        .keys()
        .map(|field| UnsupportedField::new(record.clone(), field.clone()))
        .collect()
}
