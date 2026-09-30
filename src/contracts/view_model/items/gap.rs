//! Gap navigation items for auditor review.

use derive_builder::Builder;
use derive_getters::Getters;

use crate::contracts::{landscape::LandscapeGapKind, view_model::TraceRef};

/// Auditor-facing severity for a conservative review gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GapSeverity {
    /// Gap should block certification review until resolved.
    Blocking,
    /// Gap needs explicit reviewer attention before confidence can increase.
    Elevated,
    /// Gap is visible review work but is not currently blocking.
    Review,
}

impl GapSeverity {
    /// Whether this severity should be elevated in overview counts.
    #[must_use]
    pub const fn high_priority(self) -> bool {
        matches!(self, Self::Blocking | Self::Elevated)
    }

    /// Stable auditor-facing severity label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::Elevated => "elevated",
            Self::Review => "review",
        }
    }
}

/// Gap item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct GapItem {
    claim_id: String,
    #[getter(skip)]
    kind: LandscapeGapKind,
    #[getter(skip)]
    severity: GapSeverity,
    action: String,
    detail: String,
    #[getter(skip)]
    high_priority: bool,
    #[getter(skip)]
    trace: TraceRef,
}

impl GapItem {
    /// Gap kind.
    #[must_use]
    pub const fn kind(&self) -> LandscapeGapKind {
        self.kind
    }

    /// Auditor-facing severity.
    #[must_use]
    pub const fn severity(&self) -> GapSeverity {
        self.severity
    }

    /// Whether this item should be elevated in overview.
    #[must_use]
    pub const fn high_priority(&self) -> bool {
        self.high_priority
    }

    /// Trace reference.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}
