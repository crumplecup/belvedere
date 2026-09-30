//! Auditor navigation view-model exchange implementations.

use std::collections::{BTreeMap, BTreeSet};

use tracing::instrument;

use crate::{
    certify::{ViewModelReceipt, ViewModelReceiptBuilder, ViewModelReceiptCountsBuilder},
    contracts::{
        AssessmentQueueItemBuilder, AuditorNavigationContract, AuditorNavigationModel,
        AuditorNavigationModelBuilder, BackendCoverageItemBuilder, BelvedereContract,
        ClaimExplorerCounts, ClaimExplorerItem, GapItem, GapItemBuilder, LandscapeClaim,
        LandscapeGap, LandscapeGapKind, OverviewView, ProofChainItem, ProofChainItemBuilder,
        ProvenanceRootItem, Rubric, SourceNavigationItemBuilder, TraceRef, ViewModelInput,
        WitnessSurfaceItemBuilder,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::BelvedereExchange,
};

const TARGET_VERIFIERS: [&str; 3] = ["kani", "creusot", "verus"];

/// Exchange from a landscape graph into UI-neutral auditor navigation evidence.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditorNavigationProjection;

impl AuditorNavigationProjection {
    /// Project a landscape graph into auditor navigation evidence.
    ///
    /// # Errors
    ///
    /// Returns an error if crate-internal evidence builders reject the projected
    /// navigation model or receipt. Missing or uncertain review facts are
    /// represented as view-model gap items.
    #[instrument(level = "debug", skip(self, input))]
    pub fn project(
        &self,
        input: ViewModelInput,
    ) -> Result<(AuditorNavigationModel, ViewModelReceipt), BelvedereError> {
        <Self as BelvedereExchange<AuditorNavigationContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<AuditorNavigationContract> for AuditorNavigationProjection {
    type Error = BelvedereError;

    #[instrument(level = "trace", skip(self, input))]
    fn exchange(
        &self,
        input: ViewModelInput,
    ) -> Result<(AuditorNavigationModel, ViewModelReceipt), Self::Error> {
        let landscape = input.landscape();
        let source_record_ids = landscape.source_record_ids().to_vec();
        let gaps = project_gaps(landscape.gaps(), &source_record_ids)?;
        let filter_facets = project_filter_facets(
            landscape.claims().values(),
            landscape.assessments(),
            &gaps,
            &source_record_ids,
        )?;
        let high_priority_gap_count = gaps.iter().filter(|gap| gap.high_priority()).count();
        let overview = OverviewView::new(
            landscape.claims().len(),
            landscape.root_claim_ids().len(),
            landscape.proof_count(),
            landscape.assessments().len(),
            gaps.len(),
            high_priority_gap_count,
        );

        let backend_coverage =
            project_backend_coverage(landscape.claims().values(), &source_record_ids)?;

        let claim_explorer: Vec<ClaimExplorerItem> = landscape
            .claims()
            .values()
            .map(|claim| {
                let claim_gap_count = gaps
                    .iter()
                    .filter(|gap| gap.claim_id() == claim.id())
                    .count();
                ClaimExplorerItem::new(
                    claim.id(),
                    claim.is_root(),
                    claim.bases().iter().cloned().collect(),
                    claim.proofs().iter().cloned().collect(),
                    ClaimExplorerCounts::new(
                        claim.contracts().len(),
                        claim.assessment_ids().len(),
                        claim_gap_count,
                    ),
                    trace(claim.id(), &source_record_ids),
                )
            })
            .collect();

        let proof_chains: Vec<ProofChainItem> = landscape
            .claims()
            .values()
            .map(|claim| {
                let claim_gap_count = gaps
                    .iter()
                    .filter(|gap| gap.claim_id() == claim.id())
                    .count();
                ProofChainItemBuilder::default()
                    .claim_id(claim.id())
                    .basis_ids(claim.bases().iter().cloned().collect::<Vec<_>>())
                    .root_ids(chain_roots(claim.id(), landscape.claims()))
                    .verifiers(claim.proofs().iter().cloned().collect::<Vec<_>>())
                    .gap_count(claim_gap_count)
                    .trace(trace(claim.id(), &source_record_ids))
                    .build()
                    .map_err(|error| BelvedereConstructionError::new("ProofChainItem", error))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let superseded_counts = superseded_counts_by_current_assessment(landscape.assessments());
        let assessment_queue = landscape
            .assessments()
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
                    .trace(trace(assessment.proof_id(), &source_record_ids))
                    .build()
                    .map_err(|error| BelvedereConstructionError::new("AssessmentQueueItem", error))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut witness_surface = Vec::new();
        for claim in landscape.claims().values() {
            for witness in claim.witness_surfaces() {
                let item = WitnessSurfaceItemBuilder::default()
                    .claim_id(claim.id())
                    .verifier(witness.verifier())
                    .destination_module(witness.destination_module())
                    .checked(witness.checked())
                    .trusted(witness.trusted())
                    .opaque(witness.opaque())
                    .trace(trace(claim.id(), &source_record_ids))
                    .build()
                    .map_err(|error| {
                        BelvedereConstructionError::new("WitnessSurfaceItem", error)
                    })?;
                witness_surface.push(item);
            }
        }

        let provenance_roots: Vec<ProvenanceRootItem> = landscape
            .root_claim_ids()
            .into_iter()
            .map(|claim_id| ProvenanceRootItem::new(claim_id, trace(claim_id, &source_record_ids)))
            .collect();

        let source_navigation = project_source_navigation(&source_record_ids)?;

        let counts = ViewModelReceiptCountsBuilder::default()
            .claim_cards(landscape.claims().len())
            .filter_facets(filter_facets.len())
            .backend_coverage_items(backend_coverage.len())
            .proof_chain_items(proof_chains.len())
            .assessment_items(landscape.assessments().len())
            .witness_items(witness_surface.len())
            .root_items(landscape.root_claim_ids().len())
            .gap_items(gaps.len())
            .source_items(source_navigation.len())
            .build()
            .map_err(|error| BelvedereConstructionError::new("ViewModelReceiptCounts", error))?;

        let model = AuditorNavigationModelBuilder::default()
            .overview(overview)
            .filter_facets(filter_facets)
            .backend_coverage(backend_coverage)
            .claim_explorer(claim_explorer)
            .proof_chains(proof_chains)
            .assessment_queue(assessment_queue)
            .witness_surface(witness_surface)
            .provenance_roots(provenance_roots)
            .gaps(gaps)
            .source_navigation(source_navigation)
            .source_record_ids(source_record_ids.clone())
            .build()
            .map_err(|error| BelvedereConstructionError::new("AuditorNavigationModel", error))?;
        let receipt = ViewModelReceiptBuilder::default()
            .contract(AuditorNavigationContract::NAME)
            .counts(counts)
            .source_record_ids(source_record_ids)
            .warnings(Vec::new())
            .build()
            .map_err(|error| BelvedereConstructionError::new("ViewModelReceipt", error))?;

        Ok((model, receipt))
    }
}

#[instrument(level = "debug", skip(claims, source_record_ids))]
fn project_backend_coverage<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    source_record_ids: &[String],
) -> Result<Vec<crate::contracts::BackendCoverageItem>, BelvedereConstructionError> {
    let mut items = Vec::new();
    for claim in claims {
        for verifier in TARGET_VERIFIERS {
            items.push(
                BackendCoverageItemBuilder::default()
                    .claim_id(claim.id())
                    .verifier(verifier)
                    .covered(claim.proofs().contains(verifier))
                    .trace(trace(claim.id(), source_record_ids))
                    .build()
                    .map_err(|error| {
                        BelvedereConstructionError::new("BackendCoverageItem", error)
                    })?,
            );
        }
    }
    Ok(items)
}

#[instrument(level = "debug", skip(assessments))]
fn superseded_counts_by_current_assessment(
    assessments: &[crate::contracts::LandscapeAssessment],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for assessment in assessments {
        if let Some(current_id) = assessment.superseded_by() {
            increment_count(&mut counts, current_id);
        }
    }
    counts
}

#[instrument(level = "debug", skip(source_record_ids))]
fn project_source_navigation(
    source_record_ids: &[String],
) -> Result<Vec<crate::contracts::SourceNavigationItem>, BelvedereConstructionError> {
    source_record_ids
        .iter()
        .map(|source_record_id| {
            SourceNavigationItemBuilder::default()
                .source_record_id(source_record_id)
                .label(format!("Source record {source_record_id}"))
                .trace(TraceRef::new(
                    format!("belvedere.source_record:{source_record_id}"),
                    vec![source_record_id.to_owned()],
                ))
                .build()
                .map_err(|error| BelvedereConstructionError::new("SourceNavigationItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(claims, assessments, gaps))]
fn project_filter_facets<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    assessments: &[crate::contracts::LandscapeAssessment],
    gaps: &[GapItem],
    source_record_ids: &[String],
) -> Result<Vec<crate::contracts::AuditorFilterFacet>, BelvedereConstructionError> {
    let mut verifier_counts = BTreeMap::new();
    for claim in claims {
        for verifier in claim.proofs() {
            increment_count(&mut verifier_counts, verifier);
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
        "gap_kind",
        "Gap kind",
        gap_kind_counts,
        source_record_ids,
        &mut facets,
    )?;

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
    facets: &mut Vec<crate::contracts::AuditorFilterFacet>,
) -> Result<(), BelvedereConstructionError> {
    for (value, count) in counts {
        let facet_id = format!("belvedere.filter.{group}:{value}");
        facets.push(
            crate::contracts::AuditorFilterFacetBuilder::default()
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

#[instrument(level = "debug", skip(claims))]
fn chain_roots(
    claim_id: &str,
    claims: &std::collections::BTreeMap<String, LandscapeClaim>,
) -> Vec<String> {
    let mut roots = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    collect_chain_roots(claim_id, claims, &mut visiting, &mut roots);
    roots.into_iter().collect()
}

#[instrument(level = "trace", skip(claims, visiting, roots))]
fn collect_chain_roots(
    claim_id: &str,
    claims: &std::collections::BTreeMap<String, LandscapeClaim>,
    visiting: &mut BTreeSet<String>,
    roots: &mut BTreeSet<String>,
) {
    if !visiting.insert(claim_id.to_owned()) {
        roots.insert(claim_id.to_owned());
        return;
    }
    let Some(claim) = claims.get(claim_id) else {
        roots.insert(claim_id.to_owned());
        visiting.remove(claim_id);
        return;
    };
    if claim.is_root() {
        roots.insert(claim.id().to_owned());
        visiting.remove(claim_id);
        return;
    }
    for basis in claim.bases() {
        collect_chain_roots(basis, claims, visiting, roots);
    }
    visiting.remove(claim_id);
}

#[instrument(level = "debug", skip(gaps))]
fn project_gaps(
    gaps: &[LandscapeGap],
    source_record_ids: &[String],
) -> Result<Vec<GapItem>, BelvedereConstructionError> {
    gaps.iter()
        .map(|gap| {
            GapItemBuilder::default()
                .claim_id(gap.claim_id())
                .kind(gap.kind())
                .detail(gap.detail())
                .high_priority(high_priority(gap.kind()))
                .trace(trace(gap.claim_id(), source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("GapItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(kind))]
fn high_priority(kind: LandscapeGapKind) -> bool {
    matches!(
        kind,
        LandscapeGapKind::ReplacementRecommended
            | LandscapeGapKind::RetirementRecommended
            | LandscapeGapKind::TrustedOrOpaqueWitnessSupport
    )
}

#[instrument(level = "debug")]
fn trace(claim_id: &str, source_record_ids: &[String]) -> TraceRef {
    TraceRef::new(claim_id, source_record_ids.to_vec())
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
