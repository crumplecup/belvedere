//! Collection helpers for registry dump artifacts.

use crate::{
    contracts::{ArtifactKind, DecodedRegistrySnapshot, RawAmenableArtifact},
    exchanges::{IngestionError, RegistryDumpIngestion},
};

use tracing::instrument;
/// Decode a raw registry dump artifact.
///
/// # Errors
///
/// Returns an ingestion error if the artifact kind or JSON shape is invalid.
#[instrument(level = "debug", skip(artifact))]
pub fn decode_registry_dump(
    artifact: RawAmenableArtifact,
) -> Result<DecodedRegistrySnapshot, IngestionError> {
    let (snapshot, _receipt) = RegistryDumpIngestion.ingest(artifact)?;
    Ok(snapshot)
}

/// Build raw registry dump evidence from bytes.
#[instrument(level = "debug", skip(path))]
#[must_use]
pub fn raw_registry_dump(
    path: impl Into<std::path::PathBuf>,
    bytes: Vec<u8>,
) -> RawAmenableArtifact {
    RawAmenableArtifact::new(path, ArtifactKind::RegistryDump, bytes)
}
