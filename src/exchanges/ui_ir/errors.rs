//! Error types for the `elicit_ui` projection exchange.

use std::{error::Error as StdError, panic::Location};

use thiserror::Error;
use tracing::instrument;

use crate::{certify::BelvedereCertificationError, error::BelvedereConstructionError};

/// Errors raised while projecting into `elicit_ui`.
#[derive(Debug, Error)]
#[error("UI IR projection failed at {file}:{line}: {kind}")]
pub struct UiIrError {
    #[source]
    kind: Box<UiIrErrorKind>,
    file: String,
    line: u32,
}

impl UiIrError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    pub(super) fn elicit_ui(source: elicit_ui::UiError) -> Self {
        Self::new(UiIrErrorKind::ElicitUi(ElicitUiProjectionError::new(
            source,
        )))
    }

    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    pub(super) fn construction<E>(target: &'static str, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::new(UiIrErrorKind::Construction(
            BelvedereConstructionError::new(target, source),
        ))
    }

    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: UiIrErrorKind) -> Self {
        let location = Location::caller();
        Self {
            kind: Box::new(kind),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

impl From<BelvedereCertificationError> for UiIrError {
    #[track_caller]
    #[instrument(level = "debug", skip(source))]
    fn from(source: BelvedereCertificationError) -> Self {
        Self::new(UiIrErrorKind::Certification(source))
    }
}

#[derive(Debug, Error)]
enum UiIrErrorKind {
    #[error(transparent)]
    ElicitUi(ElicitUiProjectionError),
    #[error(transparent)]
    Construction(BelvedereConstructionError),
    #[error(transparent)]
    Certification(BelvedereCertificationError),
}

#[derive(Debug, Error)]
#[error("elicit_ui rejected UI IR projection at {file}:{line}: {source}")]
struct ElicitUiProjectionError {
    source: elicit_ui::UiError,
    file: String,
    line: u32,
}

impl ElicitUiProjectionError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: elicit_ui::UiError) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}
