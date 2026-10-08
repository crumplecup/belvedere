//! Receipts for Belvedere ingestion exchanges.

use std::{collections::BTreeMap, path::PathBuf};

use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::{
    certify::{hex_digest, update_field},
    contracts::{
        AcceptedAmenableArtifactFormat, ArtifactKind, RawAmenableArtifact, UnsupportedField,
    },
};

const INGESTION_RECEIPT_SCHEMA_ID: &str = "belvedere.receipts.ingestion.v1";
const INGESTION_RECEIPT_ID_PREFIX: &str = "belvedere.receipt.ingestion";

/// Audit receipt emitted by an ingestion exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct IngestionReceipt {
    contract: String,
    path: PathBuf,
    #[getter(skip)]
    artifact_kind: ArtifactKind,
    #[getter(skip)]
    format: AcceptedAmenableArtifactFormat,
    digest: String,
    parser_version: String,
    receipt_id: String,
    receipt_digest: String,
    decoded_counts: BTreeMap<String, usize>,
    source_record_kind_counts: BTreeMap<String, usize>,
    unsupported_field_record_counts: BTreeMap<String, usize>,
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
        let unsupported_field_record_counts = unsupported_field_record_counts(&unsupported_fields);
        let parser_version = env!("CARGO_PKG_VERSION").to_owned();
        let receipt_digest = ingestion_receipt_digest(IngestionReceiptDigestEvidence {
            contract,
            path: artifact.path(),
            artifact_kind: artifact.kind(),
            format,
            input_digest: artifact.digest(),
            parser_version: &parser_version,
            decoded_counts: &decoded_counts,
            source_record_kind_counts: &source_record_kind_counts,
            unsupported_field_record_counts: &unsupported_field_record_counts,
            unsupported_fields: &unsupported_fields,
            warnings: &warnings,
            source_record_ids: &source_record_ids,
        });
        let receipt_id = format!("{INGESTION_RECEIPT_ID_PREFIX}:{receipt_digest}");
        Self {
            contract: contract.to_owned(),
            path: artifact.path().clone(),
            artifact_kind: artifact.kind(),
            format,
            digest: artifact.digest().to_owned(),
            parser_version,
            receipt_id,
            receipt_digest,
            decoded_counts,
            source_record_kind_counts,
            unsupported_field_record_counts,
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

#[derive(Debug, Clone, Copy)]
struct IngestionReceiptDigestEvidence<'evidence> {
    contract: &'evidence str,
    path: &'evidence PathBuf,
    artifact_kind: ArtifactKind,
    format: AcceptedAmenableArtifactFormat,
    input_digest: &'evidence str,
    parser_version: &'evidence str,
    decoded_counts: &'evidence BTreeMap<String, usize>,
    source_record_kind_counts: &'evidence BTreeMap<String, usize>,
    unsupported_field_record_counts: &'evidence BTreeMap<String, usize>,
    unsupported_fields: &'evidence [UnsupportedField],
    warnings: &'evidence [String],
    source_record_ids: &'evidence [String],
}

#[instrument(level = "debug", skip(evidence))]
fn ingestion_receipt_digest(evidence: IngestionReceiptDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, INGESTION_RECEIPT_SCHEMA_ID);
    update_field(&mut hasher, evidence.contract);
    update_field(&mut hasher, &evidence.path.display().to_string());
    update_field(&mut hasher, evidence.artifact_kind.as_str());
    update_field(&mut hasher, evidence.format.schema_id());
    update_field(&mut hasher, evidence.input_digest);
    update_field(&mut hasher, evidence.parser_version);
    update_counts(&mut hasher, evidence.decoded_counts);
    update_counts(&mut hasher, evidence.source_record_kind_counts);
    update_counts(&mut hasher, evidence.unsupported_field_record_counts);
    for unsupported_field in evidence.unsupported_fields {
        update_field(&mut hasher, unsupported_field.record());
        update_field(&mut hasher, unsupported_field.field());
    }
    for warning in evidence.warnings {
        update_field(&mut hasher, warning);
    }
    for source_record_id in evidence.source_record_ids {
        update_field(&mut hasher, source_record_id);
    }
    hex_digest(hasher)
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

#[instrument(level = "trace", skip(hasher, counts))]
fn update_counts(hasher: &mut Sha256, counts: &BTreeMap<String, usize>) {
    for (name, count) in counts {
        update_field(hasher, name);
        update_field(hasher, &count.to_string());
    }
}

#[instrument(level = "trace", skip(unsupported_fields))]
fn unsupported_field_record_counts(
    unsupported_fields: &[UnsupportedField],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for unsupported_field in unsupported_fields {
        let record_family = unsupported_field
            .record()
            .split_once('[')
            .map_or(unsupported_field.record().as_str(), |(record, _index)| {
                record
            });
        *counts
            .entry(format!("unsupported_field_record_{record_family}"))
            .or_default() += 1;
    }
    counts
}
