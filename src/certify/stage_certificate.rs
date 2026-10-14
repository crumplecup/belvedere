//! Stage certificate construction.

use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

use crate::certify::{
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereCertificationError, BelvedereInvariantCheck, BelvederePipelineStage,
    stage_invariant_checks,
};

mod digest;
mod ingestion;
mod landscape;
mod ui_ir;
mod view_model;

use digest::stage_digest;

/// Certificate for one successful Belvedere pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereStageCertificate {
    schema_id: String,
    schema_version: u16,
    id: String,
    #[getter(skip)]
    stage: BelvederePipelineStage,
    contract: String,
    standards: Vec<String>,
    requires: Vec<String>,
    ensures: Vec<String>,
    input_ids: Vec<String>,
    input_digests: Vec<String>,
    output_ids: Vec<String>,
    counts: BTreeMap<String, usize>,
    invariant_checks: Vec<BelvedereInvariantCheck>,
    warning_count: usize,
    digest: String,
}

#[derive(Debug)]
struct StageCertificateEvidence {
    stage: BelvederePipelineStage,
    contract: String,
    standards: Vec<String>,
    requires: Vec<String>,
    ensures: Vec<String>,
    input_ids: Vec<String>,
    input_digests: Vec<String>,
    output_ids: Vec<String>,
    counts: BTreeMap<String, usize>,
    additional_invariant_checks: Vec<BelvedereInvariantCheck>,
    warning_count: usize,
}

impl BelvedereStageCertificate {
    /// Pipeline stage named by this certificate.
    #[must_use]
    pub const fn stage(&self) -> BelvederePipelineStage {
        self.stage
    }

    #[instrument(level = "debug", skip(evidence))]
    fn new(evidence: StageCertificateEvidence) -> Result<Self, BelvedereCertificationError> {
        let mut invariant_checks = stage_invariant_checks(
            &evidence.requires,
            &evidence.ensures,
            &evidence.input_ids,
            &evidence.counts,
            evidence.warning_count,
        );
        invariant_checks.extend(evidence.additional_invariant_checks.iter().cloned());
        BelvedereCertificationError::require_passing(
            evidence.stage.to_string(),
            &invariant_checks,
        )?;
        let digest = stage_digest(&evidence, &invariant_checks);
        let id = format!("belvedere.stage.{}:{digest}", evidence.stage);
        Ok(Self {
            schema_id: BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID.to_owned(),
            schema_version: BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
            id,
            stage: evidence.stage,
            contract: evidence.contract,
            standards: evidence.standards,
            requires: evidence.requires,
            ensures: evidence.ensures,
            input_ids: evidence.input_ids,
            input_digests: evidence.input_digests,
            output_ids: evidence.output_ids,
            counts: evidence.counts,
            invariant_checks,
            warning_count: evidence.warning_count,
            digest,
        })
    }
}
