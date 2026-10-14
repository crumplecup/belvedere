//! `elicit_ui` IR contracts and Amenable standard adapters.

use derive_getters::Getters;
use derive_new::new;
use tracing::instrument;

use crate::{
    certify::{LandscapeDiffReviewReceipt, UiIrReceipt, ViewModelReceipt},
    contracts::{
        ACCESSIBILITY_BASELINE_STANDARD_ID, AuditorNavigationModel, BelvedereContract,
        LandscapeDiffReviewModel, UI_PROJECTION_FIDELITY_STANDARD_ID,
    },
};

mod projection;
mod standards;

pub use projection::{
    DIFF_REVIEW_UI_IR_SECTION_ITEMS, DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION,
    DIFF_REVIEW_UI_IR_SECTION_SUMMARY, TraceWidgetRef, UI_IR_SECTION_ASSESSMENT_HISTORY,
    UI_IR_SECTION_ASSESSMENT_QUEUE, UI_IR_SECTION_ASSESSMENT_WORK, UI_IR_SECTION_BACKEND_COVERAGE,
    UI_IR_SECTION_CLAIM_DETAIL, UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS,
    UI_IR_SECTION_GAP_QUEUE, UI_IR_SECTION_OVERVIEW, UI_IR_SECTION_PROOF_CHAINS,
    UI_IR_SECTION_PROVENANCE_ROOTS, UI_IR_SECTION_REVIEW_WORKFLOW, UI_IR_SECTION_SOURCE_NAVIGATION,
    UI_IR_SECTION_WITNESS_SURFACE, UiIrProjection, UiIrProofBundle, UiIrSection, UiStandardMint,
    UiViewportVerification,
};
pub(crate) use projection::{TraceWidgetRefBuilder, UiIrProjectionBuilder};
pub use standards::{
    ElicitUiIrSourcedStandard, ElicitUiRenderCompleteStandard, ElicitUiRolePreservedStandard,
    ElicitUiWcagVerifiedStandard, MintedElicitUiStandard,
};

/// Input accepted by the UI IR projection exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct UiIrInput {
    #[getter(skip)]
    model: AuditorNavigationModel,
    source_view_model_receipt_id: String,
    source_view_model_receipt_digest: String,
}

/// Input accepted by the landscape-diff review UI IR projection exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct DiffReviewUiIrInput {
    #[getter(skip)]
    model: LandscapeDiffReviewModel,
    source_view_model_receipt_id: String,
    source_view_model_receipt_digest: String,
}

impl DiffReviewUiIrInput {
    /// Create UI IR input from diff review-model exchange outputs.
    #[must_use]
    #[instrument(level = "debug", skip(model, receipt))]
    pub fn from_diff_review_model(
        model: LandscapeDiffReviewModel,
        receipt: &LandscapeDiffReviewReceipt,
    ) -> Self {
        Self::new(
            model,
            receipt.receipt_id().to_owned(),
            receipt.receipt_digest().to_owned(),
        )
    }

    /// Diff review evidence to project.
    #[must_use]
    pub const fn model(&self) -> &LandscapeDiffReviewModel {
        &self.model
    }
}

impl UiIrInput {
    /// Create UI IR input directly from view-model exchange outputs.
    #[must_use]
    #[instrument(level = "debug", skip(model, view_model_receipt))]
    pub fn from_view_model(
        model: AuditorNavigationModel,
        view_model_receipt: &ViewModelReceipt,
    ) -> Self {
        Self::new(
            model,
            view_model_receipt.receipt_id().to_owned(),
            view_model_receipt.receipt_digest().to_owned(),
        )
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
            "source view-model receipt identity is available",
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
            "claim detail widgets preserve direct bases, resolved roots, verifier coverage, contracts, assessments, gaps, review state, next action, source counts, and trace IDs",
            "proof-chain widgets preserve chain topology and backend completeness labels",
            "review workflow widgets preserve task lanes, priorities, actions, rationales, and trace IDs",
            "assessment history widgets preserve proof-grouped reviewer flow, current review, superseded reviews, weak dimensions, comments, and trace IDs",
            "claim, assessment, witness, provenance, and gap widgets retain source trace references",
            "assessment widgets preserve review recency status and explicit recommendation-and-weakness groups in auditor-visible labels",
            "source record navigation widgets retain ingested source record IDs, artifact families, and artifact-local locator hints",
            "projection receipts record emitted section-root widget IDs, trace widget IDs, section population counts, and per-section source trace coverage",
            "projection receipts record WCAG verification coverage across configured auditor viewports",
            "IrSourced is asserted at the projection boundary and WcagVerified is proven from the emitted tree",
            "the projection records which elicit_ui props were lifted into Amenable standards",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[
            UI_PROJECTION_FIDELITY_STANDARD_ID,
            ACCESSIBILITY_BASELINE_STANDARD_ID,
        ]
    }
}

/// Contract for projecting landscape-diff review evidence into `elicit_ui`.
pub struct ElicitUiDiffReviewProjectionContract;

impl BelvedereContract for ElicitUiDiffReviewProjectionContract {
    type Input = DiffReviewUiIrInput;
    type Output = UiIrProjection;
    type Receipt = UiIrReceipt;

    const NAME: &'static str = "belvedere.ui_ir.elicit_ui_diff_review_projection";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "landscape diff review evidence was produced by the diff review-model exchange",
            "source diff review-model receipt identity is available",
            "diff review items retain changed subject IDs, review priorities, actions, and source record IDs",
            "elicit_ui props are minted into explicit Amenable standards at the boundary",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "the emitted tree is an elicit_ui VerifiedTree",
            "diff review summary, changed-item queue, and source navigation are represented as named UI regions",
            "each diff review widget preserves change kind, subject identity, review priority, review action, and trace IDs",
            "source record navigation widgets preserve every source record represented by the diff review model",
            "projection receipts record emitted section-root widget IDs, trace widget IDs, section population counts, and per-section source trace coverage",
            "projection receipts record WCAG verification coverage across configured auditor viewports",
            "IrSourced is asserted at the projection boundary and WcagVerified is proven from the emitted tree",
            "the projection records which elicit_ui props were lifted into Amenable standards",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[
            UI_PROJECTION_FIDELITY_STANDARD_ID,
            ACCESSIBILITY_BASELINE_STANDARD_ID,
        ]
    }
}
