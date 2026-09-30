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
        Self {
            contract: contract.to_owned(),
            path: artifact.path().clone(),
            artifact_kind: artifact.kind(),
            format,
            digest: artifact.digest().to_owned(),
            parser_version: env!("CARGO_PKG_VERSION").to_owned(),
            decoded_counts,
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
