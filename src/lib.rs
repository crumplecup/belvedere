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
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereInvariantCheck, BelvedereInvocationSurface, BelvederePipelineStage,
    BelvedereRunCertificate, BelvedereRunInvocation, BelvedereStageCertificate, IngestionReceipt,
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
    AssessmentStreamIngestionContract, AuditorFilterFacet, AuditorNavigationContract,
    AuditorNavigationModel, BelvedereContract, ClaimEdge, ClaimExplorerCounts, ClaimExplorerItem,
    ContractRecordRecord, DecodedAssessmentStream, DecodedRegistrySnapshot,
    ElicitUiIrSourcedStandard, ElicitUiProjectionContract, ElicitUiRenderCompleteStandard,
    ElicitUiRolePreservedStandard, ElicitUiWcagVerifiedStandard, EvidenceLinkRecord, GapItem,
    KaniProofRecord, KaniProofSummary, LandscapeAssessment, LandscapeClaim,
    LandscapeContractFragment, LandscapeGap, LandscapeGapKind, LandscapeGraph, LandscapeInput,
    LandscapeModelingContract, MintedElicitUiStandard, OverviewView, ProofRecordRecord,
    ProvenanceRootItem, RawAmenableArtifact, Recommendation, RegistryDumpIngestionContract,
    ResolutionPath, Rubric, SourceNavigationItem, TraceRef, TraceWidgetRef,
    UI_IR_SECTION_ASSESSMENT_QUEUE, UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS,
    UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW, UI_IR_SECTION_PROOF_CHAINS,
    UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_SOURCE_NAVIGATION, UI_IR_SECTION_WITNESS_SURFACE,
    UiIrInput, UiIrProjection, UiIrProofBundle, UiIrSection, UiStandardMint, UnsupportedField,
    ViewModelInput, WitnessExportRecord, WitnessSurfaceItem, WitnessSurfaceSummary,
    elicit_ui_provenance,
};
pub use error::{BelvedereConstructionError, BelvedereError};
pub use exchanges::{
    AssessmentStreamIngestion, AuditorNavigationProjection, BelvedereExchange, ElicitUiProjection,
    IngestionError, LandscapeModeling, RegistryDumpIngestion, UiIrError, WrongArtifactKindError,
};
pub use run::{BelvedereRunInput, BelvedereRunReport, run_belvedere_pipeline};
pub use telemetry::init_tracing;
