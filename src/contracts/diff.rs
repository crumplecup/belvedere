//! Landscape snapshot diff contracts and evidence types.

use std::collections::BTreeSet;

use derive_getters::Getters;
use derive_new::new;
use serde::Serialize;
use tracing::instrument;

use crate::{
    certify::{
        BelvedereCertificationError, BelvedereStageCertificate, LandscapeDiffReceipt,
        LandscapeReceipt,
    },
    contracts::{
        BelvedereContract, LANDSCAPE_DIFF_FIDELITY_STANDARD_ID, LandscapeGraph,
        REPORT_REPRODUCIBILITY_STANDARD_ID, TraceRef, require_transition,
    },
};

/// Two landscape snapshots accepted by the landscape diff exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct LandscapeDiffInput {
    #[getter(skip)]
    previous: LandscapeGraph,
    #[getter(skip)]
    current: LandscapeGraph,
    source_landscape_receipt_ids: Vec<String>,
    source_landscape_receipt_digests: Vec<String>,
}

impl LandscapeDiffInput {
    /// Create a diff input from an earlier and later landscape snapshot.
    #[must_use]
    #[instrument(level = "debug", skip(previous, current))]
    pub fn new(previous: LandscapeGraph, current: LandscapeGraph) -> Self {
        Self {
            previous,
            current,
            source_landscape_receipt_ids: Vec::new(),
            source_landscape_receipt_digests: Vec::new(),
        }
    }

    /// Create a diff input from landscapes and their modeling receipts.
    #[must_use]
    #[instrument(
        level = "debug",
        skip(previous, previous_receipt, current, current_receipt)
    )]
    pub fn from_landscape_receipts(
        previous: LandscapeGraph,
        previous_receipt: &LandscapeReceipt,
        current: LandscapeGraph,
        current_receipt: &LandscapeReceipt,
    ) -> Self {
        Self {
            previous,
            current,
            source_landscape_receipt_ids: vec![
                previous_receipt.receipt_id().to_owned(),
                current_receipt.receipt_id().to_owned(),
            ],
            source_landscape_receipt_digests: vec![
                previous_receipt.receipt_digest().to_owned(),
                current_receipt.receipt_digest().to_owned(),
            ],
        }
    }

    /// Earlier landscape snapshot.
    #[must_use]
    pub const fn previous(&self) -> &LandscapeGraph {
        &self.previous
    }

    /// Later landscape snapshot.
    #[must_use]
    pub const fn current(&self) -> &LandscapeGraph {
        &self.current
    }
}

/// UI-neutral diff between two landscape snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct LandscapeDiff {
    items: Vec<LandscapeDiffItem>,
    source_record_ids: Vec<String>,
}

/// One changed landscape item.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, new)]
pub struct LandscapeDiffItem {
    #[new(into)]
    item_id: String,
    #[getter(skip)]
    change_kind: LandscapeDiffChangeKind,
    #[getter(skip)]
    #[serde(flatten)]
    review_guidance: LandscapeDiffReviewGuidance,
    #[new(into)]
    subject_kind: String,
    #[new(into)]
    subject_id: String,
    #[new(into)]
    summary: String,
    #[getter(skip)]
    trace: TraceRef,
}

impl LandscapeDiffItem {
    /// Kind of snapshot change.
    #[must_use]
    pub const fn change_kind(&self) -> LandscapeDiffChangeKind {
        self.change_kind
    }

    /// Auditor review priority for this changed item.
    #[must_use]
    pub const fn review_priority(&self) -> LandscapeDiffReviewPriority {
        self.review_guidance.review_priority()
    }

    /// Recommended auditor action for this changed item.
    #[must_use]
    pub const fn review_action(&self) -> LandscapeDiffReviewAction {
        self.review_guidance.review_action()
    }

    /// Source trace for the changed subject.
    #[must_use]
    pub const fn trace(&self) -> &TraceRef {
        &self.trace
    }
}

/// Snapshot change category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandscapeDiffChangeKind {
    /// Item exists only in the current snapshot.
    Added,
    /// Item exists only in the previous snapshot.
    Removed,
    /// Item exists in both snapshots but its modeled evidence changed.
    Changed,
}

/// Auditor guidance attached to a landscape diff item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, new)]
pub struct LandscapeDiffReviewGuidance {
    review_priority: LandscapeDiffReviewPriority,
    review_action: LandscapeDiffReviewAction,
}

impl LandscapeDiffReviewGuidance {
    /// Auditor review priority.
    #[must_use]
    pub const fn review_priority(self) -> LandscapeDiffReviewPriority {
        self.review_priority
    }

    /// Recommended auditor action.
    #[must_use]
    pub const fn review_action(self) -> LandscapeDiffReviewAction {
        self.review_action
    }
}

/// Auditor triage priority for a landscape diff item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandscapeDiffReviewPriority {
    /// Change may affect certification reliance and should be reviewed first.
    Blocking,
    /// Change deserves focused review before accepting the current snapshot.
    Review,
    /// Change is useful context but is not expected to block certification.
    Informational,
}

/// Stable auditor action category for a landscape diff item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandscapeDiffReviewAction {
    /// Review evidence for a newly modeled claim.
    ReviewNewClaim,
    /// Compare changed evidence for an existing claim.
    CompareClaimEvidence,
    /// Confirm a removed claim was intentionally scoped away.
    ConfirmClaimRemoval,
    /// Review a newly attached assessment.
    ReviewNewAssessment,
    /// Compare changed reviewer evidence for an existing assessment.
    CompareAssessmentEvidence,
    /// Confirm a removed assessment does not reopen certification risk.
    ConfirmAssessmentRemoval,
    /// Triage a newly modeled certification gap.
    TriageNewGap,
    /// Reassess a changed certification gap.
    ReassessChangedGap,
    /// Verify a removed gap remains covered by current evidence.
    VerifyResolvedGap,
}

/// Landscape snapshot diff contract.
pub struct LandscapeDiffContract;

impl BelvedereContract for LandscapeDiffContract {
    type Input = LandscapeDiffInput;
    type Output = LandscapeDiff;
    type Receipt = LandscapeDiffReceipt;

    const NAME: &'static str = "belvedere.diff.landscape";

    #[instrument(level = "trace", skip(input, output, receipt))]
    fn verify_transition(
        input: &Self::Input,
        output: &Self::Output,
        receipt: &Self::Receipt,
    ) -> Result<(), BelvedereCertificationError> {
        let sources = input
            .previous()
            .source_record_ids()
            .iter()
            .chain(input.current().source_record_ids())
            .cloned()
            .collect::<BTreeSet<_>>();
        let item_ids = output
            .items()
            .iter()
            .map(|item| item.item_id().clone())
            .collect::<Vec<_>>();
        require_transition::<Self>([
            (
                "diff_source_binding",
                output.source_record_ids() == &sources.into_iter().collect::<Vec<_>>(),
            ),
            (
                "diff_receipt_input_binding",
                receipt.source_landscape_receipt_ids() == input.source_landscape_receipt_ids()
                    && receipt.source_landscape_receipt_digests()
                        == input.source_landscape_receipt_digests(),
            ),
            (
                "diff_receipt_output_binding",
                receipt.source_record_ids() == output.source_record_ids()
                    && receipt.diff_item_ids() == &item_ids
                    && receipt.counts().total_items() == output.items().len(),
            ),
            (
                "diff_item_identity_unique",
                item_ids.iter().collect::<BTreeSet<_>>().len() == item_ids.len(),
            ),
        ])?;
        BelvedereStageCertificate::landscape_diff(receipt)?;
        Ok(())
    }

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "both landscape snapshots were produced by the landscape modeling exchange",
            "source record identities remain available for both snapshots",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "claim additions, removals, and modeled evidence changes are surfaced deterministically",
            "assessment additions, removals, and reviewer evidence changes are surfaced deterministically",
            "gap additions, removals, and detail changes are surfaced deterministically",
            "each diff item receives a deterministic auditor review priority and action category",
            "each diff item preserves the source identities available from the compared snapshots",
            "gap diff traces retain source identities for the assessment that caused a replacement or retirement gap",
            "repeatable gaps are compared by their stable origin identities",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[
            LANDSCAPE_DIFF_FIDELITY_STANDARD_ID,
            REPORT_REPRODUCIBILITY_STANDARD_ID,
        ]
    }
}
