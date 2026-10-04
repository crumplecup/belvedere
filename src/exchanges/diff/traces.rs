//! Source-record attribution for landscape diff items.

use std::collections::BTreeSet;

use tracing::instrument;

use crate::contracts::{LandscapeAssessment, LandscapeClaim, LandscapeGap};
use crate::exchanges::source_records::{
    subject_source_record_ids_for_assessment, subject_source_record_ids_for_claim,
    subject_source_record_ids_for_claim_id,
};

#[instrument(level = "debug", skip(previous, current, source_record_ids))]
pub(super) fn changed_claim_source_record_ids(
    previous: &LandscapeClaim,
    current: &LandscapeClaim,
    source_record_ids: &[String],
) -> Vec<String> {
    let ids = subject_source_record_ids_for_claim(previous, source_record_ids)
        .into_iter()
        .chain(subject_source_record_ids_for_claim(
            current,
            source_record_ids,
        ))
        .collect::<BTreeSet<_>>();
    ids.into_iter().collect()
}

#[instrument(level = "debug", skip(previous, current, source_record_ids))]
pub(super) fn changed_assessment_source_record_ids(
    previous: &LandscapeAssessment,
    current: &LandscapeAssessment,
    source_record_ids: &[String],
) -> Vec<String> {
    let ids = subject_source_record_ids_for_assessment(previous, source_record_ids)
        .into_iter()
        .chain(subject_source_record_ids_for_assessment(
            current,
            source_record_ids,
        ))
        .collect::<BTreeSet<_>>();
    ids.into_iter().collect()
}

#[instrument(level = "debug", skip(previous, current, source_record_ids))]
pub(super) fn changed_gap_source_record_ids(
    previous: &LandscapeGap,
    current: &LandscapeGap,
    source_record_ids: &[String],
) -> Vec<String> {
    let ids = subject_source_record_ids_for_claim_id(previous.claim_id(), source_record_ids)
        .into_iter()
        .chain(subject_source_record_ids_for_claim_id(
            current.claim_id(),
            source_record_ids,
        ))
        .collect::<BTreeSet<_>>();
    ids.into_iter().collect()
}
