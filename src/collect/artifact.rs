//! Raw artifact loading.

use std::{fs, panic::Location, path::Path};

use thiserror::Error;
use tracing::instrument;

use crate::contracts::{ArtifactKind, RawAmenableArtifact};

/// Errors emitted while collecting raw artifacts.
#[derive(Debug, Error)]
#[error("artifact read failed at {file}:{line}: {kind}")]
pub struct ArtifactReadError {
    #[source]
    kind: Box<ArtifactReadErrorKind>,
    file: String,
    line: u32,
}

impl ArtifactReadError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn io(source: std::io::Error) -> Self {
        Self::new(ArtifactReadErrorKind::Io(ArtifactReadIoError::new(source)))
    }

    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: ArtifactReadErrorKind) -> Self {
        let location = Location::caller();
        Self {
            kind: Box::new(kind),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

impl From<std::io::Error> for ArtifactReadError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn from(source: std::io::Error) -> Self {
        Self::io(source)
    }
}

#[derive(Debug, Error)]
enum ArtifactReadErrorKind {
    #[error(transparent)]
    Io(ArtifactReadIoError),
}

#[derive(Debug, Error)]
#[error("artifact I/O failed at {file}:{line}: {source}")]
struct ArtifactReadIoError {
    source: std::io::Error,
    file: String,
    line: u32,
}

impl ArtifactReadIoError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: std::io::Error) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

/// Read a raw Amenable artifact from disk.
///
/// # Errors
///
/// Returns an I/O error if the artifact cannot be read.
#[instrument(level = "info", skip(path, kind))]
pub fn read_raw_artifact(
    path: impl AsRef<Path>,
    kind: ArtifactKind,
) -> Result<RawAmenableArtifact, ArtifactReadError> {
    let path = path.as_ref();
    let bytes = fs::read(path)?;
    Ok(RawAmenableArtifact::new(path.to_owned(), kind, bytes))
}
