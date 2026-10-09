//! Audit artifacts emitted by Belvedere pipeline exchanges.

mod artifact_manifest;
mod certificate_digest;
mod certificate_schema;
mod diff;
mod invariant;
mod landscape;
mod receipt;
mod run_certificate;
mod stage_certificate;
mod ui_ir;
mod view_model;

pub use artifact_manifest::{
    BelvedereArtifactManifest, BelvedereManifestInputArtifact, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate,
};
pub(crate) use artifact_manifest::{
    BelvedereDiffManifestReceipts, BelvedereSnapshotManifestReceipts,
};
pub(crate) use certificate_digest::{hex_digest, update_field, update_fields};
pub use certificate_schema::{
    BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID, BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
};
pub use diff::{LandscapeDiffReceipt, LandscapeDiffReceiptCounts};
pub(crate) use diff::{
    LandscapeDiffReceiptBuilder, LandscapeDiffReceiptDigestEvidenceBuilder,
    landscape_diff_receipt_identity,
};
pub use invariant::BelvedereInvariantCheck;
pub(crate) use invariant::{
    RunInputArtifactEvidence, RunInvariantEvidence, RunStageCertificateEvidence,
    diff_item_invariant_checks, diff_trace_invariant_checks, ingestion_record_invariant_checks,
    landscape_model_invariant_checks, run_invariant_checks, stage_invariant_checks,
    ui_ir_output_invariant_checks, ui_ir_trace_invariant_checks, view_model_item_invariant_checks,
    view_model_trace_invariant_checks,
};
pub use landscape::{LandscapeReceipt, LandscapeReceiptCounts};
pub(crate) use landscape::{
    LandscapeReceiptBuilder, LandscapeReceiptDigestEvidenceBuilder, landscape_receipt_identity,
};
pub use receipt::IngestionReceipt;
pub use run_certificate::{
    BelvedereInvocationSurface, BelvedereRunCertificate, BelvedereRunInvocation,
};
pub use stage_certificate::{BelvederePipelineStage, BelvedereStageCertificate};
pub use ui_ir::{UiIrReceipt, UiIrReceiptCounts};
pub(crate) use ui_ir::{
    UiIrReceiptBuilder, UiIrReceiptDigestEvidenceBuilder, ui_ir_receipt_identity,
};
pub use view_model::{ViewModelReceipt, ViewModelReceiptCounts};
pub(crate) use view_model::{
    ViewModelReceiptBuilder, ViewModelReceiptCountsBuilder, ViewModelReceiptDigestEvidenceBuilder,
    view_model_receipt_identity,
};
