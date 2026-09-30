#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Belvedere's certification-navigation library.
//!
//! The crate is organized around Amenable-shaped pipeline boundaries:
//! contracts name the obligations, exchanges perform lawful transformations,
//! and certificates/receipts preserve audit evidence for reviewers.

mod certify;
mod cli;
mod collect;
mod contracts;
mod error;
mod exchanges;
mod run;
mod telemetry;

pub use certify::{
    BelvederePipelineStage, BelvedereRunCertificate, BelvedereStageCertificate, IngestionReceipt,
    LandscapeReceipt, LandscapeReceiptCounts, UiIrReceipt, UiIrReceiptCounts, ViewModelReceipt,
    ViewModelReceiptCounts,
};
pub use cli::{
    BelvedereCli, BelvedereCliOutput, BelvedereCliOutputError, BelvedereCliUsageError, run_cli,
    run_cli_with_writer,
};
pub use collect::{
    ArtifactReadError, decode_assessments, decode_registry_dump, raw_assessments,
    raw_registry_dump, read_raw_artifact,
};
pub use contracts::{
    AcceptedAmenableArtifactFormat, ArtifactKind, AssessmentQueueItem, AssessmentRecord,
    AssessmentStreamIngestionContract, AuditorNavigationContract, AuditorNavigationModel,
    BelvedereContract, ClaimEdge, ClaimExplorerCounts, ClaimExplorerItem, ContractRecordRecord,
    DecodedAssessmentStream, DecodedRegistrySnapshot, ElicitUiIrSourcedStandard,
    ElicitUiProjectionContract, ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
    ElicitUiWcagVerifiedStandard, EvidenceLinkRecord, GapItem, KaniProofRecord, KaniProofSummary,
    LandscapeAssessment, LandscapeClaim, LandscapeContractFragment, LandscapeGap, LandscapeGapKind,
    LandscapeGraph, LandscapeInput, LandscapeModelingContract, MintedElicitUiStandard,
    OverviewView, ProofRecordRecord, ProvenanceRootItem, RawAmenableArtifact, Recommendation,
    RegistryDumpIngestionContract, ResolutionPath, Rubric, TraceRef, TraceWidgetRef, UiIrInput,
    UiIrProjection, UiIrProofBundle, UiIrSection, UiStandardMint, UnsupportedField, ViewModelInput,
    WitnessExportRecord, WitnessSurfaceItem, WitnessSurfaceSummary, elicit_ui_provenance,
};
pub use error::{BelvedereConstructionError, BelvedereError};
pub use exchanges::{
    AssessmentStreamIngestion, AuditorNavigationProjection, BelvedereExchange, ElicitUiProjection,
    IngestionError, LandscapeModeling, RegistryDumpIngestion, UiIrError, WrongArtifactKindError,
};
pub use run::{BelvedereRunInput, BelvedereRunReport, run_belvedere_pipeline};
pub use telemetry::init_tracing;
