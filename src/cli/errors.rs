//! CLI error types.

use std::{io, panic::Location};

use thiserror::Error;
use tracing::instrument;

use super::BELVEDERE_CLI_HELP;

/// Error emitted when the Belvedere CLI receives invalid arguments.
#[derive(Debug, Error)]
#[error("invalid Belvedere CLI arguments at {file}:{line}: {usage}")]
pub struct BelvedereCliUsageError {
    usage: String,
    file: String,
    line: u32,
}

impl BelvedereCliUsageError {
    #[instrument(level = "debug")]
    #[track_caller]
    pub(crate) fn new() -> Self {
        let location = Location::caller();
        Self {
            usage: BELVEDERE_CLI_HELP.to_owned(),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

/// Error emitted when the CLI cannot write requested output.
#[derive(Debug, Error)]
#[error("Belvedere CLI output failed at {file}:{line}: {kind}")]
pub struct BelvedereCliOutputError {
    #[source]
    kind: Box<BelvedereCliOutputErrorKind>,
    file: String,
    line: u32,
}

impl BelvedereCliOutputError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    pub(super) fn serialize(source: serde_json::Error) -> Self {
        Self::new(BelvedereCliOutputErrorKind::Serialize(
            BelvedereCliSerializeError::new(source),
        ))
    }

    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    pub(super) fn write(source: io::Error) -> Self {
        Self::new(BelvedereCliOutputErrorKind::Write(
            BelvedereCliWriteError::new(source),
        ))
    }

    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: BelvedereCliOutputErrorKind) -> Self {
        let location = Location::caller();
        Self {
            kind: Box::new(kind),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
enum BelvedereCliOutputErrorKind {
    #[error(transparent)]
    Serialize(BelvedereCliSerializeError),
    #[error(transparent)]
    Write(BelvedereCliWriteError),
}

#[derive(Debug, Error)]
#[error("failed to serialize Belvedere CLI JSON output at {file}:{line}: {source}")]
struct BelvedereCliSerializeError {
    source: serde_json::Error,
    file: String,
    line: u32,
}

impl BelvedereCliSerializeError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: serde_json::Error) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
#[error("failed to write Belvedere CLI output at {file}:{line}: {source}")]
struct BelvedereCliWriteError {
    source: io::Error,
    file: String,
    line: u32,
}

impl BelvedereCliWriteError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: io::Error) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}
