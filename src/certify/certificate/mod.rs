//! Certificates emitted from Belvedere pipeline receipts.

mod digest;
mod run;
mod schema;
mod stage;

pub use run::{BelvedereInvocationSurface, BelvedereRunCertificate, BelvedereRunInvocation};
pub use schema::{
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
};
pub use stage::{BelvederePipelineStage, BelvedereStageCertificate};

pub(crate) use digest::{hex_digest, update_field, update_fields};
