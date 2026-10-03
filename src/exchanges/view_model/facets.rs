//! Auditor filter facet projections.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    contracts::{
        AuditorFilterFacet, AuditorFilterFacetBuilder, GapItem, LandscapeAssessment, LandscapeClaim,
    },
    error::BelvedereConstructionError,
};

use super::trace;

#[instrument(level = "debug", skip(claims, assessments, gaps))]
pub(super) fn project_filter_facets<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    assessments: &[LandscapeAssessment],
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<AuditorFilterFacet>, BelvedereConstructionError> {
    let mut verifier_counts = BTreeMap::new();
    let mut support_kind_counts = BTreeMap::new();
    let mut package_counts = BTreeMap::new();
    for claim in claims {
        for verifier in claim.proofs() {
            increment_count(&mut verifier_counts, verifier);
        }
        for witness in claim.witness_surfaces() {
            increment_count(&mut support_kind_counts, witness.support_kind());
        }
        for proof in claim.kani_proofs() {
            increment_count(&mut package_counts, proof.package());
        }
    }

    let mut recommendation_counts = BTreeMap::new();
    for assessment in assessments {
        increment_count(
            &mut recommendation_counts,
            &format!("{:?}", assessment.recommendation()),
        );
    }

    let mut gap_kind_counts = BTreeMap::new();
    for gap in gaps {
        increment_count(&mut gap_kind_counts, &format!("{:?}", gap.kind()));
    }
    let high_priority_count = gaps.iter().filter(|gap| gap.high_priority()).count();

    let mut facets = Vec::new();
    push_facets(
        "verifier",
        "Verifier",
        verifier_counts,
        source_record_ids,
        &mut facets,
    )?;
    push_facets(
        "recommendation",
        "Recommendation",
        recommendation_counts,
        source_record_ids,
        &mut facets,
    )?;
    push_facets(
        "support_kind",
        "Support kind",
        support_kind_counts,
        source_record_ids,
        &mut facets,
    )?;
    push_facets(
        "package",
        "Package",
        package_counts,
        source_record_ids,
        &mut facets,
    )?;
    push_facets(
        "gap_kind",
        "Gap kind",
        gap_kind_counts,
        source_record_ids,
        &mut facets,
    )?;
    push_priority_facet(high_priority_count, source_record_ids, &mut facets)?;

    Ok(facets)
}

#[instrument(level = "trace", skip(counts))]
fn increment_count(counts: &mut BTreeMap<String, usize>, value: &str) {
    *counts.entry(value.to_owned()).or_default() += 1;
}

#[instrument(level = "debug", skip(counts, facets))]
fn push_facets(
    group: &'static str,
    label_prefix: &'static str,
    counts: BTreeMap<String, usize>,
    source_record_ids: &[String],
    facets: &mut Vec<AuditorFilterFacet>,
) -> Result<(), BelvedereConstructionError> {
    for (value, count) in counts {
        let facet_id = format!("belvedere.filter.{group}:{value}");
        facets.push(
            AuditorFilterFacetBuilder::default()
                .facet_id(facet_id.clone())
                .group(group)
                .value(value.clone())
                .label(format!("{label_prefix} {value}"))
                .count(count)
                .trace(trace(&facet_id, source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("AuditorFilterFacet", error))?,
        );
    }
    Ok(())
}

#[instrument(level = "debug", skip(source_record_ids, facets))]
fn push_priority_facet(
    high_priority_count: usize,
    source_record_ids: &[String],
    facets: &mut Vec<AuditorFilterFacet>,
) -> Result<(), BelvedereConstructionError> {
    if high_priority_count == 0 {
        return Ok(());
    }
    let facet_id = "belvedere.filter.priority:high".to_owned();
    facets.push(
        AuditorFilterFacetBuilder::default()
            .facet_id(facet_id.clone())
            .group("priority")
            .value("high")
            .label("Priority high")
            .count(high_priority_count)
            .trace(trace(&facet_id, source_record_ids))
            .build()
            .map_err(|error| BelvedereConstructionError::new("AuditorFilterFacet", error))?,
    );
    Ok(())
}
