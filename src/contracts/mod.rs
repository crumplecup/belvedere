//! Named contracts for Belvedere pipeline boundaries.

mod contract;

mod diff;
mod diff_review;
mod ingestion;
mod landscape;
mod pipeline_claim;
mod standards;
mod ui_ir;
mod view_model;

pub use contract::BelvedereContract;
pub use diff::{
    LandscapeDiff, LandscapeDiffChangeKind, LandscapeDiffContract, LandscapeDiffInput,
    LandscapeDiffItem, LandscapeDiffReviewAction, LandscapeDiffReviewGuidance,
    LandscapeDiffReviewPriority,
};
pub use diff_review::{
    LandscapeDiffReviewContract, LandscapeDiffReviewInput, LandscapeDiffReviewItem,
    LandscapeDiffReviewModel, LandscapeDiffReviewSummary,
};
pub use ingestion::{
    AcceptedAmenableArtifactFormat, ArtifactKind, AssessmentRecord,
    AssessmentStreamIngestionContract, ContractRecordRecord, DecodedAssessmentStream,
    DecodedRegistrySnapshot, EvidenceLinkRecord, KaniProofRecord, ProofExecutionStatus,
    ProofRecordRecord, RawAmenableArtifact, Recommendation, RegistryDumpIngestionContract,
    ResolutionPath, Rubric, UnsupportedField, WitnessExportRecord,
};
pub(crate) use landscape::LandscapeGraphBuilder;
pub use landscape::{
    ClaimEdge, KaniProofSummary, LandscapeAssessment, LandscapeClaim, LandscapeContractFragment,
    LandscapeGap, LandscapeGapKind, LandscapeGraph, LandscapeInput, LandscapeModelingContract,
    WitnessSurfaceSummary,
};
pub use pipeline_claim::{BelvederePipelineClaim, BelvederePipelineStandard};
pub(crate) use standards::belvedere_pipeline_claims_for_standard_ids;
pub use standards::{
    ACCESSIBILITY_BASELINE_STANDARD_ID, AccessibilityBaselineStandard, GAP_SOUNDNESS_STANDARD_ID,
    GapSoundnessStandard, INPUT_FIDELITY_STANDARD_ID, InputFidelityStandard,
    LANDSCAPE_CONSTRUCTION_STANDARD_ID, LANDSCAPE_DIFF_FIDELITY_STANDARD_ID,
    LandscapeConstructionStandard, LandscapeDiffFidelityStandard,
    REPORT_REPRODUCIBILITY_STANDARD_ID, ReportReproducibilityStandard, SCHEMA_HONESTY_STANDARD_ID,
    SchemaHonestyStandard, UI_PROJECTION_FIDELITY_STANDARD_ID, UiProjectionFidelityStandard,
    VIEW_MODEL_FIDELITY_STANDARD_ID, ViewModelFidelityStandard, belvedere_pipeline_claims,
    belvedere_pipeline_provenance,
};
pub use ui_ir::{
    DIFF_REVIEW_UI_IR_SECTION_ITEMS, DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION,
    DIFF_REVIEW_UI_IR_SECTION_SUMMARY, DiffReviewUiIrInput, ElicitUiDiffReviewProjectionContract,
    ElicitUiIrSourcedStandard, ElicitUiProjectionContract, ElicitUiRenderCompleteStandard,
    ElicitUiRolePreservedStandard, ElicitUiWcagVerifiedStandard, MintedElicitUiStandard,
    TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_HISTORY, UI_IR_SECTION_ASSESSMENT_QUEUE,
    UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_DETAIL, UI_IR_SECTION_CLAIM_EXPLORER,
    UI_IR_SECTION_FILTERS, UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW,
    UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_REVIEW_WORKFLOW,
    UI_IR_SECTION_SOURCE_NAVIGATION, UI_IR_SECTION_WITNESS_SURFACE, UiIrInput, UiIrProjection,
    UiIrProofBundle, UiIrSection, UiStandardMint, UiViewportVerification, elicit_ui_provenance,
};
pub(crate) use ui_ir::{TraceWidgetRefBuilder, UiIrProjectionBuilder};
pub use view_model::{
    AssessmentHistoryItem, AssessmentQueueItem, AssessmentRecencySummary, AssessmentWorkItem,
    AuditorFilterFacet, AuditorNavigationContract, AuditorNavigationModel, BackendCoverageItem,
    ClaimDetailItem, ClaimExplorerCounts, ClaimExplorerItem, GapItem, GapSeverity, OverviewView,
    ProofChainBackendCoverage, ProofChainItem, ProofChainTopology, ProvenanceRootItem,
    ReviewWorkflowItem, ReviewWorkflowPriority, SourceNavigationItem, TraceRef, ViewModelInput,
    WitnessSurfaceItem,
};
pub(crate) use view_model::{
    AssessmentHistoryItemBuilder, AssessmentQueueItemBuilder, AssessmentWorkItemBuilder,
    AuditorFilterFacetBuilder, AuditorNavigationModelBuilder, BackendCoverageItemBuilder,
    ClaimDetailItemBuilder, GapItemBuilder, ProofChainBackendCoverageBuilder,
    ProofChainItemBuilder, ProvenanceRootItemBuilder, ReviewWorkflowItemBuilder,
    SourceNavigationItemBuilder, WitnessSurfaceItemBuilder,
};
