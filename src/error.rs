//! Crate-level error umbrella.

use std::{error::Error as StdError, panic::Location};

use thiserror::Error;
use tracing::instrument;

use crate::{
    certify::BelvedereCertificationError,
    cli::{BelvedereCliOutputError, BelvedereCliUsageError},
    collect::ArtifactReadError,
    exchanges::{IngestionError, UiIrError},
};

/// Umbrella error for Belvedere CLI and pipeline failures.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct BelvedereError {
    kind: Box<BelvedereErrorKind>,
}

impl BelvedereError {
    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: BelvedereErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }
}

impl From<ArtifactReadError> for BelvedereError {
    #[track_caller]
    fn from(source: ArtifactReadError) -> Self {
        Self::new(BelvedereErrorKind::ArtifactRead(source))
    }
}

impl From<IngestionError> for BelvedereError {
    #[track_caller]
    fn from(source: IngestionError) -> Self {
        Self::new(BelvedereErrorKind::Ingestion(source))
    }
}

impl From<BelvedereCertificationError> for BelvedereError {
    #[track_caller]
    fn from(source: BelvedereCertificationError) -> Self {
        Self::new(BelvedereErrorKind::Certification(source))
    }
}

impl From<UiIrError> for BelvedereError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn from(source: UiIrError) -> Self {
        Self::new(BelvedereErrorKind::UiIr(source))
    }
}

impl From<BelvedereConstructionError> for BelvedereError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn from(source: BelvedereConstructionError) -> Self {
        Self::new(BelvedereErrorKind::Construction(source))
    }
}

impl From<BelvedereCliUsageError> for BelvedereError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn from(source: BelvedereCliUsageError) -> Self {
        Self::new(BelvedereErrorKind::CliUsage(source))
    }
}

impl From<BelvedereCliOutputError> for BelvedereError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn from(source: BelvedereCliOutputError) -> Self {
        Self::new(BelvedereErrorKind::CliOutput(source))
    }
}

#[derive(Debug, Error)]
enum BelvedereErrorKind {
    #[error(transparent)]
    CliUsage(BelvedereCliUsageError),
    #[error(transparent)]
    CliOutput(BelvedereCliOutputError),
    #[error(transparent)]
    ArtifactRead(ArtifactReadError),
    #[error(transparent)]
    Ingestion(IngestionError),
    #[error(transparent)]
    Certification(BelvedereCertificationError),
    #[error(transparent)]
    UiIr(UiIrError),
    #[error(transparent)]
    Construction(BelvedereConstructionError),
}

/// Error raised when crate-internal builders cannot construct evidence.
#[derive(Debug, Error)]
#[error("failed to build {target} at {file}:{line}: {source}")]
pub struct BelvedereConstructionError {
    target: String,
    #[source]
    source: Box<dyn StdError + Send + Sync + 'static>,
    file: String,
    line: u32,
}

impl BelvedereConstructionError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    pub(crate) fn new<E>(target: &'static str, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let location = Location::caller();
        Self {
            target: target.to_owned(),
            source: Box::new(source),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}
