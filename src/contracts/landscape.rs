//! Landscape model contracts and evidence types.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

use crate::{
    certify::{IngestionReceipt, LandscapeReceipt},
    contracts::{
        BelvedereContract, DecodedAssessmentStream, DecodedRegistrySnapshot,
        GAP_SOUNDNESS_STANDARD_ID, LANDSCAPE_CONSTRUCTION_STANDARD_ID,
    },
};

mod claims;
mod gaps;

pub use claims::{
    ClaimEdge, KaniProofSummary, LandscapeAssessment, LandscapeClaim, LandscapeContractFragment,
    WitnessSurfaceSummary,
};
pub use gaps::{LandscapeGap, LandscapeGapKind};

/// Decoded evidence accepted by the landscape modeling exchange.
#[derive(Debug, Clone, PartialEq, Getters, new)]
pub struct LandscapeInput {
    #[getter(skip)]
    registry: DecodedRegistrySnapshot,
    #[getter(skip)]
    assessments: DecodedAssessmentStream,
    source_receipt_ids: Vec<String>,
    source_receipt_digests: Vec<String>,
}

impl LandscapeInput {
    /// Create landscape input directly from ingestion exchange outputs.
    #[must_use]
    #[instrument(
        level = "debug",
        skip(registry, registry_receipt, assessments, assessment_receipt)
    )]
    pub fn from_ingestion(
        registry: DecodedRegistrySnapshot,
        registry_receipt: &IngestionReceipt,
        assessments: DecodedAssessmentStream,
        assessment_receipt: &IngestionReceipt,
    ) -> Self {
        Self::new(
            registry,
            assessments,
            vec![
                registry_receipt.receipt_id().to_owned(),
                assessment_receipt.receipt_id().to_owned(),
            ],
            vec![
                registry_receipt.receipt_digest().to_owned(),
                assessment_receipt.receipt_digest().to_owned(),
            ],
        )
    }

    /// Decoded registry evidence.
    #[must_use]
    pub const fn registry(&self) -> &DecodedRegistrySnapshot {
        &self.registry
    }

    /// Decoded assessment evidence.
    #[must_use]
    pub const fn assessments(&self) -> &DecodedAssessmentStream {
        &self.assessments
    }
}

/// Auditor-facing defensive landscape graph.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct LandscapeGraph {
    #[getter(skip)]
    claims: BTreeMap<String, LandscapeClaim>,
    edges: Vec<ClaimEdge>,
    assessments: Vec<LandscapeAssessment>,
    gaps: Vec<LandscapeGap>,
    source_record_ids: Vec<String>,
}

impl LandscapeGraph {
    /// Claims by stable claim ID.
    #[must_use]
    pub const fn claims(&self) -> &BTreeMap<String, LandscapeClaim> {
        &self.claims
    }

    /// Explicit root claims with no further modeled basis or self-basis.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn root_claim_ids(&self) -> Vec<&str> {
        self.claims
            .values()
            .filter(|claim| claim.is_root())
            .map(|claim| claim.id().as_str())
            .collect()
    }

    /// Number of verifier coverage entries attached to claims.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn proof_count(&self) -> usize {
        self.claims()
            .values()
            .map(|claim| claim.proofs().len())
            .sum()
    }
}

/// Landscape modeling contract.
pub struct LandscapeModelingContract;

impl BelvedereContract for LandscapeModelingContract {
    type Input = LandscapeInput;
    type Output = LandscapeGraph;
    type Receipt = LandscapeReceipt;

    const NAME: &'static str = "belvedere.model.landscape";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "registry snapshot was produced by an ingestion exchange",
            "assessment stream was produced by an ingestion exchange",
            "source record identities remain available for traceability",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "evidence links become directed claim-to-basis edges",
            "proof records attach verifier coverage to claims",
            "contract records attach backend-native fragments to claims",
            "witness exports attach support summaries to claims",
            "Kani proof registrations remain addressable as proof claims",
            "assessment events attach by proof ID without losing reviewer vocabulary",
            "latest and superseded assessment events are distinguished per proof ID",
            "equal assessment timestamps use assessment ID as the deterministic recency tie-break for current status and gaps",
            "missing proofs, missing assessments, replacement, retirement, and trusted or opaque support remain visible as gaps",
            "replacement and retirement gaps identify the assessment whose recommendation caused them",
            "repeatable witness and unresolved-proof gaps retain stable origin identities",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[
            LANDSCAPE_CONSTRUCTION_STANDARD_ID,
            GAP_SOUNDNESS_STANDARD_ID,
        ]
    }
}
