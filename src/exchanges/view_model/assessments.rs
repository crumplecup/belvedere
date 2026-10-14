//! Assessment queue and assessment work projections.

use std::collections::{BTreeMap, BTreeSet};

use derive_getters::Getters;
use tracing::instrument;

use crate::{
    contracts::{
        AssessmentHistoryItem, AssessmentHistoryItemBuilder, AssessmentQueueItem,
        AssessmentQueueItemBuilder, AssessmentWorkItem, AssessmentWorkItemBuilder, GapItem,
        LandscapeAssessment, LandscapeGapKind, Recommendation, ResolutionPath, Rubric, TraceRef,
    },
    error::BelvedereConstructionError,
};

use super::{gaps::gap_queue_lane, traces::trace_assessment};

#[derive(Getters)]
pub(super) struct AssessmentQueueGrouping {
    queue_lane: String,
    group_id: String,
    group_label: String,
    weak_dimensions: Vec<String>,
}

#[instrument(level = "debug", skip(assessments, source_record_ids))]
pub(super) fn project_assessment_history(
    assessments: &[LandscapeAssessment],
    source_record_ids: &[String],
) -> Result<Vec<AssessmentHistoryItem>, BelvedereConstructionError> {
    let mut assessments_by_proof: BTreeMap<&str, Vec<&LandscapeAssessment>> = BTreeMap::new();
    for assessment in assessments {
        assessments_by_proof
            .entry(assessment.proof_id())
            .or_default()
            .push(assessment);
    }

    let mut items = Vec::new();
    for (proof_id, mut proof_assessments) in assessments_by_proof {
        proof_assessments.sort_by_key(|assessment| assessment.timestamp());
        let Some(current) = current_assessment(&proof_assessments) else {
            continue;
        };
        items.push(
            AssessmentHistoryItemBuilder::default()
                .proof_id(proof_id)
                .current_assessment_id(current.id())
                .assessment_ids(
                    proof_assessments
                        .iter()
                        .map(|assessment| assessment.id().to_owned())
                        .collect::<Vec<_>>(),
                )
                .reviewer_ids(reviewer_ids(&proof_assessments))
                .recommendation_flow(recommendation_flow(&proof_assessments))
                .weak_dimensions(weak_dimensions(current.rubric()))
                .current_resolution_label(resolution_label(current.resolution_path()))
                .current_comment(current.comment())
                .latest_timestamp(current.timestamp())
                .history_count(proof_assessments.len())
                .superseded_count(
                    proof_assessments
                        .iter()
                        .filter(|assessment| !assessment.current())
                        .count(),
                )
                .trace(assessment_history_trace(
                    proof_id,
                    &proof_assessments,
                    source_record_ids,
                ))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AssessmentHistoryItem", error))?,
        );
    }

    Ok(items)
}

#[instrument(level = "debug", skip(assessments, source_record_ids))]
pub(super) fn project_assessment_queue(
    assessments: &[LandscapeAssessment],
    source_record_ids: &[String],
) -> Result<Vec<AssessmentQueueItem>, BelvedereConstructionError> {
    let superseded_counts = superseded_counts_by_current_assessment(assessments);
    assessments
        .iter()
        .map(|assessment| {
            let grouping = assessment_queue_grouping(assessment);
            AssessmentQueueItemBuilder::default()
                .assessment_id(assessment.id())
                .proof_id(assessment.proof_id())
                .current(assessment.current())
                .superseded_count(
                    superseded_counts
                        .get(assessment.id())
                        .copied()
                        .unwrap_or_default(),
                )
                .superseded_by(assessment.superseded_by().map(ToOwned::to_owned))
                .recommendation(assessment.recommendation())
                .queue_lane(grouping.queue_lane)
                .queue_group_id(grouping.group_id)
                .queue_group_label(grouping.group_label)
                .resolution_path(assessment.resolution_path())
                .weak_dimensions(grouping.weak_dimensions)
                .trace(trace_assessment(assessment, source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AssessmentQueueItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(assessments, gaps, source_record_ids))]
pub(super) fn project_assessment_work(
    assessments: &[LandscapeAssessment],
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<AssessmentWorkItem>, BelvedereConstructionError> {
    let mut items = Vec::new();
    for gap in gaps {
        if gap.kind() == LandscapeGapKind::MissingAssessment {
            items.push(
                AssessmentWorkItemBuilder::default()
                    .work_id(missing_assessment_work_id(gap))
                    .claim_id(gap.claim_id())
                    .queue_lane(gap_queue_lane(gap.kind()))
                    .action("assess proof coverage")
                    .resolution_label("collect proof assessment")
                    .reason(gap.detail())
                    .high_priority(gap.high_priority())
                    .trace(gap.trace().clone())
                    .build()
                    .map_err(|error| {
                        BelvedereConstructionError::new("AssessmentWorkItem", error)
                    })?,
            );
        }
    }

    for assessment in assessments {
        if !assessment.current() || assessment.recommendation() == Recommendation::Accept {
            continue;
        }
        let action = match assessment.recommendation() {
            Recommendation::Accept => "accept proof",
            Recommendation::Strengthen => "strengthen proof",
            Recommendation::Replace => "replace proof",
            Recommendation::Retire => "retire claim",
        };
        let high_priority = matches!(
            assessment.recommendation(),
            Recommendation::Replace | Recommendation::Retire
        );
        items.push(
            AssessmentWorkItemBuilder::default()
                .work_id(assessment_work_id(assessment))
                .claim_id(assessment.proof_id())
                .queue_lane(assessment_queue_lane(assessment.recommendation()))
                .action(action)
                .resolution_label(resolution_label(assessment.resolution_path()))
                .reason(format!(
                    "current assessment {} recommends {:?}",
                    assessment.id(),
                    assessment.recommendation()
                ))
                .high_priority(high_priority)
                .trace(trace_assessment(assessment, source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AssessmentWorkItem", error))?,
        );
    }

    Ok(items)
}

#[instrument(level = "trace", skip(gap))]
fn missing_assessment_work_id(gap: &GapItem) -> String {
    format!("assessment_work.missing_assessment:{}", gap.gap_id())
}

#[instrument(level = "trace", skip(assessment))]
fn assessment_work_id(assessment: &LandscapeAssessment) -> String {
    format!("assessment_work.recommendation:{}", assessment.id())
}

#[instrument(level = "trace", skip(assessment))]
pub(super) fn assessment_queue_grouping(
    assessment: &LandscapeAssessment,
) -> AssessmentQueueGrouping {
    let weak_dimensions = weak_dimensions(assessment.rubric());
    let queue_lane = assessment_queue_lane(assessment.recommendation());
    let (group_id, group_label) = assessment_queue_group(queue_lane, &weak_dimensions);
    AssessmentQueueGrouping {
        queue_lane: queue_lane.to_owned(),
        group_id,
        group_label,
        weak_dimensions,
    }
}

#[instrument(level = "trace", skip(weak_dimensions))]
fn assessment_queue_group(queue_lane: &str, weak_dimensions: &[String]) -> (String, String) {
    if weak_dimensions.is_empty() {
        (
            format!("assessment_queue.{queue_lane}:no_weak_dimensions"),
            format!("{queue_lane} / no weak dimensions"),
        )
    } else {
        let dimension_key = weak_dimensions.join("+");
        (
            format!("assessment_queue.{queue_lane}:{dimension_key}"),
            format!("{queue_lane} / {}", weak_dimensions.join(", ")),
        )
    }
}

#[instrument(level = "trace", skip(recommendation))]
fn assessment_queue_lane(recommendation: Recommendation) -> &'static str {
    match recommendation {
        Recommendation::Accept => "accepted",
        Recommendation::Strengthen => "strengthening",
        Recommendation::Replace => "replacement",
        Recommendation::Retire => "retirement",
    }
}

#[instrument(level = "trace", skip(path))]
fn resolution_label(path: Option<ResolutionPath>) -> &'static str {
    match path {
        Some(ResolutionPath::KeepCurrentProof) => "keep current proof",
        Some(ResolutionPath::StrengthenCurrentProof) => "strengthen current proof",
        Some(ResolutionPath::ReplaceWithProofSpecificModel) => "replace with proof-specific model",
        Some(ResolutionPath::ReplaceWithAccommodationModel) => "replace with accommodation model",
        Some(ResolutionPath::RetireClaim) => "retire claim",
        None => "no resolution path",
    }
}

#[instrument(level = "trace", skip(assessments))]
fn current_assessment<'a>(
    assessments: &[&'a LandscapeAssessment],
) -> Option<&'a LandscapeAssessment> {
    assessments
        .iter()
        .rev()
        .find(|assessment| assessment.current())
        .or_else(|| assessments.last())
        .copied()
}

#[instrument(level = "debug", skip(assessments))]
fn reviewer_ids(assessments: &[&LandscapeAssessment]) -> Vec<String> {
    let mut reviewers = assessments
        .iter()
        .map(|assessment| assessment.reviewer().to_owned())
        .collect::<Vec<_>>();
    reviewers.sort();
    reviewers.dedup();
    reviewers
}

#[instrument(level = "debug", skip(assessments))]
fn recommendation_flow(assessments: &[&LandscapeAssessment]) -> Vec<String> {
    assessments
        .iter()
        .map(|assessment| format!("{:?}", assessment.recommendation()))
        .collect()
}

#[instrument(level = "debug", skip(assessments, source_record_ids))]
fn assessment_history_trace(
    proof_id: &str,
    assessments: &[&LandscapeAssessment],
    source_record_ids: &[String],
) -> TraceRef {
    let mut ids = BTreeSet::new();
    for assessment in assessments {
        for source_record_id in trace_assessment(assessment, source_record_ids).source_record_ids()
        {
            ids.insert(source_record_id.to_owned());
        }
    }
    TraceRef::new(proof_id, ids.into_iter().collect())
}

#[instrument(level = "debug", skip(assessments))]
fn superseded_counts_by_current_assessment(
    assessments: &[LandscapeAssessment],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for assessment in assessments {
        if let Some(current_id) = assessment.superseded_by() {
            increment_count(&mut counts, current_id);
        }
    }
    counts
}

#[instrument(level = "trace", skip(counts))]
fn increment_count(counts: &mut BTreeMap<String, usize>, value: &str) {
    *counts.entry(value.to_owned()).or_default() += 1;
}

#[instrument(level = "debug", skip(rubric))]
fn weak_dimensions(rubric: Rubric) -> Vec<String> {
    [
        ("claim_alignment", rubric.claim_alignment()),
        ("assumption_adequacy", rubric.assumption_adequacy()),
        ("model_fidelity", rubric.model_fidelity()),
        ("assertion_strength", rubric.assertion_strength()),
        ("adversarial_coverage", rubric.adversarial_coverage()),
        ("clarity", rubric.clarity()),
    ]
    .into_iter()
    .filter(|(_dimension, score)| *score < 4)
    .map(|(dimension, _score)| dimension.to_owned())
    .collect()
}
