//! Pipeline stage names used by Belvedere certificates.

use std::fmt::Display;

use serde::Serialize;
use tracing::instrument;

/// Named stage in a complete Belvedere pipeline run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BelvederePipelineStage {
    /// Registry dump ingestion.
    RegistryIngestion,
    /// Proof assessment stream ingestion.
    AssessmentIngestion,
    /// Landscape graph construction.
    LandscapeModeling,
    /// Landscape snapshot comparison.
    LandscapeDiff,
    /// Landscape diff review-model construction.
    LandscapeDiffReview,
    /// Landscape diff review `elicit_ui` IR projection.
    LandscapeDiffReviewUiIrProjection,
    /// Auditor navigation view-model construction.
    AuditorNavigation,
    /// `elicit_ui` IR projection.
    UiIrProjection,
}

impl BelvederePipelineStage {
    /// Stable stage identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryIngestion => "registry_ingestion",
            Self::AssessmentIngestion => "assessment_ingestion",
            Self::LandscapeModeling => "landscape_modeling",
            Self::LandscapeDiff => "landscape_diff",
            Self::LandscapeDiffReview => "landscape_diff_review",
            Self::LandscapeDiffReviewUiIrProjection => "landscape_diff_review_ui_ir_projection",
            Self::AuditorNavigation => "auditor_navigation",
            Self::UiIrProjection => "ui_ir_projection",
        }
    }
}

impl Display for BelvederePipelineStage {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
