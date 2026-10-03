//! `elicit_ui` IR contracts and Amenable standard adapters.

use tracing::instrument;

use crate::{
    certify::UiIrReceipt,
    contracts::{AuditorNavigationModel, BelvedereContract},
};

mod projection;
mod standards;

pub use projection::{
    TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_QUEUE, UI_IR_SECTION_BACKEND_COVERAGE,
    UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS, UI_IR_SECTION_GAP_QUEUE,
    UI_IR_SECTION_OVERVIEW, UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_PROVENANCE_ROOTS,
    UI_IR_SECTION_SOURCE_NAVIGATION, UI_IR_SECTION_WITNESS_SURFACE, UiIrProjection,
    UiIrProofBundle, UiIrSection, UiStandardMint,
};
pub(crate) use projection::{TraceWidgetRefBuilder, UiIrProjectionBuilder};
pub use standards::{
    ElicitUiIrSourcedStandard, ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
    ElicitUiWcagVerifiedStandard, MintedElicitUiStandard,
};

/// Input accepted by the UI IR projection exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiIrInput {
    model: AuditorNavigationModel,
}

impl UiIrInput {
    /// Create UI IR input from auditor navigation evidence.
    #[must_use]
    pub const fn new(model: AuditorNavigationModel) -> Self {
        Self { model }
    }

    /// Auditor navigation evidence to project.
    #[must_use]
    pub const fn model(&self) -> &AuditorNavigationModel {
        &self.model
    }
}

/// Provenance shared by Belvedere's temporary `elicit_ui` standards.
#[instrument(level = "debug")]
#[must_use]
pub fn elicit_ui_provenance() -> amenable::RustLanguageProvenance {
    amenable::RustLanguageProvenance::for_source("elicit_ui", "elicit_ui::contracts")
}

/// Contract for projecting Belvedere auditor evidence into `elicit_ui`.
pub struct ElicitUiProjectionContract;

impl BelvedereContract for ElicitUiProjectionContract {
    type Input = UiIrInput;
    type Output = UiIrProjection;
    type Receipt = UiIrReceipt;

    const NAME: &'static str = "belvedere.ui_ir.elicit_ui_projection";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "auditor navigation evidence was produced by the view-model exchange",
            "visible auditor items retain claim IDs and source record IDs",
            "elicit_ui props are minted into explicit Amenable standards at the boundary",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "the emitted tree is an elicit_ui VerifiedTree",
            "top-level auditor sections are represented as named UI regions",
            "top-level auditor sections carry stable section IDs in projection receipts",
            "auditor filter facets are represented as traceable UI controls",
            "backend coverage status is represented as traceable UI controls",
            "proof-chain widgets preserve chain topology and backend completeness labels",
            "claim, assessment, witness, provenance, and gap widgets retain source trace references",
            "assessment widgets preserve review recency status in auditor-visible labels",
            "source record navigation widgets retain ingested source record IDs and artifact families",
            "IrSourced is asserted at the projection boundary and WcagVerified is proven from the emitted tree",
            "the projection records which elicit_ui props were lifted into Amenable standards",
        ]
    }
}
