//! Source-record attribution for landscape diff items.

use std::collections::BTreeSet;

use tracing::instrument;

use crate::contracts::{LandscapeAssessment, LandscapeClaim, LandscapeGap};

#[instrument(level = "debug", skip(claim, source_record_ids))]
pub(super) fn subject_source_record_ids_for_claim(
    claim: &LandscapeClaim,
    source_record_ids: &[String],
) -> Vec<String> {
    let mut ids = source_record_ids_for_claim_id(claim.id(), source_record_ids);
    for assessment_id in claim.assessment_ids() {
        extend_subject_source_record_ids(
            &mut ids,
            assessment_source_record_id(assessment_id),
            source_record_ids,
        );
    }
    nonempty_subject_source_record_ids(ids, source_record_ids)
}

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

#[instrument(level = "debug", skip(source_record_ids))]
pub(super) fn subject_source_record_ids_for_claim_id(
    claim_id: &str,
    source_record_ids: &[String],
) -> Vec<String> {
    nonempty_subject_source_record_ids(
        source_record_ids_for_claim_id(claim_id, source_record_ids),
        source_record_ids,
    )
}

#[instrument(level = "debug", skip(assessment, source_record_ids))]
pub(super) fn subject_source_record_ids_for_assessment(
    assessment: &LandscapeAssessment,
    source_record_ids: &[String],
) -> Vec<String> {
    let mut ids = Vec::new();
    extend_subject_source_record_ids(
        &mut ids,
        assessment_source_record_id(assessment.id()),
        source_record_ids,
    );
    ids.extend(source_record_ids_for_claim_id(
        assessment.proof_id(),
        source_record_ids,
    ));
    nonempty_subject_source_record_ids(ids, source_record_ids)
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

#[instrument(level = "debug", skip(source_record_ids))]
fn source_record_ids_for_claim_id(claim_id: &str, source_record_ids: &[String]) -> Vec<String> {
    source_record_ids
        .iter()
        .filter(|source_record_id| source_record_references_claim(source_record_id, claim_id))
        .cloned()
        .collect()
}

#[instrument(level = "trace", skip(source_record_id, claim_id))]
fn source_record_references_claim(source_record_id: &str, claim_id: &str) -> bool {
    let Some((_kind, identity)) = source_record_id.split_once(':') else {
        return false;
    };

    identity == claim_id
        || identity.starts_with(&format!("{claim_id}#"))
        || identity.starts_with(&format!("{claim_id}@"))
        || identity.starts_with(&format!("{claim_id}:"))
}

#[instrument(level = "trace", skip(assessment_id))]
fn assessment_source_record_id(assessment_id: &str) -> String {
    if assessment_id.starts_with("assessment:") {
        assessment_id.to_owned()
    } else {
        format!("assessment:{assessment_id}")
    }
}

#[instrument(level = "trace", skip(ids, source_record_ids))]
fn extend_subject_source_record_ids(
    ids: &mut Vec<String>,
    expected_source_record_id: String,
    source_record_ids: &[String],
) {
    if source_record_ids.contains(&expected_source_record_id) {
        ids.push(expected_source_record_id);
    }
}

#[instrument(level = "trace", skip(ids, source_record_ids))]
fn nonempty_subject_source_record_ids(
    ids: Vec<String>,
    source_record_ids: &[String],
) -> Vec<String> {
    if ids.is_empty() {
        source_record_ids.to_vec()
    } else {
        ids.into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
