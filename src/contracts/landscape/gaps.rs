//! Conservative review gaps surfaced by landscape modeling.

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;

/// Conservative review gap surfaced by landscape modeling.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new, Serialize)]
pub struct LandscapeGap {
    #[new(into)]
    claim_id: String,
    #[getter(skip)]
    kind: LandscapeGapKind,
    #[new(into)]
    detail: String,
}

impl LandscapeGap {
    /// Gap kind.
    #[must_use]
    pub const fn kind(&self) -> LandscapeGapKind {
        self.kind
    }
}

/// Conservative gap categories computed by the landscape model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandscapeGapKind {
    /// A claim has no modeled proof coverage.
    MissingProof,
    /// A claim has proof coverage but no assessment event.
    MissingAssessment,
    /// The latest assessment asks for replacement.
    ReplacementRecommended,
    /// The latest assessment asks for retirement.
    RetirementRecommended,
    /// Witness support includes trusted or opaque support.
    TrustedOrOpaqueWitnessSupport,
    /// A registered proof did not complete as verified coverage.
    ProofExecutionUnresolved,
}
