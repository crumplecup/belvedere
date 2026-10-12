//! Shared CLI output option assembly.

use std::path::PathBuf;

use tracing::instrument;

use crate::cli::{
    BelvedereCliArtifact, BelvedereCliOutput, BelvedereCliOutputSink, BelvedereCliUsageError,
};
use crate::{BelvedereCli, BelvedereError};

impl BelvedereCli {
    #[instrument(level = "debug")]
    pub(in crate::cli::command) fn next_path(
        arguments: &[String],
        index: usize,
    ) -> Result<PathBuf, BelvedereError> {
        let Some(path) = arguments.get(index + 1) else {
            return Err(BelvedereCliUsageError::new().into());
        };
        Ok(PathBuf::from(path))
    }

    #[instrument(level = "trace", skip(artifact, next))]
    pub(in crate::cli::command) fn set_artifact(
        artifact: &mut Option<BelvedereCliArtifact>,
        next: BelvedereCliArtifact,
    ) -> Result<(), BelvedereError> {
        if artifact.replace(next).is_some() {
            return Err(BelvedereCliUsageError::new().into());
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(sink, next))]
    pub(in crate::cli::command) fn set_sink(
        sink: &mut Option<BelvedereCliOutputSink>,
        next: BelvedereCliOutputSink,
    ) -> Result<(), BelvedereError> {
        if sink.replace(next).is_some() {
            return Err(BelvedereCliUsageError::new().into());
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(artifact, sink))]
    pub(in crate::cli::command) fn output_from_parts(
        artifact: Option<BelvedereCliArtifact>,
        sink: Option<BelvedereCliOutputSink>,
    ) -> Result<BelvedereCliOutput, BelvedereError> {
        match (artifact, sink) {
            (None, None) => Ok(BelvedereCliOutput::LogSummary),
            (Some(artifact), Some(sink)) => Ok(BelvedereCliOutput::artifact(artifact, sink)),
            (Some(_), None) | (None, Some(_)) => Err(BelvedereCliUsageError::new().into()),
        }
    }
}
