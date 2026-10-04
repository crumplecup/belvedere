//! Receipts for Belvedere ingestion exchanges.

use std::{collections::BTreeMap, path::PathBuf};

use derive_getters::Getters;
use tracing::instrument;

use crate::contracts::{
    AcceptedAmenableArtifactFormat, ArtifactKind, RawAmenableArtifact, UnsupportedField,
};

/// Audit receipt emitted by an ingestion exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct IngestionReceipt {
    contract: String,
    path: PathBuf,
    #[getter(skip)]
    artifact_kind: ArtifactKind,
    #[getter(skip)]
    format: AcceptedAmenableArtifactFormat,
    digest: String,
    parser_version: String,
    decoded_counts: BTreeMap<String, usize>,
    source_record_kind_counts: BTreeMap<String, usize>,
    unsupported_fields: Vec<UnsupportedField>,
    warnings: Vec<String>,
    source_record_ids: Vec<String>,
}

impl IngestionReceipt {
    /// Build a receipt for a successful ingestion exchange.
    #[must_use]
    #[instrument(
        level = "debug",
        skip(artifact, format, decoded_counts, unsupported_fields)
    )]
    pub fn new(
        contract: &'static str,
        artifact: &RawAmenableArtifact,
        format: AcceptedAmenableArtifactFormat,
        decoded_counts: BTreeMap<String, usize>,
        unsupported_fields: Vec<UnsupportedField>,
        warnings: Vec<String>,
        source_record_ids: Vec<String>,
    ) -> Self {
        let source_record_kind_counts = source_record_kind_counts(&source_record_ids);
        Self {
            contract: contract.to_owned(),
            path: artifact.path().clone(),
            artifact_kind: artifact.kind(),
            format,
            digest: artifact.digest().to_owned(),
            parser_version: env!("CARGO_PKG_VERSION").to_owned(),
            decoded_counts,
            source_record_kind_counts,
            unsupported_fields,
            warnings,
            source_record_ids,
        }
    }

    /// Input artifact kind.
    #[must_use]
    pub const fn artifact_kind(&self) -> ArtifactKind {
        self.artifact_kind
    }

    /// Accepted format root.
    #[must_use]
    pub const fn format(&self) -> AcceptedAmenableArtifactFormat {
        self.format
    }
}

#[instrument(level = "trace", skip(source_record_ids))]
fn source_record_kind_counts(source_record_ids: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for source_record_id in source_record_ids {
        let kind = source_record_id
            .split_once(':')
            .map_or(source_record_id.as_str(), |(kind, _rest)| kind);
        *counts.entry(format!("source_record_{kind}")).or_default() += 1;
    }
    counts
}
