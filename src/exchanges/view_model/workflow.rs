//! Review workflow projection for auditor task navigation.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::{
    contracts::{
        GapItem, ProofChainItem, ReviewWorkflowItem, ReviewWorkflowItemBuilder,
        ReviewWorkflowPriority,
    },
    error::BelvedereConstructionError,
};

#[instrument(level = "debug", skip(proof_chains, gaps))]
pub(super) fn project_review_workflow(
    proof_chains: &[ProofChainItem],
    gaps: &[GapItem],
) -> Result<Vec<ReviewWorkflowItem>, BelvedereConstructionError> {
    let gaps_by_claim = gaps_by_claim(gaps);
    proof_chains
        .iter()
        .map(|chain| {
            let claim_gaps = gaps_by_claim
                .get(chain.claim_id().as_str())
                .map_or(&[][..], Vec::as_slice);
            let priority = workflow_priority(chain.review_state());
            let lane = workflow_lane(chain, claim_gaps);
            ReviewWorkflowItemBuilder::default()
                .task_id(format!("belvedere.review:{}:{lane}", chain.claim_id()))
                .claim_id(chain.claim_id())
                .lane(lane)
                .action(chain.next_action())
                .rationale(workflow_rationale(chain, claim_gaps))
                .priority(priority)
                .source_count(chain.trace().source_record_ids().len())
                .trace(chain.trace().clone())
                .build()
                .map_err(|error| BelvedereConstructionError::new("ReviewWorkflowItem", error))
        })
        .collect()
}

#[instrument(level = "debug", skip(gaps))]
fn gaps_by_claim<'gap>(gaps: &'gap [GapItem]) -> BTreeMap<&'gap str, Vec<&'gap GapItem>> {
    let mut by_claim = BTreeMap::new();
    for gap in gaps {
        by_claim
            .entry(gap.claim_id().as_str())
            .or_insert_with(Vec::new)
            .push(gap);
    }
    by_claim
}

#[instrument(level = "trace", skip(review_state))]
fn workflow_priority(review_state: &str) -> ReviewWorkflowPriority {
    match review_state {
        "blocked" => ReviewWorkflowPriority::Blocking,
        "elevated" => ReviewWorkflowPriority::Elevated,
        _ => ReviewWorkflowPriority::Ready,
    }
}

#[instrument(level = "trace", skip(chain, gaps))]
fn workflow_lane(chain: &ProofChainItem, gaps: &[&GapItem]) -> String {
    let lane = gaps
        .iter()
        .copied()
        .find(|gap| gap.high_priority())
        .or_else(|| gaps.first().copied())
        .map_or_else(
            || "certification_review".to_owned(),
            |gap| gap.queue_lane().to_owned(),
        );
    format!("{}:{}", chain.review_state(), lane.replace('_', "-"))
}

#[instrument(level = "trace", skip(chain, gaps))]
fn workflow_rationale(chain: &ProofChainItem, gaps: &[&GapItem]) -> String {
    if gaps.is_empty() {
        return format!(
            "proof chain is {} with {} source records",
            chain.review_state(),
            chain.trace().source_record_ids().len()
        );
    }

    let labels = gaps
        .iter()
        .map(|gap| format!("{:?}", gap.kind()))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "proof chain is {} with {} modeled gaps: {labels}",
        chain.review_state(),
        gaps.len()
    )
}
