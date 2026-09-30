//! Ingestion exchange implementations.

use std::panic::Location;

use derive_getters::Getters;
use serde::de::DeserializeOwned;
use thiserror::Error;
use tracing::instrument;

use crate::{
    certify::IngestionReceipt,
    contracts::{
        AcceptedAmenableArtifactFormat, ArtifactKind, AssessmentRecord,
        AssessmentStreamIngestionContract, BelvedereContract, DecodedAssessmentStream,
        DecodedRegistrySnapshot, RawAmenableArtifact, RegistryDumpIngestionContract,
    },
    exchanges::BelvedereExchange,
};

/// Errors emitted by ingestion exchanges.
#[derive(Debug, Error)]
#[error("ingestion failed at {file}:{line}: {kind}")]
pub struct IngestionError {
    #[source]
    kind: Box<IngestionErrorKind>,
    file: String,
    line: u32,
}

impl IngestionError {
    #[instrument(level = "debug", skip(expected, actual))]
    #[track_caller]
    fn wrong_artifact_kind(
        contract: &'static str,
        expected: ArtifactKind,
        actual: ArtifactKind,
    ) -> Self {
        Self::new(IngestionErrorKind::WrongArtifactKind(
            WrongArtifactKindError::new(contract, expected, actual),
        ))
    }

    #[instrument(level = "debug", skip(path, source))]
    #[track_caller]
    fn utf8(path: String, source: std::str::Utf8Error) -> Self {
        Self::new(IngestionErrorKind::Utf8(Utf8IngestionError::new(
            path, source,
        )))
    }

    #[instrument(level = "debug", skip(path, format, source))]
    #[track_caller]
    fn json(
        path: String,
        format: AcceptedAmenableArtifactFormat,
        source: serde_json::Error,
    ) -> Self {
        Self::new(IngestionErrorKind::Json(JsonIngestionError::new(
            path, format, source,
        )))
    }

    #[instrument(level = "debug", skip(path, format, source))]
    #[track_caller]
    fn json_line(
        path: String,
        format: AcceptedAmenableArtifactFormat,
        source_line: usize,
        source: serde_json::Error,
    ) -> Self {
        Self::new(IngestionErrorKind::JsonLine(JsonLineIngestionError::new(
            path,
            format,
            source_line,
            source,
        )))
    }

    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: IngestionErrorKind) -> Self {
        let location = Location::caller();
        Self {
            kind: Box::new(kind),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }

    /// Return wrong-kind details when this error came from contract mismatch.
    #[instrument(level = "debug", skip(self))]
    pub fn wrong_artifact_kind_source(&self) -> Option<&WrongArtifactKindError> {
        match self.kind.as_ref() {
            IngestionErrorKind::WrongArtifactKind(source) => Some(source),
            IngestionErrorKind::Utf8(_)
            | IngestionErrorKind::Json(_)
            | IngestionErrorKind::JsonLine(_) => None,
        }
    }
}

#[derive(Debug, Error)]
enum IngestionErrorKind {
    #[error(transparent)]
    WrongArtifactKind(WrongArtifactKindError),
    #[error(transparent)]
    Utf8(Utf8IngestionError),
    #[error(transparent)]
    Json(JsonIngestionError),
    #[error(transparent)]
    JsonLine(JsonLineIngestionError),
}

/// Artifact-kind mismatch details.
#[derive(Debug, Error, Getters)]
#[error("artifact kind {actual} does not satisfy {contract}; expected {expected} at {file}:{line}")]
pub struct WrongArtifactKindError {
    contract: String,
    #[getter(copy)]
    expected: ArtifactKind,
    #[getter(copy)]
    actual: ArtifactKind,
    file: String,
    line: u32,
}

impl WrongArtifactKindError {
    #[instrument(level = "debug", skip(expected, actual))]
    #[track_caller]
    fn new(contract: &'static str, expected: ArtifactKind, actual: ArtifactKind) -> Self {
        let location = Location::caller();
        Self {
            contract: contract.to_owned(),
            expected,
            actual,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
#[error("artifact {path} is not valid UTF-8 at {file}:{line}: {source}")]
struct Utf8IngestionError {
    path: String,
    source: std::str::Utf8Error,
    file: String,
    line: u32,
}

impl Utf8IngestionError {
    #[instrument(level = "debug", skip(path, source))]
    #[track_caller]
    fn new(path: String, source: std::str::Utf8Error) -> Self {
        let location = Location::caller();
        Self {
            path,
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
#[error("artifact {path} is not valid {format} at {file}:{line}: {source}")]
struct JsonIngestionError {
    path: String,
    format: AcceptedAmenableArtifactFormat,
    source: serde_json::Error,
    file: String,
    line: u32,
}

impl JsonIngestionError {
    #[instrument(level = "debug", skip(path, format, source))]
    #[track_caller]
    fn new(
        path: String,
        format: AcceptedAmenableArtifactFormat,
        source: serde_json::Error,
    ) -> Self {
        let location = Location::caller();
        Self {
            path,
            format,
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
#[error("artifact {path} has invalid {format} on line {source_line} at {file}:{line}: {source}")]
struct JsonLineIngestionError {
    path: String,
    format: AcceptedAmenableArtifactFormat,
    source_line: usize,
    source: serde_json::Error,
    file: String,
    line: u32,
}

impl JsonLineIngestionError {
    #[instrument(level = "debug", skip(path, format, source))]
    #[track_caller]
    fn new(
        path: String,
        format: AcceptedAmenableArtifactFormat,
        source_line: usize,
        source: serde_json::Error,
    ) -> Self {
        let location = Location::caller();
        Self {
            path,
            format,
            source_line,
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

/// Exchange for `dump-registry` JSON.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegistryDumpIngestion;

impl RegistryDumpIngestion {
    /// Ingest a raw registry dump artifact.
    ///
    /// # Errors
    ///
    /// Returns an ingestion error if the artifact kind or JSON shape is invalid.
    #[instrument(level = "debug", skip(self, input))]
    pub fn ingest(
        &self,
        input: RawAmenableArtifact,
    ) -> Result<(DecodedRegistrySnapshot, IngestionReceipt), IngestionError> {
        <Self as BelvedereExchange<RegistryDumpIngestionContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<RegistryDumpIngestionContract> for RegistryDumpIngestion {
    type Error = IngestionError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: RawAmenableArtifact,
    ) -> Result<(DecodedRegistrySnapshot, IngestionReceipt), Self::Error> {
        ensure_kind::<RegistryDumpIngestionContract>(
            &input,
            AcceptedAmenableArtifactFormat::RegistryDumpJson,
        )?;

        let snapshot: DecodedRegistrySnapshot =
            decode_json(&input, AcceptedAmenableArtifactFormat::RegistryDumpJson)?;
        let receipt = IngestionReceipt::new(
            RegistryDumpIngestionContract::NAME,
            &input,
            AcceptedAmenableArtifactFormat::RegistryDumpJson,
            snapshot.decoded_counts(),
            snapshot.unsupported_fields(),
            Vec::new(),
            snapshot.source_record_ids(),
        );
        Ok((snapshot, receipt))
    }
}

/// Exchange for proof assessment JSON Lines.
#[derive(Debug, Clone, Copy, Default)]
pub struct AssessmentStreamIngestion;

impl AssessmentStreamIngestion {
    /// Ingest a raw proof assessment artifact stream.
    ///
    /// # Errors
    ///
    /// Returns an ingestion error if the artifact kind or JSON Lines shape is invalid.
    #[instrument(level = "debug", skip(self, input))]
    pub fn ingest(
        &self,
        input: RawAmenableArtifact,
    ) -> Result<(DecodedAssessmentStream, IngestionReceipt), IngestionError> {
        <Self as BelvedereExchange<AssessmentStreamIngestionContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<AssessmentStreamIngestionContract> for AssessmentStreamIngestion {
    type Error = IngestionError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: RawAmenableArtifact,
    ) -> Result<(DecodedAssessmentStream, IngestionReceipt), Self::Error> {
        ensure_kind::<AssessmentStreamIngestionContract>(
            &input,
            AcceptedAmenableArtifactFormat::ProofAssessmentJsonLines,
        )?;

        let source = std::str::from_utf8(input.bytes())
            .map_err(|error| IngestionError::utf8(input.path().display().to_string(), error))?;

        let assessments: Result<Vec<AssessmentRecord>, IngestionError> = source
            .lines()
            .enumerate()
            .filter(|(_index, line)| !line.trim().is_empty())
            .map(|(index, line)| {
                serde_json::from_str::<AssessmentRecord>(line)
                    .map(|record| record.with_source_line(index + 1))
                    .map_err(|error| {
                        IngestionError::json_line(
                            input.path().display().to_string(),
                            AcceptedAmenableArtifactFormat::ProofAssessmentJsonLines,
                            index + 1,
                            error,
                        )
                    })
            })
            .collect();

        let stream = DecodedAssessmentStream::new(assessments?);
        let receipt = IngestionReceipt::new(
            AssessmentStreamIngestionContract::NAME,
            &input,
            AcceptedAmenableArtifactFormat::ProofAssessmentJsonLines,
            stream.decoded_counts(),
            stream.unsupported_fields(),
            Vec::new(),
            stream.source_record_ids(),
        );
        Ok((stream, receipt))
    }
}

#[instrument(level = "debug", skip(artifact, format))]
fn ensure_kind<C>(
    artifact: &RawAmenableArtifact,
    format: AcceptedAmenableArtifactFormat,
) -> Result<(), IngestionError>
where
    C: BelvedereContract<Input = RawAmenableArtifact>,
{
    let expected = format.artifact_kind();
    let actual = artifact.kind();
    if actual == expected {
        Ok(())
    } else {
        Err(IngestionError::wrong_artifact_kind(
            C::NAME,
            expected,
            actual,
        ))
    }
}

#[instrument(level = "debug", skip(artifact, format))]
fn decode_json<T>(
    artifact: &RawAmenableArtifact,
    format: AcceptedAmenableArtifactFormat,
) -> Result<T, IngestionError>
where
    T: DeserializeOwned,
{
    serde_json::from_slice(artifact.bytes())
        .map_err(|error| IngestionError::json(artifact.path().display().to_string(), format, error))
}
