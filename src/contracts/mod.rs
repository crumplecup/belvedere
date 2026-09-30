//! Named contracts for Belvedere pipeline boundaries.

mod contract;

mod ingestion;
mod landscape;
mod ui_ir;
mod view_model;

pub use contract::BelvedereContract;
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
pub use ui_ir::{
    ElicitUiIrSourcedStandard, ElicitUiProjectionContract, ElicitUiRenderCompleteStandard,
    ElicitUiRolePreservedStandard, ElicitUiWcagVerifiedStandard, MintedElicitUiStandard,
    TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_QUEUE, UI_IR_SECTION_CLAIM_EXPLORER,
    UI_IR_SECTION_FILTERS, UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW,
    UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_WITNESS_SURFACE,
    UiIrInput, UiIrProjection, UiIrProofBundle, UiIrSection, UiStandardMint, elicit_ui_provenance,
};
pub(crate) use ui_ir::{TraceWidgetRefBuilder, UiIrProjectionBuilder};
pub use view_model::{
    AssessmentQueueItem, AuditorFilterFacet, AuditorNavigationContract, AuditorNavigationModel,
    ClaimExplorerCounts, ClaimExplorerItem, GapItem, OverviewView, ProofChainItem,
    ProvenanceRootItem, TraceRef, ViewModelInput, WitnessSurfaceItem,
};
pub(crate) use view_model::{
    AssessmentQueueItemBuilder, AuditorFilterFacetBuilder, AuditorNavigationModelBuilder,
    GapItemBuilder, ProofChainItemBuilder, WitnessSurfaceItemBuilder,
};
