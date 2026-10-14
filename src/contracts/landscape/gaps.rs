//! Conservative review gaps surfaced by landscape modeling.

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

/// Conservative review gap surfaced by landscape modeling.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new, derive_setters::Setters, Serialize)]
#[setters(prefix = "with_", strip_option, into)]
pub struct LandscapeGap {
    #[setters(skip)]
    #[new(into)]
    claim_id: String,
    #[setters(skip)]
    #[getter(skip)]
    kind: LandscapeGapKind,
    #[setters(skip)]
    #[new(into)]
    detail: String,
    #[setters(skip)]
    #[getter(skip)]
    #[new(default)]
    triggering_assessment_id: Option<String>,
    #[setters(doc = "Set the stable origin of a repeatable gap kind.")]
    #[getter(skip)]
    #[new(default)]
    origin_key: Option<String>,
}

impl LandscapeGap {
    /// Gap kind.
    #[must_use]
    pub const fn kind(&self) -> LandscapeGapKind {
        self.kind
    }

    /// Assessment whose recommendation caused this gap, when applicable.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn triggering_assessment_id(&self) -> Option<&str> {
        self.triggering_assessment_id.as_deref()
    }

    /// Stable origin of a repeatable gap kind.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn origin_key(&self) -> Option<&str> {
        self.origin_key.as_deref()
    }

    #[instrument(level = "trace", skip(self))]
    pub(crate) fn with_triggering_assessment(mut self, assessment_id: &str) -> Self {
        self.triggering_assessment_id = Some(assessment_id.to_owned());
        self
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
