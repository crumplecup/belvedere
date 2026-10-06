//! Landscape model contracts and evidence types.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

use crate::{
    certify::LandscapeReceipt,
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
#[derive(Debug, Clone, PartialEq)]
pub struct LandscapeInput {
    registry: DecodedRegistrySnapshot,
    assessments: DecodedAssessmentStream,
}

impl LandscapeInput {
    /// Create landscape input from decoded ingestion outputs.
    #[must_use]
    pub const fn new(
        registry: DecodedRegistrySnapshot,
        assessments: DecodedAssessmentStream,
    ) -> Self {
        Self {
            registry,
            assessments,
        }
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
            "missing proofs, missing assessments, replacement, retirement, and trusted or opaque support remain visible as gaps",
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
