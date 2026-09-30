//! Assessment queue and assessment work projections.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    contracts::{
        AssessmentQueueItem, AssessmentQueueItemBuilder, AssessmentWorkItem,
        AssessmentWorkItemBuilder, GapItem, LandscapeAssessment, LandscapeGapKind, Recommendation,
        Rubric,
    },
    error::BelvedereConstructionError,
};

use super::trace;

#[instrument(level = "debug", skip(assessments, source_record_ids))]
pub(super) fn project_assessment_queue(
    assessments: &[LandscapeAssessment],
    source_record_ids: &[String],
) -> Result<Vec<AssessmentQueueItem>, BelvedereConstructionError> {
    let superseded_counts = superseded_counts_by_current_assessment(assessments);
    assessments
        .iter()
        .map(|assessment| {
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
                .recommendation(assessment.recommendation())
                .resolution_path(assessment.resolution_path())
                .weak_dimensions(weak_dimensions(assessment.rubric()))
                .trace(trace(assessment.proof_id(), source_record_ids))
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
                    .claim_id(gap.claim_id())
                    .action("assess proof coverage")
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
                .claim_id(assessment.proof_id())
                .action(action)
                .reason(format!(
                    "current assessment {} recommends {:?}",
                    assessment.id(),
                    assessment.recommendation()
                ))
                .high_priority(high_priority)
                .trace(trace(assessment.proof_id(), source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AssessmentWorkItem", error))?,
        );
    }

    Ok(items)
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
