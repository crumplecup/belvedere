//! Auditor filter facet projections.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    contracts::{
        AuditorFilterFacet, AuditorFilterFacetBuilder, GapItem, LandscapeAssessment,
        LandscapeClaim, TraceRef,
    },
    error::BelvedereConstructionError,
};

use super::{assessments::assessment_queue_grouping, trace, traces::trace_assessment};

#[derive(Default)]
struct FacetTally {
    count: usize,
    source_record_ids: BTreeSet<String>,
}

impl FacetTally {
    #[instrument(level = "trace", skip(self, item_trace))]
    fn include(&mut self, item_trace: &TraceRef) {
        self.count += 1;
        self.source_record_ids
            .extend(item_trace.source_record_ids().iter().cloned());
    }

    #[instrument(level = "trace", skip(self))]
    fn into_trace(self, facet_id: &str) -> TraceRef {
        TraceRef::new(facet_id, self.source_record_ids.into_iter().collect())
    }
}

type LabeledFacetTally = (String, FacetTally);

#[instrument(level = "debug", skip(claims, assessments, gaps))]
pub(super) fn project_filter_facets<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    assessments: &[LandscapeAssessment],
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<AuditorFilterFacet>, BelvedereConstructionError> {
    let mut claim_family_counts = BTreeMap::new();
    let mut verifier_counts = BTreeMap::new();
    let mut support_kind_counts = BTreeMap::new();
    let mut package_counts = BTreeMap::new();
    let mut proof_status_counts = BTreeMap::new();
    for claim in claims {
        let claim_trace = trace(claim.id(), source_record_ids);
        let family = claim_family(claim.id());
        increment_tally(&mut claim_family_counts, &family, &claim_trace);
        for verifier in claim.proofs() {
            increment_tally(&mut verifier_counts, verifier, &claim_trace);
        }
        for witness in claim.witness_surfaces() {
            increment_tally(
                &mut support_kind_counts,
                witness.support_kind(),
                &claim_trace,
            );
        }
        for proof in claim.kani_proofs() {
            increment_tally(&mut package_counts, proof.package(), &claim_trace);
            if !proof.provides_coverage() {
                increment_tally(&mut proof_status_counts, proof.status_label(), &claim_trace);
            }
        }
    }

    let mut recommendation_counts = BTreeMap::new();
    let mut assessment_queue_group_counts = BTreeMap::new();
    for assessment in assessments {
        let assessment_trace = trace_assessment(assessment, source_record_ids);
        increment_tally(
            &mut recommendation_counts,
            &format!("{:?}", assessment.recommendation()),
            &assessment_trace,
        );
        let grouping = assessment_queue_grouping(assessment);
        increment_labeled_tally(
            &mut assessment_queue_group_counts,
            grouping.group_id(),
            &format!("Assessment queue group {}", grouping.group_label()),
            &assessment_trace,
        );
    }

    let mut gap_kind_counts = BTreeMap::new();
    let mut priority_tally = FacetTally::default();
    for gap in gaps {
        increment_tally(
            &mut gap_kind_counts,
            &format!("{:?}", gap.kind()),
            gap.trace(),
        );
        if gap.high_priority() {
            priority_tally.include(gap.trace());
        }
    }

    let mut facets = Vec::new();
    push_facets(
        "claim_family",
        "Claim family",
        claim_family_counts,
        &mut facets,
    )?;
    push_facets("verifier", "Verifier", verifier_counts, &mut facets)?;
    push_facets(
        "recommendation",
        "Recommendation",
        recommendation_counts,
        &mut facets,
    )?;
    push_labeled_facets(
        "assessment_queue_group",
        assessment_queue_group_counts,
        &mut facets,
    )?;
    push_facets(
        "support_kind",
        "Support kind",
        support_kind_counts,
        &mut facets,
    )?;
    push_facets("package", "Package", package_counts, &mut facets)?;
    push_facets(
        "proof_status",
        "Proof status",
        proof_status_counts,
        &mut facets,
    )?;
    push_facets("gap_kind", "Gap kind", gap_kind_counts, &mut facets)?;
    push_priority_facet(priority_tally, &mut facets)?;

    Ok(facets)
}

#[instrument(level = "trace", skip(claim_id))]
fn claim_family(claim_id: &str) -> String {
    claim_id
        .rsplit_once("::")
        .map_or_else(|| claim_id.to_owned(), |(family, _claim)| family.to_owned())
}

#[instrument(level = "trace", skip(counts, item_trace))]
fn increment_tally(counts: &mut BTreeMap<String, FacetTally>, value: &str, item_trace: &TraceRef) {
    counts
        .entry(value.to_owned())
        .or_default()
        .include(item_trace);
}

#[instrument(level = "trace", skip(counts, item_trace))]
fn increment_labeled_tally(
    counts: &mut BTreeMap<String, LabeledFacetTally>,
    value: &str,
    label: &str,
    item_trace: &TraceRef,
) {
    let (_label, tally) = counts
        .entry(value.to_owned())
        .or_insert_with(|| (label.to_owned(), FacetTally::default()));
    tally.include(item_trace);
}

#[instrument(level = "debug", skip(counts, facets))]
fn push_facets(
    group: &'static str,
    label_prefix: &'static str,
    counts: BTreeMap<String, FacetTally>,
    facets: &mut Vec<AuditorFilterFacet>,
) -> Result<(), BelvedereConstructionError> {
    for (value, tally) in counts {
        let facet_id = format!("belvedere.filter.{group}:{value}");
        let count = tally.count;
        facets.push(
            AuditorFilterFacetBuilder::default()
                .facet_id(facet_id.clone())
                .group(group)
                .value(value.clone())
                .label(format!("{label_prefix} {value}"))
                .count(count)
                .trace(tally.into_trace(&facet_id))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AuditorFilterFacet", error))?,
        );
    }
    Ok(())
}

#[instrument(level = "debug", skip(counts, facets))]
fn push_labeled_facets(
    group: &'static str,
    counts: BTreeMap<String, LabeledFacetTally>,
    facets: &mut Vec<AuditorFilterFacet>,
) -> Result<(), BelvedereConstructionError> {
    for (value, (label, tally)) in counts {
        let facet_id = format!("belvedere.filter.{group}:{value}");
        let count = tally.count;
        facets.push(
            AuditorFilterFacetBuilder::default()
                .facet_id(facet_id.clone())
                .group(group)
                .value(value.clone())
                .label(label)
                .count(count)
                .trace(tally.into_trace(&facet_id))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AuditorFilterFacet", error))?,
        );
    }
    Ok(())
}

#[instrument(level = "debug", skip(tally, facets))]
fn push_priority_facet(
    tally: FacetTally,
    facets: &mut Vec<AuditorFilterFacet>,
) -> Result<(), BelvedereConstructionError> {
    if tally.count == 0 {
        return Ok(());
    }
    let facet_id = "belvedere.filter.priority:high".to_owned();
    let count = tally.count;
    facets.push(
        AuditorFilterFacetBuilder::default()
            .facet_id(facet_id.clone())
            .group("priority")
            .value("high")
            .label("Priority high")
            .count(count)
            .trace(tally.into_trace(&facet_id))
            .build()
            .map_err(|error| BelvedereConstructionError::new("AuditorFilterFacet", error))?,
    );
    Ok(())
}
