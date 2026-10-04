//! Proof-bearing transformation boundaries for Belvedere pipeline phases.

mod exchange;

mod diff;
mod ingestion;
mod landscape;
mod source_records;
mod ui_ir;
mod view_model;

pub use diff::LandscapeDiffing;
pub use exchange::BelvedereExchange;
pub use ingestion::{
    AssessmentStreamIngestion, IngestionError, RegistryDumpIngestion, WrongArtifactKindError,
};
pub use landscape::LandscapeModeling;
pub use ui_ir::{ElicitUiProjection, UiIrError};
pub use view_model::AuditorNavigationProjection;
