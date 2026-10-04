//! Audit artifacts emitted by Belvedere pipeline exchanges.

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

pub(crate) use certificate_digest::{hex_digest, update_field, update_fields};
pub use certificate_schema::{
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
};
pub(crate) use diff::LandscapeDiffReceiptBuilder;
pub use diff::{LandscapeDiffReceipt, LandscapeDiffReceiptCounts};
pub use invariant::BelvedereInvariantCheck;
pub(crate) use invariant::{
    RunInputArtifactEvidence, RunInvariantEvidence, RunStageCertificateEvidence,
    run_invariant_checks, stage_invariant_checks,
};
pub(crate) use landscape::LandscapeReceiptBuilder;
pub use landscape::{LandscapeReceipt, LandscapeReceiptCounts};
pub use receipt::IngestionReceipt;
pub use run_certificate::{
    BelvedereInvocationSurface, BelvedereRunCertificate, BelvedereRunInvocation,
};
pub use stage_certificate::{BelvederePipelineStage, BelvedereStageCertificate};
pub(crate) use ui_ir::UiIrReceiptBuilder;
pub use ui_ir::{UiIrReceipt, UiIrReceiptCounts};
pub use view_model::{ViewModelReceipt, ViewModelReceiptCounts};
pub(crate) use view_model::{ViewModelReceiptBuilder, ViewModelReceiptCountsBuilder};
