//! Proof assessment stream evidence decoded at the ingestion boundary.

use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::Deserialize;
use tracing::instrument;

use super::unsupported::{UnsupportedField, unsupported};

/// Decoded proof assessment stream evidence.
#[derive(Debug, Clone, PartialEq, Getters)]
pub struct DecodedAssessmentStream {
    assessments: Vec<AssessmentRecord>,
}

impl DecodedAssessmentStream {
    /// Create a decoded assessment stream.
    #[must_use]
    pub const fn new(assessments: Vec<AssessmentRecord>) -> Self {
        Self { assessments }
    }

    /// Stable source record identifiers derived from decoded assessments.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn source_record_ids(&self) -> Vec<String> {
        self.assessments
            .iter()
            .map(|record| {
                record.assessment_id.as_ref().map_or_else(
                    || format!("assessment:{}@{}", record.proof_id, record.timestamp()),
                    |id| format!("assessment:{id}"),
                )
            })
            .collect()
    }

    /// Count decoded record groups for receipts.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn decoded_counts(&self) -> BTreeMap<String, usize> {
        BTreeMap::from([("assessments".to_owned(), self.assessments.len())])
    }

    /// Uninterpreted fields preserved during assessment ingestion.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn unsupported_fields(&self) -> Vec<UnsupportedField> {
        self.assessments
            .iter()
            .flat_map(|record| {
                unsupported(
                    format!("assessment_line[{}]", record.source_line),
                    &record.extra,
                )
            })
            .collect()
    }
}

/// One persisted proof assessment event.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct AssessmentRecord {
    #[serde(default)]
    #[getter(skip)]
    version: Option<String>,
    #[serde(default)]
    #[getter(skip)]
    schema_version: Option<u8>,
    #[serde(default)]
    #[getter(skip)]
    assessment_id: Option<String>,
    proof_id: String,
    reviewer: String,
    #[serde(default)]
    #[getter(skip)]
    timestamp: Option<u64>,
    #[serde(default)]
    #[getter(skip)]
    timestamp_unix_seconds: Option<u64>,
    #[getter(skip)]
    rubric: Rubric,
    #[getter(skip)]
    recommendation: Recommendation,
    #[serde(default)]
    #[getter(skip)]
    resolution_path: Option<ResolutionPath>,
    comment: String,
    #[serde(skip)]
    #[getter(skip)]
    source_line: usize,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl AssessmentRecord {
    /// Attach the JSONL source line number.
    #[must_use]
    pub const fn with_source_line(mut self, line: usize) -> Self {
        self.source_line = line;
        self
    }

    /// Source line number in the JSON Lines artifact.
    #[must_use]
    pub const fn source_line(&self) -> usize {
        self.source_line
    }

    /// Assessment schema version.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Legacy assessment schema version.
    #[must_use]
    pub const fn schema_version(&self) -> Option<u8> {
        self.schema_version
    }

    /// Optional stable assessment ID.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn assessment_id(&self) -> Option<&str> {
        self.assessment_id.as_deref()
    }

    /// Normalized timestamp, regardless of legacy/current field spelling.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn timestamp(&self) -> u64 {
        self.timestamp.or(self.timestamp_unix_seconds).unwrap_or(0)
    }

    /// Rubric scores.
    #[must_use]
    pub const fn rubric(&self) -> Rubric {
        self.rubric
    }

    /// Reviewer recommendation.
    #[must_use]
    pub const fn recommendation(&self) -> Recommendation {
        self.recommendation
    }

    /// Operational resolution path.
    #[must_use]
    pub const fn resolution_path(&self) -> Option<ResolutionPath> {
        self.resolution_path
    }
}

/// Six scored proof-quality dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Rubric {
    claim_alignment: u8,
    assumption_adequacy: u8,
    model_fidelity: u8,
    assertion_strength: u8,
    adversarial_coverage: u8,
    clarity: u8,
}

impl Rubric {
    /// Claim-alignment score.
    #[must_use]
    pub const fn claim_alignment(self) -> u8 {
        self.claim_alignment
    }

    /// Assumption-adequacy score.
    #[must_use]
    pub const fn assumption_adequacy(self) -> u8 {
        self.assumption_adequacy
    }

    /// Model-fidelity score.
    #[must_use]
    pub const fn model_fidelity(self) -> u8 {
        self.model_fidelity
    }

    /// Assertion-strength score.
    #[must_use]
    pub const fn assertion_strength(self) -> u8 {
        self.assertion_strength
    }

    /// Adversarial-coverage score.
    #[must_use]
    pub const fn adversarial_coverage(self) -> u8 {
        self.adversarial_coverage
    }

    /// Clarity score.
    #[must_use]
    pub const fn clarity(self) -> u8 {
        self.clarity
    }
}

/// Reviewer recommendation vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
    /// Accept the proof as persuasive in its current form.
    Accept,
    /// Strengthen the proof while retaining its direction.
    Strengthen,
    /// Replace the proof with a substantively different argument.
    Replace,
    /// Stop relying on the current proof/claim.
    Retire,
}

/// Operational path for acting on an assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionPath {
    /// Keep the proof as-is.
    KeepCurrentProof,
    /// Strengthen the current proof.
    StrengthenCurrentProof,
    /// Replace with a proof-specific model.
    ReplaceWithProofSpecificModel,
    /// Replace with an accommodation model.
    ReplaceWithAccommodationModel,
    /// Retire the claim.
    RetireClaim,
}
