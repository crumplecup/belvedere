//! Failure to attest a Belvedere transformation.

use std::panic::Location;

use thiserror::Error;
use tracing::instrument;

use crate::certify::BelvedereInvariantCheck;

/// A required certificate check failed, so no success certificate was issued.
#[derive(Debug, Error)]
#[error("{scope} certificate rejected at {file}:{line}: {check}: {detail}")]
pub struct BelvedereCertificationError {
    scope: String,
    check: String,
    detail: String,
    file: String,
    line: u32,
}

impl BelvedereCertificationError {
    #[track_caller]
    #[instrument(level = "debug", skip(scope, check, detail))]
    fn new(scope: String, check: String, detail: String) -> Self {
        let location = Location::caller();
        Self {
            scope,
            check,
            detail,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }

    #[track_caller]
    #[instrument(level = "debug", skip(scope, checks))]
    pub(crate) fn require_passing(
        scope: impl Into<String> + std::fmt::Debug,
        checks: &[BelvedereInvariantCheck],
    ) -> Result<(), Self> {
        if let Some(check) = checks.iter().find(|check| !check.passed()) {
            return Err(Self::new(
                scope.into(),
                check.name().clone(),
                check.detail().clone(),
            ));
        }
        Ok(())
    }
}
