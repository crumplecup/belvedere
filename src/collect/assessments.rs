//! Collection helpers for proof assessment artifacts.

use crate::{
    contracts::{ArtifactKind, DecodedAssessmentStream, RawAmenableArtifact},
    exchanges::{AssessmentStreamIngestion, IngestionError},
};

use tracing::instrument;
/// Decode a raw proof assessment JSON Lines artifact.
///
/// # Errors
///
/// Returns an ingestion error if the artifact kind or JSON Lines shape is invalid.
#[instrument(level = "debug", skip(artifact))]
pub fn decode_assessments(
    artifact: RawAmenableArtifact,
) -> Result<DecodedAssessmentStream, IngestionError> {
    let (stream, _receipt) = AssessmentStreamIngestion.ingest(artifact)?;
    Ok(stream)
}

/// Build raw proof assessment evidence from bytes.
#[instrument(level = "debug", skip(path))]
#[must_use]
pub fn raw_assessments(path: impl Into<std::path::PathBuf>, bytes: Vec<u8>) -> RawAmenableArtifact {
    RawAmenableArtifact::new(path, ArtifactKind::ProofAssessments, bytes)
}
