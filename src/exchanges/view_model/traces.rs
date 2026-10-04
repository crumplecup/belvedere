//! Source-record attribution for auditor navigation items.

use tracing::instrument;

use crate::{
    contracts::{LandscapeAssessment, TraceRef},
    exchanges::source_records::{
        subject_source_record_ids_for_assessment, subject_source_record_ids_for_claim_id,
    },
};

#[instrument(level = "debug", skip(source_record_ids))]
pub(super) fn trace(claim_id: &str, source_record_ids: &[String]) -> TraceRef {
    TraceRef::new(
        claim_id,
        subject_source_record_ids_for_claim_id(claim_id, source_record_ids),
    )
}

#[instrument(level = "debug", skip(assessment, source_record_ids))]
pub(super) fn trace_assessment(
    assessment: &LandscapeAssessment,
    source_record_ids: &[String],
) -> TraceRef {
    TraceRef::new(
        assessment.proof_id(),
        subject_source_record_ids_for_assessment(assessment, source_record_ids),
    )
}
