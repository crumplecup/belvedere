//! Evidence emitted by the `elicit_ui` projection boundary.

use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use elicitation::Established;
use tracing::instrument;

use super::MintedElicitUiStandard;

/// A minted-standard entry returned with the UI IR projection.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct UiStandardMint {
    standard: String,
    prop: String,
    purpose: String,
}

impl UiStandardMint {
    /// Create a minted-standard entry from a typed adapter.
    #[must_use]
    #[instrument(level = "debug")]
    pub fn from_standard<S>() -> Self
    where
        S: MintedElicitUiStandard,
    {
        Self {
            standard: std::any::type_name::<S>().to_owned(),
            prop: S::PROP_NAME.to_owned(),
            purpose: S::PURPOSE.to_owned(),
        }
    }
}

/// Top-level section projected into the UI IR.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct UiIrSection {
    #[new(into)]
    name: String,
    #[getter(skip)]
    root_widget_id: u64,
}

impl UiIrSection {
    /// Root widget ID for the section.
    #[must_use]
    pub const fn root_widget_id(&self) -> u64 {
        self.root_widget_id
    }
}

/// A widget emitted from a traceable auditor navigation item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct TraceWidgetRef {
    #[getter(skip)]
    widget_id: u64,
    label: String,
    claim_id: String,
    source_record_ids: Vec<String>,
}

impl TraceWidgetRef {
    /// Widget ID in the emitted `elicit_ui` tree.
    #[must_use]
    pub const fn widget_id(&self) -> u64 {
        self.widget_id
    }
}

/// Proof tokens carried across the `elicit_ui` projection boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiIrProofBundle {
    ir_sourced: Established<elicit_ui::IrSourced>,
    wcag_verified: Established<elicit_ui::WcagVerified>,
}

impl UiIrProofBundle {
    /// Build the proof bundle.
    #[must_use]
    pub const fn new(
        ir_sourced: Established<elicit_ui::IrSourced>,
        wcag_verified: Established<elicit_ui::WcagVerified>,
    ) -> Self {
        Self {
            ir_sourced,
            wcag_verified,
        }
    }

    /// Proof that the tree is sourced from the UI IR pipeline.
    #[must_use]
    pub const fn ir_sourced(&self) -> Established<elicit_ui::IrSourced> {
        self.ir_sourced
    }

    /// Proof that the tree is WCAG verified under `elicit_ui` semantics.
    #[must_use]
    pub const fn wcag_verified(&self) -> Established<elicit_ui::WcagVerified> {
        self.wcag_verified
    }
}

/// `elicit_ui` projection evidence for downstream renderers.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct UiIrProjection {
    #[getter(skip)]
    tree: elicit_ui::VerifiedTree,
    sections: Vec<UiIrSection>,
    trace_widgets: Vec<TraceWidgetRef>,
    minted_standards: Vec<UiStandardMint>,
    #[getter(skip)]
    proofs: UiIrProofBundle,
}

impl UiIrProjection {
    /// Emitted `elicit_ui` tree.
    #[must_use]
    pub const fn tree(&self) -> &elicit_ui::VerifiedTree {
        &self.tree
    }

    /// Proof tokens minted from the emitted tree.
    #[must_use]
    pub const fn proofs(&self) -> &UiIrProofBundle {
        &self.proofs
    }
}
