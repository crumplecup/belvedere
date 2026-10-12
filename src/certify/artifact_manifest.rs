//! Artifact manifests for auditor-facing Belvedere runs.

mod accounting;
mod indexed_artifacts;
mod projection_outputs;
mod receipts;
mod run_outputs;
mod runs;

use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::Serialize;

use crate::certify::BelvedereInvariantCheck;

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
    id: String,
    run_certificate_id: String,
    run_certificate_digest: String,
    counts: BTreeMap<String, usize>,
    input_role_counts: BTreeMap<String, usize>,
    stage_role_counts: BTreeMap<String, usize>,
    stage_kind_counts: BTreeMap<String, usize>,
    output_role_counts: BTreeMap<String, usize>,
    output_kind_counts: BTreeMap<String, usize>,
    output_source_stage_counts: BTreeMap<String, usize>,
    invariant_checks: Vec<BelvedereInvariantCheck>,
    input_artifacts: Vec<BelvedereManifestInputArtifact>,
    stage_certificates: Vec<BelvedereManifestStageCertificate>,
    output_artifacts: Vec<BelvedereManifestOutputArtifact>,
    digest: String,
}
