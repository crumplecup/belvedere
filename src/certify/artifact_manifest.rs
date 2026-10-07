//! Artifact manifests for auditor-facing Belvedere runs.

mod indexed_artifacts;
mod receipts;
mod runs;

use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::Serialize;

pub use indexed_artifacts::{
    BelvedereManifestInputArtifact, BelvedereManifestOutputArtifact,
    BelvedereManifestStageCertificate,
};
pub(crate) use receipts::{BelvedereDiffManifestReceipts, BelvedereSnapshotManifestReceipts};

/// Deterministic index of artifacts emitted by a Belvedere run.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereArtifactManifest {
    schema_id: String,
    schema_version: u16,
    run_certificate_id: String,
    run_certificate_digest: String,
    counts: BTreeMap<String, usize>,
    input_artifacts: Vec<BelvedereManifestInputArtifact>,
    stage_certificates: Vec<BelvedereManifestStageCertificate>,
    output_artifacts: Vec<BelvedereManifestOutputArtifact>,
}
