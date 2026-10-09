//! Typed claim records for Belvedere pipeline standards.

use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

/// Named Belvedere pipeline claim that can be registered as an Amenable standard.
pub trait BelvederePipelineStandard {
    /// Stable claim identifier used in certificates and assessments.
    const CLAIM_ID: &'static str;

    /// Pipeline phase that owns this claim.
    const PHASE: &'static str;

    /// Auditor-facing meaning of the claim.
    const DESCRIPTION: &'static str;
}

/// Structured record for a named Belvedere pipeline claim.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvederePipelineClaim {
    claim_id: String,
    phase: String,
    description: String,
    standard_type: String,
}

impl BelvederePipelineClaim {
    /// Build a claim record from a typed Belvedere pipeline standard.
    #[instrument(level = "debug")]
    #[must_use]
    pub fn from_standard<S>() -> Self
    where
        S: BelvederePipelineStandard,
    {
        Self {
            claim_id: S::CLAIM_ID.to_owned(),
            phase: S::PHASE.to_owned(),
            description: S::DESCRIPTION.to_owned(),
            standard_type: std::any::type_name::<S>().to_owned(),
        }
    }
}
