//! Named contracts for Belvedere pipeline boundaries.

mod contract;

mod diff;
mod ingestion;
mod landscape;
mod standards;
mod ui_ir;
mod view_model;

pub use contract::BelvedereContract;
pub use diff::{
    LandscapeDiff, LandscapeDiffChangeKind, LandscapeDiffContract, LandscapeDiffInput,
    LandscapeDiffItem,
};
pub use ingestion::{
    AcceptedAmenableArtifactFormat, ArtifactKind, AssessmentRecord,
    AssessmentStreamIngestionContract, ContractRecordRecord, DecodedAssessmentStream,
    DecodedRegistrySnapshot, EvidenceLinkRecord, KaniProofRecord, ProofRecordRecord,
    RawAmenableArtifact, Recommendation, RegistryDumpIngestionContract, ResolutionPath, Rubric,
    UnsupportedField, WitnessExportRecord,
};
pub(crate) use landscape::LandscapeGraphBuilder;
pub use landscape::{
    ClaimEdge, KaniProofSummary, LandscapeAssessment, LandscapeClaim, LandscapeContractFragment,
    LandscapeGap, LandscapeGapKind, LandscapeGraph, LandscapeInput, LandscapeModelingContract,
    WitnessSurfaceSummary,
};
pub use standards::{
    ACCESSIBILITY_BASELINE_STANDARD_ID, AccessibilityBaselineStandard, BelvederePipelineStandard,
    GAP_SOUNDNESS_STANDARD_ID, GapSoundnessStandard, INPUT_FIDELITY_STANDARD_ID,
    InputFidelityStandard, LANDSCAPE_CONSTRUCTION_STANDARD_ID, LandscapeConstructionStandard,
    REPORT_REPRODUCIBILITY_STANDARD_ID, ReportReproducibilityStandard, SCHEMA_HONESTY_STANDARD_ID,
    SchemaHonestyStandard, UI_PROJECTION_FIDELITY_STANDARD_ID, UiProjectionFidelityStandard,
    VIEW_MODEL_FIDELITY_STANDARD_ID, ViewModelFidelityStandard, belvedere_pipeline_provenance,
};
pub use ui_ir::{
    ElicitUiIrSourcedStandard, ElicitUiProjectionContract, ElicitUiRenderCompleteStandard,
    ElicitUiRolePreservedStandard, ElicitUiWcagVerifiedStandard, MintedElicitUiStandard,
    TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_QUEUE, UI_IR_SECTION_BACKEND_COVERAGE,
    UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS, UI_IR_SECTION_GAP_QUEUE,
    UI_IR_SECTION_OVERVIEW, UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_PROVENANCE_ROOTS,
    UI_IR_SECTION_SOURCE_NAVIGATION, UI_IR_SECTION_WITNESS_SURFACE, UiIrInput, UiIrProjection,
    UiIrProofBundle, UiIrSection, UiStandardMint, UiViewportVerification, elicit_ui_provenance,
};
pub(crate) use ui_ir::{TraceWidgetRefBuilder, UiIrProjectionBuilder};
pub use view_model::{
    AssessmentQueueItem, AssessmentRecencySummary, AssessmentWorkItem, AuditorFilterFacet,
    AuditorNavigationContract, AuditorNavigationModel, BackendCoverageItem, ClaimExplorerCounts,
    ClaimExplorerItem, GapItem, GapSeverity, OverviewView, ProofChainBackendCoverage,
    ProofChainItem, ProofChainTopology, ProvenanceRootItem, SourceNavigationItem, TraceRef,
    ViewModelInput, WitnessSurfaceItem,
};
pub(crate) use view_model::{
    AssessmentQueueItemBuilder, AssessmentWorkItemBuilder, AuditorFilterFacetBuilder,
    AuditorNavigationModelBuilder, BackendCoverageItemBuilder, GapItemBuilder,
    ProofChainBackendCoverageBuilder, ProofChainItemBuilder, SourceNavigationItemBuilder,
    WitnessSurfaceItemBuilder,
};
