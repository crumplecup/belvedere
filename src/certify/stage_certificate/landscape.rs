//! Stage certificates for landscape and diff exchanges.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    certify::{
        BelvederePipelineStage, LandscapeDiffReceipt, LandscapeDiffReviewReceipt, LandscapeReceipt,
        diff_item_invariant_checks, diff_review_invariant_checks, diff_trace_invariant_checks,
        landscape_model_invariant_checks,
    },
    contracts::{LandscapeDiffContract, LandscapeDiffReviewContract, LandscapeModelingContract},
};

use super::{BelvedereStageCertificate, StageCertificateEvidence};
use crate::certify::contract_metadata::{
    contract_guarantees, contract_requirements, contract_standards,
};

impl BelvedereStageCertificate {
    /// Certificate the landscape modeling stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn landscape_modeling(receipt: &LandscapeReceipt) -> Self {
        let mut counts = BTreeMap::from([
            ("assessments".to_owned(), receipt.assessment_count()),
            ("claims".to_owned(), receipt.claim_count()),
            ("edges".to_owned(), receipt.edge_count()),
            ("gaps".to_owned(), receipt.gap_count()),
            (
                "landscape_items".to_owned(),
                receipt.landscape_item_ids().len(),
            ),
            ("proofs".to_owned(), receipt.proof_count()),
            ("roots".to_owned(), receipt.root_count()),
            (
                "source_receipts".to_owned(),
                receipt.source_receipt_ids().len(),
            ),
            ("warnings".to_owned(), receipt.warnings().len()),
        ]);
        counts.extend(receipt.item_kind_counts().clone());
        counts.extend(receipt.gap_kind_counts().clone());
        counts.extend(receipt.assessment_recency_counts().clone());
        let additional_invariant_checks = landscape_model_invariant_checks(
            &counts,
            receipt.item_kind_counts(),
            receipt.landscape_item_ids(),
        );
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::LandscapeModeling,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<LandscapeModelingContract>(),
            requires: contract_requirements::<LandscapeModelingContract>(),
            ensures: contract_guarantees::<LandscapeModelingContract>(),
            input_ids: receipt.source_receipt_ids().clone(),
            input_digests: receipt.source_receipt_digests().clone(),
            output_ids: receipt.landscape_item_ids().clone(),
            counts,
            additional_invariant_checks,
            warning_count: receipt.warnings().len(),
        })
    }

    /// Certificate the landscape snapshot diff stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn landscape_diff(receipt: &LandscapeDiffReceipt) -> Self {
        let mut counts = BTreeMap::from([
            ("added_items".to_owned(), receipt.counts().added_items()),
            ("changed_items".to_owned(), receipt.counts().changed_items()),
            ("removed_items".to_owned(), receipt.counts().removed_items()),
            ("total_items".to_owned(), receipt.counts().total_items()),
            (
                "source_landscape_receipts".to_owned(),
                receipt.source_landscape_receipt_ids().len(),
            ),
            ("warnings".to_owned(), receipt.warnings().len()),
        ]);
        counts.extend(receipt.item_kind_counts().clone());
        counts.extend(receipt.subject_kind_counts().clone());
        counts.extend(receipt.change_subject_counts().clone());
        counts.extend(receipt.review_priority_counts().clone());
        counts.extend(receipt.review_action_counts().clone());
        counts.extend(receipt.trace_source_counts().clone());
        counts.extend(receipt.trace_source_kind_counts().clone());
        let mut additional_invariant_checks = diff_trace_invariant_checks(
            receipt.counts().total_items(),
            receipt.trace_source_counts(),
            receipt.source_record_ids(),
            receipt.trace_source_record_ids(),
        );
        additional_invariant_checks.extend(diff_item_invariant_checks(
            &counts,
            receipt.item_kind_counts(),
            receipt.diff_item_ids(),
        ));
        additional_invariant_checks.extend(diff_review_invariant_checks(
            receipt.diff_item_ids(),
            receipt.review_priority_counts(),
            receipt.review_action_counts(),
        ));
        let input_ids = if receipt.source_landscape_receipt_ids().is_empty() {
            receipt.source_record_ids().clone()
        } else {
            receipt.source_landscape_receipt_ids().clone()
        };
        let input_digests = receipt.source_landscape_receipt_digests().clone();
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::LandscapeDiff,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<LandscapeDiffContract>(),
            requires: contract_requirements::<LandscapeDiffContract>(),
            ensures: contract_guarantees::<LandscapeDiffContract>(),
            input_ids,
            input_digests,
            output_ids: receipt.diff_item_ids().clone(),
            counts,
            additional_invariant_checks,
            warning_count: receipt.warnings().len(),
        })
    }

    /// Certificate the landscape diff review-model stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn landscape_diff_review(receipt: &LandscapeDiffReviewReceipt) -> Self {
        let mut counts = BTreeMap::from([
            (
                "blocking_items".to_owned(),
                receipt.counts().blocking_items(),
            ),
            (
                "informational_items".to_owned(),
                receipt.counts().informational_items(),
            ),
            ("review_items".to_owned(), receipt.counts().review_items()),
            (
                "source_record_ids".to_owned(),
                receipt.source_record_ids().len(),
            ),
            ("source_diffs".to_owned(), 1),
            ("total_items".to_owned(), receipt.counts().total_items()),
            ("warnings".to_owned(), receipt.warnings().len()),
        ]);
        counts.extend(receipt.change_kind_counts().clone());
        counts.extend(receipt.subject_kind_counts().clone());
        counts.extend(receipt.review_priority_counts().clone());
        counts.extend(receipt.review_action_counts().clone());
        counts.extend(receipt.trace_source_counts().clone());
        let additional_invariant_checks = diff_review_invariant_checks(
            receipt.review_item_ids(),
            receipt.review_priority_counts(),
            receipt.review_action_counts(),
        );
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::LandscapeDiffReview,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<LandscapeDiffReviewContract>(),
            requires: contract_requirements::<LandscapeDiffReviewContract>(),
            ensures: contract_guarantees::<LandscapeDiffReviewContract>(),
            input_ids: vec![receipt.source_diff_receipt_id().to_owned()],
            input_digests: vec![receipt.source_diff_receipt_digest().to_owned()],
            output_ids: receipt.review_item_ids().clone(),
            counts,
            additional_invariant_checks,
            warning_count: receipt.warnings().len(),
        })
    }
}
