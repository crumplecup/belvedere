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
    BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID, BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereArtifactManifest, BelvedereInvariantCheck, BelvedereInvocationSurface,
    BelvedereManifestInputArtifact, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate, BelvederePipelineStage, BelvedereRunCertificate,
    BelvedereRunInvocation, BelvedereStageCertificate, IngestionReceipt, LandscapeDiffReceipt,
    LandscapeDiffReceiptCounts, LandscapeReceipt, LandscapeReceiptCounts, UiIrReceipt,
    UiIrReceiptCounts, ViewModelReceipt, ViewModelReceiptCounts,
};
pub use cli::{
    BelvedereCli, BelvedereCliOutput, BelvedereCliOutputError, BelvedereCliReport,
    BelvedereCliUsageError, run_cli, run_cli_with_writer,
};
pub use collect::{
    ArtifactReadError, decode_assessments, decode_registry_dump, raw_assessments,
    raw_registry_dump, read_raw_artifact,
};
pub use contracts::{
    ACCESSIBILITY_BASELINE_STANDARD_ID, AcceptedAmenableArtifactFormat,
    AccessibilityBaselineStandard, ArtifactKind, AssessmentHistoryItem, AssessmentQueueItem,
    AssessmentRecord, AssessmentStreamIngestionContract, AssessmentWorkItem, AuditorFilterFacet,
    AuditorNavigationContract, AuditorNavigationModel, BackendCoverageItem, BelvedereContract,
    BelvederePipelineStandard, ClaimDetailItem, ClaimEdge, ClaimExplorerCounts, ClaimExplorerItem,
    ContractRecordRecord, DecodedAssessmentStream, DecodedRegistrySnapshot,
    ElicitUiIrSourcedStandard, ElicitUiProjectionContract, ElicitUiRenderCompleteStandard,
    ElicitUiRolePreservedStandard, ElicitUiWcagVerifiedStandard, EvidenceLinkRecord,
    GAP_SOUNDNESS_STANDARD_ID, GapItem, GapSeverity, GapSoundnessStandard,
    INPUT_FIDELITY_STANDARD_ID, InputFidelityStandard, KaniProofRecord, KaniProofSummary,
    LANDSCAPE_CONSTRUCTION_STANDARD_ID, LandscapeAssessment, LandscapeClaim,
    LandscapeConstructionStandard, LandscapeContractFragment, LandscapeDiff,
    LandscapeDiffChangeKind, LandscapeDiffContract, LandscapeDiffInput, LandscapeDiffItem,
    LandscapeGap, LandscapeGapKind, LandscapeGraph, LandscapeInput, LandscapeModelingContract,
    MintedElicitUiStandard, OverviewView, ProofExecutionStatus, ProofRecordRecord,
    ProvenanceRootItem, REPORT_REPRODUCIBILITY_STANDARD_ID, RawAmenableArtifact, Recommendation,
    RegistryDumpIngestionContract, ReportReproducibilityStandard, ResolutionPath,
    ReviewWorkflowItem, ReviewWorkflowPriority, Rubric, SCHEMA_HONESTY_STANDARD_ID,
    SchemaHonestyStandard, SourceNavigationItem, TraceRef, TraceWidgetRef,
    UI_IR_SECTION_ASSESSMENT_HISTORY, UI_IR_SECTION_ASSESSMENT_QUEUE,
    UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_DETAIL, UI_IR_SECTION_CLAIM_EXPLORER,
    UI_IR_SECTION_FILTERS, UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW,
    UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_REVIEW_WORKFLOW,
    UI_IR_SECTION_SOURCE_NAVIGATION, UI_IR_SECTION_WITNESS_SURFACE,
    UI_PROJECTION_FIDELITY_STANDARD_ID, UiIrInput, UiIrProjection, UiIrProofBundle, UiIrSection,
    UiProjectionFidelityStandard, UiStandardMint, UiViewportVerification, UnsupportedField,
    VIEW_MODEL_FIDELITY_STANDARD_ID, ViewModelFidelityStandard, ViewModelInput,
    WitnessExportRecord, WitnessSurfaceItem, WitnessSurfaceSummary, belvedere_pipeline_provenance,
    elicit_ui_provenance,
};
pub use error::{BelvedereConstructionError, BelvedereError};
pub use exchanges::{
    AssessmentStreamIngestion, AuditorNavigationProjection, BelvedereExchange, ElicitUiProjection,
    IngestionError, LandscapeDiffing, LandscapeModeling, RegistryDumpIngestion, UiIrError,
    WrongArtifactKindError,
};
pub use run::{
    BelvedereLandscapeDiffRunInput, BelvedereLandscapeDiffRunReport, BelvedereRunInput,
    BelvedereRunReport, run_belvedere_landscape_diff, run_belvedere_pipeline,
};
pub use telemetry::init_tracing;
